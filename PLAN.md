# VST3 instrument

OpenRB-338 as an instrument in Ableton Live: MIDI in, the host's clock, a stereo mix,
and separate outputs for the modules and the buses. The crate is `crates/orb-vst`.
The library name is `openrb338` (`libopenrb338.dylib`). ReWire stays out of scope.
The standalone app still has no MIDI.

This file is the specification of that instrument. A fact about the original program
still goes in `docs/` with an address. Nothing below is a trace of ReBirth's MIDI,
VST or ReWire. `sub_100261F0` is named in `docs/initial_investigation.md` and was
not read for this work.

## What it is

A VST3 instrument (`Instrument`, `Synth`), class id `OpenRB338VstInst`, vendor
`OpenRB-338`, bundle id `net.emphased.openrb338.vst3`. No audio input. One stereo
main output named `Mix`, plus six stereo auxiliary outputs.

The window is the desktop app's panel (`orb_ui::App`), parented into the host
view with baseview and egui 0.37. A winit event loop is not started: that
replaces the host's application delegate. The audio callback is the same
`orb_player::Audio` the panel drives, so the knobs, the pattern and Play are
what the host records. There is no second engine and no stand-in knob editor.

The engine still computes in f64, rounds to f32 where the original stores a
float, uses no FMA, and keeps the original's operand order. The mix path in
`Frame::block` is unchanged. The new stores are copies of terms that were
already computed. A player's call is still one `Engine::render` of 1024
frames. Splitting that call into sixteen `render(64)` changes the address the
delay leaves for the PCF, and the reference hash moves.

The plugin is built on nih-plug, which is not published. `third_party/nih-plug`
is upstream `de421011f41a6d10fc8c7a6084e4f4dee0143683` (ISC, `LICENSE` kept)
with host resize added: `IPlugView::can_resize` accepts a resize,
`check_size_constraint` clamps to the editor's minimum, and `on_size` stores
the logical size through `Editor::on_host_resize`. Default features off,
feature `vst3` only. Standalone (cpal) is not enabled. `nih_export_vst3!` is
called from a submodule so the `semicolon_in_expressions_from_non_local_macros`
warning that `vst3_com::vtable` emits can be allowed there. The parameter
struct is `PlugParams`: a struct named `Params` collides with the `Params`
trait.

## Outputs

Fourteen channels, `pull::CHANS`. The host sees them as the main port and the
aux ports.

| Port | Channels | What it is |
|---|---|---|
| Mix | 0–1 | The engine's int16 block, divided by 32768. Same scale as `orb-player`. |
| 303 1 | 2–3 | 303-1's post-pan contribution, `f32(bus * gain)`, before delay, master comp and master gain. |
| 303 2 | 4–5 | The same for 303-2. |
| 808 | 6–7 | The same for the 808. |
| 909 | 8–9 | The same for the 909. |
| Delay send | 10–11 | The mono send bus, copied to both sides, after every channel has added to it and before the delay reads it. |
| Pre-master | 12–13 | The master after the delay and the master compressor, before the master gain. |

The taps live on `Frame` as `BlockCapture` (`crates/orb-engine/src/frame/mod.rs`).
They are written beside the mix, not back into it. An early `return Err` from
`block` does not store the capture. The plugin treats a newly faulting block as
silence. Summing the aux ports does not reconstruct the mix: the module ports
are dry contributions, the delay's wet return is only in Mix and Pre-master, and
Pre-master is before the master gain.

At 44100 Hz the host gets those frames in order. The player renders them with
one `Engine::render(1024)`, the same call the app's callback uses, and
`Engine::render_captured` copies each finished block into a buffer the call
already owns. A fault still writes no int16, and the taps of that call stay
silent. `render(64)` leaves no leftover, so two calls match one `render(128)`.
The unit test `at_44100_the_mix_is_the_engine_blocks` checks that. Any other
host rate is linear interpolation between those frames and is not the engine's
sample values. `|rate - 44100| < 0.5` counts as 44100.

Events are applied at the start of the host buffer, then the audio is quantized
to 64-frame engine blocks. That is not sample-accurate MIDI.
`SAMPLE_ACCURATE_AUTOMATION` is false. `process` does not allocate unless the
host asks for more frames than `max_buffer_size`.

## MIDI

Channels are 0-based, as the plugin API numbers them.

| Channel | MIDI channel | Plays |
|---|---|---|
| 0 | 1 | TB-303 #1 |
| 1 | 2 | TB-303 #2 |
| 2 and 9 | 3 and 10 | TR-808 |
| 3 and 10 | 4 and 11 | TR-909 |

A 303 note uses MIDI 36 as the pattern's low C (pattern note 0). The semitone
offset is clamped so `pitch_target >> 4` stays inside the 6144-entry pitch
table (`live_note_stays_inside_the_pitch_table`). Velocity 100 or above sets
the accent flag. A second note while one is held sets slide. Note-off of the
held note calls `Tb303::note_off`. A note-on with velocity 0 is a note-off.

Drum notes are a GM-ish map in `midi.rs` (`tr808_column`, `tr909_column`).
`hit` uses columns 1..=11 and the step's accent formula. The 909 also sets
gain 0.8 below velocity 100 and 2.0 at or above it.

CCs on a 303 channel: 2 tune, 3 cutoff, 4 resonance, 5 env mod, 6 decay,
8 waveform (non-zero is square), 77 accent. CC 7 on every channel is the
master level, which is why accent is 77 and not 7. Other channels ignore
every CC except 7.

## Clock

The host transport, not MIDI clock. `Clock::apply` in `clock.rs`.

- Tempo is BPM×1000, rounded, clamped to 1..=500000. A missing or non-finite
  tempo is left as the engine has it (a non-finite value passed in becomes 1).
- Position is quarter notes × 192, truncated toward zero, clamped to
  0..=767231 (the song-end tick).
- When the host starts, the engine locates to that tick and then `play`s.
- While playing, a gap of more than 48 ticks (one 16th) relocates. Smaller
  drift is left alone so a locate does not retrigger every buffer.
- When the host stops, the engine `halt`s and then locates to the host tick.
  `halt` does not jump to the bar start; `stop` would.
- The plugin always follows. `Clock::apply` still has a `follow` argument,
  used by its tests; `process` passes true.

Pattern mode is the engine's default. While the host is playing, the pattern
steps the devices. A rest step still schedules a 303 note-off about 24 ticks
later, so a live 303 note is cut. Empty drum patterns do not trigger the
drums, and `hit` is independent of the step, so drum notes still sound.
The panel's own Play starts the engine even while Live is stopped. The
panel's Stop while Live is playing stays until Live stops and starts again,
because the clock only plays on the rising edge.

## Window

`Editor::spawn` opens the app panel in the host's view (`ui_host`). The OpenGL
context is made current before glow loads: glow reads `GL_VERSION` at that
moment, and a panic there crosses `IPlugView::attached` and aborts the host.
The logical size starts at the panel, 624×510, plus the menu bar, 22:
624×532. The host may resize it. The minimum is half the panel, the same
floor as the app (`PANEL_W / 2` by `PANEL_H / 2 + MENU_H`). The child view is
resized to that logical size as soon as the editor is attached, and again
whenever the host reports a size. `App::show` fits the 624×510 panel
into the area under the menu, one scale for both axes, letterboxed, and the
SVGs rasterize again at the new pixel size. The scale comes from the view.
The editor holds `Arc<Mutex<App>>`. The audio thread
never locks it. Commands go through the player's queue, and the snapshot
comes back, as in the app. Meters move because the view paints continuously;
there is no cpal stream. `ORB_VST_SONG` is opened in `initialize` through
`orb_ui::session::open_song`.

`PlugParams` is empty. The host has no stand-in knobs. Cutoff, resonance,
vintage and the master are the panel's controls, at the engine's constructor
values until the panel or a CC moves them. `reset` halts, locates to tick 0,
and clears the clock and the held notes. It does not reload the song.

## Song

`ORB_VST_SONG`, if set, is read once in `initialize`. Parse errors are printed
and the plugin keeps the empty engine. After a successful load the engine is
halted. The VST always uses the built-in kit (`rbm::standard::builtin_samples`).
It does not load the song's mod. The variable is not a test input.

## Build and install (macOS)

```sh
cargo build --release -p orb-vst && scripts/macos-vst3.sh
```

The script copies `target/release/libopenrb338.dylib` to
`target/OpenRB-338.vst3/Contents/MacOS/OpenRB-338`, writes the bundle plist
(`CFBundlePackageType` `BNDL`, minimum macOS 11.0), ad-hoc signs it, and
installs a copy at `~/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3`. Rescan
plug-ins in Live. In Live's I/O, the main output is Mix; the aux ports are
the separate outputs. Set the track to 44100 Hz when the render must match
the engine's samples.

## Checks

- `cargo test -p orb-vst --lib`: clock (tempo, play/locate, stop does not jump
  to the bar, follow-off halts), drum notes, unit-to-byte, and the 44100 mix
  against `render`.
- `cargo test -p orb-player --test live_messages`: the session hashes, including
  `pcf_junk_wave_on_the_303_channels`, match rbhost. That is the check that the
  1024-frame call was not split.
- `cargo test -p orb-engine --lib live_note_stays_inside_the_pitch_table`.
- `cargo clippy -p orb-vst -p orb-engine --all-targets -- -D warnings`.
- `cargo test --release -p orb-engine --test corpus_golden` (2026-11-10): diag,
  app at the fitted N, and shipped all matched. The taps do not move the mix.

## Limits

- Not ReBirth's MIDI map, and not sample-accurate.
- Pattern rests cut live 303 notes while the host is playing.
- Built-in kit only. No mod folder.
- Off 44100 Hz the audio is a linear resample.
- The delay return is not its own port. "Delay send" is the dry send bus.
- Aux channels do not sum to the mix.
- CC 7 is master, on every channel. The 303 accent CC is 77.
- One instrument, Apple Silicon or whatever target the cdylib was built for.
  The bundle script is macOS only.
