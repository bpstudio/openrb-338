//! OpenRB-338 as a VST3 instrument for a host such as Ableton Live.
//!
//! The window is the desktop app's panel (`orb-ui`). The host's audio callback
//! is the same player the panel drives, so the knobs, the pattern and Play are
//! the sound the host records. MIDI and the host clock are OpenRB's, described
//! in [`midi`] and [`clock`]. The map and the build are written up in `PLAN.md`.

mod clock;
mod midi;
mod out;
mod pull;
mod ui_host;

use std::any::Any;
use std::sync::{Arc, Mutex};

use nih_plug::prelude::*;
use orb_engine::tables::Tables;
use orb_player::Audio;
use orb_ui::app::App;
use orb_ui::link::Link;
use orb_ui::model::Panel;

use clock::{Clock, Host};
use midi::{Event, Keys, byte_from_unit};
use out::Out;
use pull::CHANS;

pub struct OpenRb {
    params: Arc<PlugParams>,
    /// The panel. The editor draws it; the audio thread never locks it.
    app: Arc<Mutex<App>>,
    audio: Audio,
    clock: Clock,
    keys: Keys,
    out: Out,
    scratch: Vec<[f32; CHANS]>,
}

/// No automatable stand-ins: the panel is the control surface.
#[derive(Default, Params)]
struct PlugParams {}

impl Default for OpenRb {
    fn default() -> Self {
        let samples = rbm::standard::builtin_samples();
        let (link, audio) = Link::new(Arc::new(Tables::new()), Some(samples.clone()));
        let mut app = App::with_link(Panel::new(), link, None);
        app.samples = Some(samples);
        app.menu = true;
        app.enable_logo_anim();
        app.settings = orb_ui::settings::Settings::load();
        app.action(|app| app.start(None));
        OpenRb {
            params: Arc::new(PlugParams::default()),
            app: Arc::new(Mutex::new(app)),
            audio,
            clock: Clock::new(),
            keys: Keys::default(),
            out: Out::new(44_100.0, 512),
            scratch: vec![[0.0; CHANS]; 512],
        }
    }
}

impl OpenRb {
    fn load_song_env(&mut self) {
        let Ok(path) = std::env::var("ORB_VST_SONG") else {
            return;
        };
        if path.is_empty() {
            return;
        }
        let opened = match orb_ui::session::open_song(std::path::Path::new(&path)) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("OpenRB-338: {path}: {e}");
                return;
            }
        };
        if let Ok(mut app) = self.app.lock() {
            app.action(|app| app.start(Some((std::path::PathBuf::from(path), opened))));
        }
    }
}

impl Plugin for OpenRb {
    const NAME: &'static str = "OpenRB-338";
    const VENDOR: &'static str = "OpenRB-338";
    const URL: &'static str = "https://github.com/pingw33n/openrb-338";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_output_ports: &[
            new_nonzero_u32(2),
            new_nonzero_u32(2),
            new_nonzero_u32(2),
            new_nonzero_u32(2),
            new_nonzero_u32(2),
            new_nonzero_u32(2),
        ],
        names: PortNames {
            main_output: Some("Mix"),
            aux_outputs: &["303 1", "303 2", "808", "909", "Delay send", "Pre-master"],
            ..PortNames::const_default()
        },
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let frame = Arc::new(ui_host::FrameSize::new(
            ui_host::W.round() as u32,
            ui_host::H.round() as u32,
        ));
        Some(Box::new(PanelEditor {
            app: self.app.clone(),
            frame,
        }))
    }

    fn initialize(
        &mut self,
        _layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let n = buffer_config.max_buffer_size as usize;
        self.out = Out::new(buffer_config.sample_rate, n.max(1));
        self.scratch.resize(n.max(1), [0.0; CHANS]);
        self.load_song_env();
        true
    }

    fn reset(&mut self) {
        let engine = self.audio.current_mut();
        engine.halt();
        engine.locate(0);
        self.clock = Clock::new();
        self.keys = Keys::default();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();
        let host = Host {
            playing: transport.playing,
            tempo_bpm: transport.tempo,
            pos_beats: transport.pos_beats(),
        };
        {
            let engine = self.audio.current_mut();
            self.clock.apply(engine, host, true);
            while let Some(event) = context.next_event() {
                if let Some(ev) = midi_event(event) {
                    self.keys.apply(engine, ev);
                }
            }
        }

        let n = buffer.samples();
        if self.scratch.len() < n {
            self.scratch.resize(n, [0.0; CHANS]);
        }
        self.out
            .fill(&mut self.audio, &mut self.scratch[..n]);
        write_main(buffer, &self.scratch[..n]);
        for (port, bus) in aux.outputs.iter_mut().enumerate() {
            write_aux(bus, &self.scratch[..n], 2 + 2 * port);
        }
        ProcessStatus::KeepAlive
    }
}

struct PanelEditor {
    app: Arc<Mutex<App>>,
    frame: Arc<ui_host::FrameSize>,
}

impl Editor for PanelEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        Box::new(ui_host::open(&parent, self.app.clone(), self.frame.clone()))
    }

    fn size(&self) -> (u32, u32) {
        self.frame.get()
    }

    fn on_host_resize(&self, width: u32, height: u32) {
        self.frame.set(width, height);
    }

    fn min_size(&self) -> (u32, u32) {
        let w = (orb_ui::layout::PANEL_W / 2.0).round() as u32;
        let h = (orb_ui::layout::PANEL_H / 2.0 + orb_ui::menu::MENU_H).round() as u32;
        (w, h)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        false
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}

    fn param_values_changed(&self) {}
}

fn midi_event(event: NoteEvent<()>) -> Option<Event> {
    match event {
        NoteEvent::NoteOn {
            channel,
            note,
            velocity,
            ..
        } if velocity > 0.0 => Some(Event::NoteOn {
            channel,
            note,
            velocity: byte_from_unit(velocity),
        }),
        NoteEvent::NoteOn { channel, note, .. } | NoteEvent::NoteOff { channel, note, .. } => {
            Some(Event::NoteOff { channel, note })
        }
        NoteEvent::MidiCC {
            channel, cc, value, ..
        } => Some(Event::Cc {
            channel,
            cc,
            value: byte_from_unit(value),
        }),
        _ => None,
    }
}

fn write_main(buffer: &mut Buffer, frames: &[[f32; CHANS]]) {
    for (i, samples) in buffer.iter_samples().enumerate() {
        let frame = frames.get(i).copied().unwrap_or([0.0; CHANS]);
        for (ch, sample) in samples.into_iter().enumerate() {
            if ch < 2 {
                *sample = frame[ch];
            }
        }
    }
}

fn write_aux(buffer: &mut Buffer, frames: &[[f32; CHANS]], at: usize) {
    for (i, samples) in buffer.iter_samples().enumerate() {
        let frame = frames.get(i).copied().unwrap_or([0.0; CHANS]);
        for (ch, sample) in samples.into_iter().enumerate() {
            if let Some(v) = frame.get(at + ch) {
                *sample = *v;
            }
        }
    }
}

impl Vst3Plugin for OpenRb {
    const VST3_CLASS_ID: [u8; 16] = *b"OpenRB338VstInst";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

#[allow(semicolon_in_expressions_from_non_local_macros)]
mod vst3_export {
    use super::OpenRb;
    use nih_plug::prelude::*;

    nih_export_vst3!(OpenRb);
}
