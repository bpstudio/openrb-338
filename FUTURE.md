# Later work on the VST

`PLAN.md` is the instrument as it is. This file is the work after that. Nothing
here is a trace of ReBirth's MIDI, VST or ReWire. A fact about the original
still goes in `docs/` with an address. The map, the ports and the saved blob
are OpenRB's.

The standing rules stay in force. The mix path in `Frame::block` stays as it
is. A player's call stays one `Engine::render` of 1024 frames. The audio thread
does not lock `App`. Engine code stays f64, stores f32 where the original
stores a float, uses no FMA, and keeps the original's operand order. An engine
change is checked with `cargo test --release -p orb-engine --test corpus_golden`
and with `cargo test -p orb-player --test live_messages`.

## Order

1. The song inside the Live set.
2. Host parameters, and the panel following them.
3. A dry tap and a wet tap per device.
4. One MIDI input per device.
5. MIDI output of the pattern.

The song comes first because a set should reopen as it was left, and that work
does not touch the mix. Parameters come next so later MIDI and automation share
one byte path into the engine and one way back to the knobs. The taps are
copies beside that mix. The MIDI buses are a change to the vendored wrapper
(`third_party/nih-plug`), which today exposes one event input and, while
`MIDI_OUTPUT` is `None`, no event output.

## 1. The song in the plugin state

Live's project save asks the plugin for its state. Today that state is
`PluginState`: the parameter map and the `#[persist]` fields
(`third_party/nih-plug/src/wrapper/state.rs`). `PlugParams` is empty, so a
saved set reopens on the empty song. `ORB_VST_SONG` is read once in
`initialize` and is not part of the state.

File > Save already writes an RBS 4.2 song. `App::song_bytes` composes it with
`document::compose` and `rbs::Song::write`. That is the blob to store.

- Add a persisted field, stable name `rbs`, holding those bytes. A missing
  field is the empty song, so a set saved before this change still opens.
- On the host's save, take the bytes from `App` the same way File > Save does.
  That callback is not the audio thread. Do not compose from inside `process`.
- On load, open those bytes with the same path as a song dropped on the panel
  (`app.start` with the opened song). A parse error keeps the empty song and
  is printed. `reset` still does not reload the song.
- The kit stays the built-in one. The blob does not contain a mod, and the
  VST still does not load the song's mod.
- File > Save to an `.rbs` stays as it is. The set and the file are two copies
  of one compose.

Done when: save a set with steps and knob moves, quit Live, reopen the set,
and the panel and the mix match that song. A set saved before the field still
opens on the empty song. `Song::parse` of the stored bytes succeeds and
`Song::write` of that song is the stored bytes.

## 2. Host parameters and the panel

`PlugParams` is empty on purpose. An earlier set of host knobs wrote the
engine on every buffer and fought the panel. The replacement writes a byte
only when the rounded value changes, through the same setter the panel's
command uses, and the panel draws the byte the snapshot already publishes.

`Snapshot` (`crates/orb-player/src/lib.rs`) already carries `mixer` and every
device's `live` block. The panel's `Link::sync` already applies that to the
model. The host parameter is another writer of those bytes. While a panel
gesture owns a control, the gesture wins for that byte. Otherwise the host
value wins, and the knob, the switch or the slider moves.

Ids are stable strings. Values are the engine's bytes, `0..=127` except where
the setter's range is smaller. Step 1. Groups, using the controller indices
the engine already has:

| Group | Bytes |
|---|---|
| Mix | Master. Per channel, for 303 1, 303 2, 808, 909: enabled, level, pan, delay send, distortion on. Compressor routing and PCF routing. |
| 303, each | Tune, cutoff, resonance, env mod, decay, accent, waveform. |
| 808 | The level, tone, decay, snappy and select bytes through `CH_LEVEL`. |
| 909 | The level, tune, attack, decay, tone and snappy bytes through `FLAM`. |
| Delay | Enabled, steps, mode, feedback, pan. |
| Distortion | Enabled, amount, shape. |
| PCF | Enabled, frequency, resonance, amount, wave, decay, mode. |
| Compressor | Enabled, amount, threshold. |

Pattern steps, the selected pattern and song events are the song, not host
parameters. They travel in the saved blob of section 1. Meters are not
parameters.

`SAMPLE_ACCURATE_AUTOMATION` stays false. A parameter is read at the start of
the host buffer, then the engine quantizes to 64-frame blocks, as MIDI already
does.

The existing MIDI CCs (303 tune, cutoff, resonance, env mod, decay, waveform,
accent 77, and CC 7 master) write the same bytes. One function does both, so a
CC and a host parameter cannot drift.

Done when: drawing cutoff in Live moves the panel cutoff and the sound, and
turning the panel knob moves Live's value. An automated pass recalls. The
corpus and the live-session hashes still match.

## 3. Dry and wet outputs

The host already has Mix, four post-pan channel contributions, the delay's
dry send, and pre-master. Those channel taps are taken after the inserts, so
a channel with distortion or the PCF is already wet, and the delay's wet
return has no port. Summing the auxes does not rebuild Mix.

Keep every current port. Add taps beside the mix, the same way `BlockCapture`
already copies terms `Frame::block` has computed:

- Each of 303 1, 303 2, 808 and 909: the device bus before that channel's
  inserts (dry), and the post-pan contribution after the inserts (wet). The
  wet tap is the port that exists today. The dry tap is new.
- Delay: the send bus (dry, the port that exists today) and the stereo addition
  the delay writes into the master (wet, new).
- Distortion and the PCF sit on a channel. Their dry and wet taps are the
  channel bus immediately before and after that insert, for the channel that
  has the insert. A channel without the insert has silence on that pair.
- The compressor is mono on one channel or stereo on the master. Dry and wet
  follow that placement: the bus before the comp and the bus after it.

Names in Live say the device and dry or wet. Mix stays the int16 path divided
by 32768. A faulting block still writes no int16 and stores no taps. Off
44100 Hz the new ports take the same linear resample as the old ones.

Done when: a song with distortion, the PCF, the delay and the master
compressor shows a dry tap that misses that stage and a wet tap that has it,
and Mix is unchanged against the corpus and the live-session hashes.

## 4. One MIDI input per device

Today there is one event input (`MIDI_INPUT = Basic`). The channel picks the
device: 1 and 2 are the 303s, 3 and 10 the 808, 4 and 11 the 909
(`crates/orb-vst/src/midi.rs`). The wrapper reports one event input bus
(`get_bus_count` in `third_party/nih-plug`).

Per device means four event inputs Live can route separately: 303 1, 303 2,
808, 909. Bus 0 stays the current channel map, so a set that already sends on
channel 1 still plays 303 1. Buses 1..4 are one device each, and the channel
on those buses is ignored. The note map on each bus is the map that device
already has.

This is a change in the vendored wrapper's bus count and in the place
`process` reads events, plus the plugin declaring the bus names. It is still
OpenRB's map.

Done when: four MIDI inputs appear in Live, a clip on the 808 input triggers
the 808, a clip on channel 1 of the first input still triggers 303 1, and the
other inputs stay silent.

## 5. MIDI output

`Plugin::MIDI_OUTPUT` defaults to `None`, so the plugin has no event output
even though `ProcessContext::send_event` exists. Set `MIDI_OUTPUT` to `Basic`
and the wrapper reports one event output.

The output is the pattern, not a copy of the input. When a pattern step
triggers a 303 note or a drum hit, the plugin sends the note number that
device's input map would accept, on that device's channel (303 1 on 1, 303 2
on 2, 808 on 3, 909 on 4). A note that arrived from MIDI is not sent back.
Timing is the 64-frame block the engine already quantized to.

The trigger is observed beside the step. The step's own work, the rests and
the flam stay as they are.

Done when: a one-bar 808 pattern emits the drum notes on channel 3 and does
not emit a note that was only played in from MIDI. The corpus and the
live-session hashes still match.
