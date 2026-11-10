# CLAUDE.md: collected knowledge of OpenRB-338

This file is the briefing Claude loads. It collects how the repository is worked and what is
known about ReBirth RB-338 2.01 and about orb. The line-level traces (every address, every byte
layout, every measurement) stay in `docs/`. Read the section this file names when a task needs
that detail. Do not treat a `[G]` item as confirmed.

A new fact about the original goes into its home document under `docs/`, with addresses, in the
same commit as the work (`AGENTS.md`). Update the matching part of this file in that same commit
so the briefing stays true. A new defect also goes into `docs/original_bugs.md`.

Marks, everywhere: **[C]** was checked in the disassembly (or on a real file, where the home
document says so). **[G]** is guessed or only partly traced. Addresses: `0x10xxxxxx` is
`ReBirth.dll` (image base `0x10000000`), `0x4xxxxx` is `Rebirth.exe`. Function names in the docs
are descriptive; the IDA databases have not been renamed.

`docs/RBS42.txt` is Propellerhead's RBS 4.2 specification. It may be distributed only unchanged.
Do not edit it. Its errors are in `docs/song_load.md` §10.

## 1. What this project is

OpenRB-338 (command `orb`, crates `orb-*`, environment `ORB_*`, repository and checkout folder
`openrb-338`) is a Rust reimplementation of the audio engine of Propellerhead ReBirth RB-338
2.01. It renders an `.rbs` song, optionally with an `.rbm` mod, to audio. The rendered audio
must match the original's "Export Loop as Audio File" sample for sample, within a tolerance a
`Decide:` task sets. The UI (egui, scalable SVG panel) follows the engine and does not have to
match the original exactly. ReWire is out of scope. The standalone app has no MIDI. The VST3
instrument is `crates/orb-vst`, specified in `PLAN.md`.

The original program (`Rebirth.exe`, `ReBirth.dll`) is copyrighted and is not in git. It lives
on the user's machine and on the Windows test machine. The user's IDA databases
(`Rebirth.exe.i64`, `ReBirth.dll.i64`) are not in the repository. Agents open a copy through the
idalib MCP tools. IDA needs no exe or dll next to the `.i64`. No test or tool may need the
original's exe or dll: tests take those files from the CD image (`ORB_CD_ISO`, crate
`orb-testkit`) and skip with a message when it is unset.

License: GPL-3.0-only. The built-in kit `assets/Standard.rbm` is Propellerhead's drum samples
and panel pictures, committed by decision of 2026-10-27 as the one exception
(`assets/README.md`, `docs/rbm.md` §13). UI art under `assets/ui/` is original work, generated
from `crates/orb-ui-art`; do not edit an SVG by hand.

Shipped behavior the README states, and that the engine is held to:

- A song's exported loop equals the original's export. Checked on 1,268 songs of the song
  collection, 531 of them on 43 third-party mods, and on a 175-render reference corpus.
- Opens ReBirth 2.x songs (RBS 4.0 to 4.2) and ReBirth 1.x songs, and saves RBS 4.2 the original
  opens.
- Plays recrypted `.rbm` mods. The standard kit is embedded, so no file of the original is
  required at build or run time.
- Export of the loop to WAV, FLAC or Ogg Vorbis. AIFF export is not implemented.
- Not implemented in the standalone app: MIDI, ReWire, mod themes (a mod changes drum sounds,
  not the panel look), the original's Edit-menu pattern operations and undo, the original's
  preferences and Windows menu. The VST3 instrument (`crates/orb-vst`, `PLAN.md`) has MIDI
  input, host-clock sync and a stereo mix plus separate outputs. Its MIDI map is OpenRB's, not
  a trace of the original.

### Milestones

- `m-0`: deterministic reference renders from the original, and the tools around them.
- `m-1`: the Rust engine and the `orb` CLI (`.rbs` to FLAC, Ogg or WAV).
- `m-2`: RBM mods.
- `m-3`: the desktop UI: play and edit songs live.

Planning, progress and decisions live in `backlog/` and are changed only through Backlog.md
(MCP, or the `backlog` CLI). A milestone title starts with its id (`m-1: ...`).

### Crates

Workspace root `Cargo.toml`. Toolchain pinned in `rust-toolchain.toml` (1.99.1). Edition 2024.

| Crate | Role |
|---|---|
| `rbs` | RBS reader and writer, including ReBirth 1.x import (`src/rb1x.rs`) |
| `rbm` | RBM container, descramble, recrypt, the embedded 2047-byte key window, `Standard.rbm` |
| `orb-engine` | DSP, sequencer, song load, export framing |
| `orb-cli` | Binary `orb`: `render`, `play`, `info`, `compare` (hidden), `mod` |
| `orb-player` | Playback without the UI |
| `orb-ui` | Binary `OpenRB-338` (egui / eframe / wgpu) |
| `orb-ui-art` | SVG panel generator (`ui-art`) |
| `orb-vst` | VST3 instrument (`libopenrb338`, `PLAN.md`) |
| `orb-testkit` | Extracts pinned files from `ORB_CD_ISO` once per test process |

Engine modules under `crates/orb-engine/src/`: `frame/` (mixer block and `export.rs`), `seq.rs`,
`load.rs` (and `load/rb1x.rs`), `tb303.rs`, `tr808.rs`, `tr909.rs`, `drums.rs`, `dist.rs`,
`pcf.rs`, `comp.rs`, `delay.rs`, `samples/`, `tables/`, `song.rs`, `process.rs`.

## 2. How to work

Read, for a backlog task: this file, then `AGENTS.md` if a rule here and there ever disagree
(they should not), then `backlog/docs/doc-001 - agent-brief.md` before IDA, the test machine, or
a build in an agent worktree, then the task (`backlog task view task-N --plain`), then only the
`docs/` sections the task names. Of `docs/initial_investigation.md`, grep the heading and read
that range. Do not re-read the brief.

### Who does what

- `@pingw33n` is the user and owns every decision. Never change, reassign or spawn a task
  assigned to `@pingw33n` unless the user asks.
- `@supervisor` orchestrates. It assigns freely among unassigned tasks and tasks assigned to an
  agent type. It does not self-review.
- `@agent-complex`: reverse engineering, DSP, sequencer, numerics, the offline DLL host, and
  experiments that need judgment. Opus.
- `@agent-worker`: bounded work with a clear spec (Python tools, file-format crates, CLI, sample
  loading, test songs, corpus capture, docs). Sonnet. Prefer not to change engine DSP; that
  becomes a follow-up for `@agent-complex`.
- `@agent-chore`: mechanical work whose text or script is already written. Haiku. Stop instead
  of guessing if the task needs an engine change, a disassembly reading, or an unsettled
  judgment.
- `agent-reviewer`: read-only review of a branch before merge. Haiku. Never an assignee. Verdict
  is `merge` or `send back`, each defect with `file:line`.

Definitions: `.claude/agents/`.

### Backlog

Changed only through Backlog.md's MCP tools or the `backlog` CLI. Never edit a file under
`backlog/` by hand. A multi-line text goes in with a real newline, never a literal `\n`.
`doc view` truncates long documents; read a long doc from its file under `backlog/docs/`.
In an agent worktree use the CLI, because the MCP tools write to the main checkout.

- A decision is a task labeled `decision`, titled `Decide: <title>`, `--type spike -l
  decision,<area> -a @pingw33n`, with context, options and a recommendation. The task that
  needs it depends on it. Do not make the decision yourself. When the user decides, the Decide
  task gets a final summary with the outcome and the date, and `-s Done`.
- `In Progress` is never committed. A task goes from `To Do` to `Done` in its one commit.
- Only tasks in a milestone count for "what next": lowest open milestone, then priority and
  ordinal. A task in no milestone is tracking or an idea.
- Labels: `re`, `engine`, `cli`, `tools`, `tests`, `testmachine`, `rbm`, `ui`, `docs`,
  `decision`. Types: `bug`, `feature`, `enhancement`, `task`, `chore`, `docs`, `spike`.
- A task must be doable without the conversation that created it: why, context not in the code,
  testable acceptance criteria, `--ref`, labels, type, priority, milestone, dependencies.
  `--ordinal` is the queue order. `Draft` holds ideas.
- While a task is assigned to an agent, only that agent edits its file.
- Commands: `backlog task list --ready --sort ordinal --plain`; `backlog task list -l decision
  -a @pingw33n --plain`; `backlog milestone list --plain`. Read `backlog instructions
  task-execution` before starting and `backlog instructions task-finalization` before finishing.

Starting and finishing, for an agent on a task:

- Spawned on the task as it is (`To Do`). No status change and no commit on main. Work in its
  own git worktree. Do not touch the user's checkout.
- Add the branch name with `--append-notes`, then `--plan` before changing code.
- Notes as the work goes: evidence (addresses, functions, scripts), numbers before and after,
  failed hypotheses, commands that reproduce a result.
- Verify each acceptance criterion and `--check-ac`. `--final-summary` with outcome, branch,
  commits, numbers and tests. Then `-s Done`, in the same commit as the work.
- One commit per task. Review fixes amend it. It lands on main as that one commit (`git merge
  --squash`, or a fast-forward of the rebased commit). No separate start, merge or comment
  commits. No Review status.
- The final chat message is the task id plus what the supervisor must do. Nothing is reported
  only in chat.
- On `send back`, the same task-agent session is resumed. On re-review, the same reviewer
  session is resumed. Only the first review spawns the reviewer fresh.
- Follow-up work becomes a new task with labels, milestone, `--dep` and a description a future
  agent can act on alone.
- Do not expand the task. A design decision the task does not take becomes a `Decide:` task,
  and you stop there.

Rebase onto main once, after tests pass and before the final commit. Rebase onto nothing but
main. Conflicts: a file the task did not change takes main's version; a file you changed is
merged hunk by hunk, both sides kept; a backlog file is never resolved by hand (take main's,
then redo the task's changes with the CLI). Mandatory check: `git diff main --stat` lists only
files the task changed. Reset anything else to main and amend. Say in the report that this
check ran.

Agents never push. The supervisor pushes main to `origin` (github.com/pingw33n/openrb-338,
private) after each merge and runs `backlog doctor`.

### Commit messages

Subject: `<area>: <what>`, at most 72 characters. Area is one of `engine`, `cli`, `tools`,
`tests`, `docs`, `backlog`.

Body, in this order:

1. **Change**: what and where.
2. **Result**: numbers before and after, what works now, and the corpus result for an engine
   change.
3. **Testing**: `cargo test N pass`, the corpus comparison, the tool run, or none.
4. `Task: task-NNN`, plus `Decision: task-NNN` when a decision was applied.
5. The attribution line the harness gives.

The reviewer's Co-Authored-By line names the task agent's model, not the reviewer's: Opus for
`@agent-complex`, Sonnet for `@agent-worker`, Haiku for `@agent-chore`. Flag it only if it does
not match the assignee.

### Standing rules

- Never save or modify the user's IDA databases. Never commit an IDA database or its working
  files (`*.id0` and the rest). Never edit a committed reference render by hand. A reference
  changes only through the capture script named in the corpus manifest.
- Facts about the original carry addresses and are checked against the disassembly, not only
  Hex-Rays. Hex-Rays is a second opinion. Check exact x87 operand order, every f32 store
  (`fstp dword` rounds) and integer conversions against the disassembly. Decode constants from
  the bytes, not from IDA's float rendering. `.data` above `0x10034400` in the DLL is zero in
  the file: those tables are computed at run time.
- Engine code computes in f64, rounds to f32 exactly where the original stores a float, uses no
  FMA (`mul_add` is forbidden), and keeps the original's operand order. Float-to-int truncates
  toward zero.
- An engine change is checked against the reference corpus. State the result in the commit:
  exact, within the tolerance, or the first differing sample.
- Edit source with the editor's edit tools, not Python or sed.
- Spell "color" the American way.
- Rust lines are at most 100 columns, enforced by nightly rustfmt (`rustfmt.toml`). Long string
  literals are split by hand (`\` continuation or `concat!`); rustfmt does not wrap them.
- The shell in an agent worktree is fish: put multi-step sequences in a bash script in the
  scratchpad. Use `/usr/bin/python3` for the Python tools. Builds and tests run at the tools'
  default parallelism: no `-j`.
- Temporary files go in a subdirectory of the session scratchpad named after the branch. Never
  run a script you did not write there.
- Verify a claim from written evidence against the binary before relying on it.
  `docs/initial_investigation.md` was written from a survey and marks guesses `[G]`. List what
  you could not confirm.
- orb reproduces the original's behavior, bugs included (`docs/original_bugs.md`), unless a
  task records a deliberate deviation.

### IDA

Use the idalib MCP tools only. Never drive IDA from the shell, and never use the user's GUI
session. The databases are not in the worktree. The supervisor gives their path. Without a
path, ask the supervisor and do not search the disk.

1. Copy the database into the scratchpad subdirectory.
2. `idb_open(input_path=<copy>, mode="force_headless")`.
3. `idb_close(save=false)` when done.

Do not rename, retype, comment or patch. Facts go into `docs/` and the task notes, with
addresses.

### The Windows test machine

The original runs on a Windows machine, reached with `~/dev/dcre/common/tools/rcs_client.py`
(`rcs`). Commands: `info`, `ls`, `get`, `put`, `rm`, `exec`, `ps`, `kill`, `input`,
`screenshot`. Config: `~/dev/dcre/common/tools/rcs_client.toml`. Remote paths use `/`. An
absolute path names any file; a relative one starts at `C:\re`.

ReBirth 2.01 is installed in `C:\ReBirth RB-338`, outside the rcs root. Never write into the
installation. Copies go to `C:\re\rb`. The full layout, menus, export steps and preferences are
`docs/test_machine.md`. Read it before using the machine.

Permits: the machine is shared through `locky` (semaphore `redc-win-run`). Read the permit
count from `locky list`; never assume it. A ReBirth session takes the keyboard and the screen,
so hold **all** the permits while one runs:

- `locky acquire redc-win-run N --owner YOURBRANCH --acquire-timeout 3600 --lease-timeout 1800`
- `locky refresh redc-win-run --owner YOURBRANCH --lease-timeout 1800`
- `locky release redc-win-run all --owner YOURBRANCH`

Never `locky release-all`. Never stop a process you did not start. Runner tools take and
release the permits themselves.

Start: `rcs exec cmd.exe /c start "" /D "C:\ReBirth RB-338" "C:\ReBirth RB-338\Rebirth.exe"`,
then `tasklist` for the PID, stop with `taskkill /PID <pid>` (no `/F` closed cleanly). At start
a modal "Failed opening MIDI input" (title `ReBirth RB-338`) appears. No preference prevents
it. Dismiss with `rcs input --window "ReBirth RB-338" key:enter`. Main frame title is
`ReBirth RB-338`; the song is an MDI child. Preferences file:
`C:\Users\pingw33n\AppData\Local\VirtualStore\Windows\ReBirth RB-338 2.prf`.

### Testing

- Rust: `cargo test`, every test passing. A debug `cargo test` skips the shipped songs (about
  24 s in debug). Golden: `cargo test --release -p orb-engine --test corpus_golden`.
- Python: `/usr/bin/python3 -W error::ResourceWarning -m unittest discover -s tools -p
  '*_test.py'` and the same with `-s tools/rbhost`.
- Corpus: `cargo run --release -p orb-cli -- compare --manifest tests/refs/manifest.json
  [--mod-dir DIR] [--set diag,app,shipped] [NAME...]`. Prints `EQUAL` or the first difference
  and exits 1 on any difference. One song: `orb compare SONG.rbs --reference REF.flac|REF.wav
  [--idle-blocks N]`.
- A tool change is tested by running it on at least one song, with what ran in the notes.
- A docs-only change needs nothing.
- Tests that run `tools/*.py` find Python 3 through `tests/common/python.rs`: `ORB_PYTHON` if
  set, else `python3`, `python` or `py -3`. None of those is a failure naming `ORB_PYTHON`, not
  a skip.
- Lint: `cargo clippy --all-targets -- -D warnings`.
- Format: `cargo +nightly fmt --all`. Check: `cargo +nightly fmt --all --check`. CI pins
  `nightly-2026-10-18`. Only formatting uses nightly. Build and clippy use the pinned stable
  toolchain.

## 3. The original program

| Module | Role |
|---|---|
| `Rebirth.exe` | UI, song load and save, mod open and parse, the export driver and WAV/AIFF writers, realtime audio (DirectSound and MME). Imports only KERNEL32; the rest is loaded dynamically. Talks to the engine through a shared-memory port ("ReBirth Panel->DLL" / "DLL->Panel"). |
| `ReBirth.dll` | All DSP and sequencing. About 187 KB of code, 945 functions, full RTTI. Exports `EngineOpen/Close/Idle`, `SafeSoundCallback`, `SyncSoundCallback`, MIDI sync and the TopHat (ReWire precursor) procs. `.rsrc` holds the 46 built-in drum AIFFs (resources 1000..1045). |

RTTI classes: `CMixerDevice`, `CTB303Device`, `CTR808Device`, `CTR909Device`, `CDistInsert`,
`CPCFInsert`, `CCompInsert`, `CDelayEffect`, voices `SIMPLE_VOICE`, `FILTER_VOICE`,
`DUAL_SAMPLE_VOICE`, `PITCHED_VOICE`, `CWaveSample`, and the IFF/AIFF readers and writers.

Device vtables in the DLL (slot = index). Audio is slot 14 unless noted:

| Class | vtable | Audio and other slots |
|---|---|---|
| `CTB303Device` | `0x10033FD4` | audio `0x10023F10` (Vintage inline; zero sound-model byte tail-calls 2.0 sound `0x100248C4`); step `0x100253A0`; reset `0x10026CC0`; host msg `0x100256E0`; setParam `0x10026180` |
| `CTR808Device` | `0x10034154` | audio `0x100280B0`; step `0x10027690`; setParam `0x10029050` |
| `CTR909Device` | `0x10034294` | audio `0x1002B6F0`; step `0x1002AAC0`; setParam `0x1002C6D0` |
| `CDistInsert` | `0x10033144` | process `0x10010A50` (slot 15); update `0x10010894` |
| `CPCFInsert` | `0x10033AD4` | process `0x100166D0` (slot 15) |
| `CCompInsert` | `0x10032B24` | stereo `0x1000EDE0` (slot 14); mono `0x1000EC70` (slot 15); update `0x1000EB44` |
| `CDelayEffect` | `0x10032CB4` | process `0x1000F940`; update `0x1000F754` |
| `CMixerDevice` | `0x10033214` | mixer block `0x10013444`; 16-block update `0x10012C14` |

The engine is entirely float32/x87 and block based. No hidden randomness, no oversampling, no
dither, no hardware dependence. Algorithms are recurrences, lookup tables and linear
interpolation. A byte-exact DSP port is feasible. The hard part is the state at export start:
the export does not reset the engine (`docs/export_state.md`).

### Numerics [C]

The EXE CRT sets 53-bit precision. The DLL never changes it. All 94 `fldcw` sites are a
temporary switch to truncation for `fistp`. So x87 arithmetic behaves like IEEE f64, apart from
the extended exponent range, which should not matter. Transcendentals and the CRT `pow`
(`0x1002DDF0`) run at x87 extended precision and are used only to build tables at init; every
table is stored as f32. An f64 `pow` (the `libm` crate) already matches the stored tables
(task-019). Nothing is embedded. Tables live in BSS (zero in the file) and are listed in
`docs/initial_investigation.md` §8.

Rust rule, applied everywhere in `orb-engine`: compute in f64; store f32 exactly where the
original stores; no FMA; keep operand order; truncating float-to-int.

Output conversion in `SafeSoundCallback`, per sample, L then R, no dither [C]:

`i16 = trunc_toward_zero(clamp((double)f32 * 32768.0, -32768.0, 32767.0))`

Scale is the double 32768.0. Clamp constants are f32 32767.0 and -32768.0. The upper bound is
tested first, so a NaN becomes -32768. `n` is clamped to 0x4000 first. Only output channels 0
and 1 are enabled.

### The 64-frame mixer block (`0x10013444`) [C]

Once per 64 frames. Rust: `crates/orb-engine/src/frame`.

```
every block:
  if (blk & 15) == 0: mixer_update()     # 0x10012C14; blk starts at 10 in the ctor, never reset
  blk++
  sequencer_tick()                       # 0x1001EF20, advances position BEFORE rendering
  masterL = masterR = 1e-6               # denormal guard, also on send buses
  for ch in [303-1, 303-2, 808, 909]:
      dev.process(bus)                   # mono f32[64], into a shared scratch
      for ins in [dist, pcf, comp] if routed: ins.process(bus)
      send[i] += bus[i] * sendGain
      smooth pan gains: n = old*0.93 + tgt*0.07, store f32
      ramp old -> n over the block, step every 2 samples, step = (n_unrounded - old_f32) * (1/32)
      masterL += bus*gL; masterR += bus*gR
  delay.process(send, masterL, masterR)
  if comp routed to master: comp.process_stereo
  m = mOld*0.93 + mTgt*0.07; store f32(m); master *= unrounded m, constant over the block
```

- Channel outputs in the app are null, so each device renders into the shared scratch
  `unk_1010012C`, then the channel is mixed to the master.
- The mixer never refills the send bus. The delay does, with 1e-6, enabled or not.
- The song has three 256-byte send buses. Only the delay is registered, so buses 1 and 2 have
  no reader.
- Insert order is Dist, then PCF, then Comp. Dist is per channel with one shared parameter
  set. PCF sits on one channel. Comp sits on one channel (mono) or the master (stereo). Delay
  is the only send effect.
- Mixer, dist, comp and delay targets recompute every 16 blocks (1024 samples), then smooth.
  PCF, 303 and drum parameters are read per block or per trigger.
- Level curve `T[128]`: `T[127] = 1`; for i = 126 down to 1, `T[i] = f32(T[i+1] - f32(1/24)*T[i+1])`;
  `T[0] = 0`. Used for every level, send and amount. Step `f32(1/24)` is `0x3D2AAAAB`.
- Pan law `PAN[a][b]`, `d = b-64`, `q = (d*d)>>5`, is asymmetric: b = 0 gives R = 0, b = 127
  leaves L = T[a-124].
- Sound-model byte `song+0x16D` (GLOB[482], 0 before file version 4.2): **0 is ReBirth 2.0
  sound** (303 audio tail-calls `0x100248C4`, plain send `T[lev]*T[send] + 1e-6`). **1 is
  Vintage** (inline engine in `0x10023F10`, pan-dependent send). The early survey had these
  labels the wrong way round; `docs/song_load.md` §3 and `docs/initial_investigation.md` §6 are
  the correction. Every shipped song is 0.
- Master target: `2*T[master] + 1e-6`. Initial channel gains 1e-6; master current and target
  1.0. Constructor mixer values (master 100, comp and PCF off, channels enabled, level 100,
  pan 64, send 0, no dist) are what the first blocks smooth toward, because msg 41 does not run
  an update.

Full per-sample formulas for the 303 (both engines), drum voices, distortion, PCF, compressor
and delay are `docs/initial_investigation.md` §6 to §8. Do not re-derive them from this
briefing.

### Sequencer (`0x1001EF20`) [C]

Rust: `crates/orb-engine/src/seq.rs`. The sequencer object is at `song+8`. Transport globals are
at `song+0x158`.

- Position is a double. It advances by `tempo_mBPM * 4.6439909297052154e-06` per block
  (`tempo_mBPM` is BPM×1000). That is 192 ticks per quarter note. A 16th step is 48 ticks. A
  bar is 768 ticks. `tick = trunc(pos)`. Work happens only when `tick > lastTick`. Position
  advances **before** the block renders.
- Song events apply in the first block whose `trunc(pos) >= eventPos`.
- At `pos > 767231` the song stops (999 bars, strictly greater). The DLL drops a track event
  whose quantized tick is above 767231. The exe accepts up to 767976 (OB-3).
- Shuffle: `sp = 96*(t/96) + max(0, t%96 - shuffle/6)`. Only the second 16th of each 8th is
  delayed, by up to 21 ticks. Devices use the shuffled tick when the pattern's shuffle flag is
  set.
- Every note trigger, event and PCF step is quantized to a 64-frame block boundary. A note that
  starts in block k renders from sample 0 of that block.
- Pattern changes and per-device enable take effect only on absolute bar lines
  (`(tick/48)&15 == 0`).
- `patStart` resets only at a pattern change, at start and at relocate. A pattern shorter than
  16 steps keeps its own length; it is not realigned every bar.
- Pattern mode applies no event, never wraps and never ends. Devices still step. Loop wrap and
  song end are tested only in blocks whose tick moved.
- An event whose tick equals the loop end never fires in a loop (OB-4): the wrap applies events
  up to `loopEnd - 1`, and the chase starts the next pass at `loopStart`.
- Set-param calls of a block (tracks 0..8) come before every step call. Device order: mixer,
  303, 303, 808, 909, delay, dist, PCF, comp. Only the 303s, the 808 and the 909 implement step,
  relocate and note-off.
- 909 flam [C]: on a step change, `flamTick = tick + (tempo*flam)/700000 + 1` in 32-bit
  arithmetic, unshuffled tick. When `tick > flamTick`, every column whose step value is above 2
  retriggers at `f32(accent*1.5)`.

Tracks are circular singly linked lists of `{next, tick, ctrl, value}` from one process-wide
pool of 100000 nodes of 12 bytes (`dword_10100230`). An insert rounds the tick up to a multiple
of `24q` (q = 1 for loaded events, except 303 controllers 0 and 1 which are 32), drops a tick
above 767231, and replaces a node of the same tick and controller. Events of one tick apply in
reverse arrival order. One controller has at most one event per tick. Msg 37 refuses a whole
batch whose count exceeds the free count and replies `['K', 22]`; the exe then abandons the
load and never closes the engine's song (OB-20). orb uses one pool of 100000 nodes per song.

Stop prunes events whose value equals the previous value of that controller, unless the tick is
0. Locate chases in song mode, sets `pos` and `lastTick`, and relocates devices. Nothing
triggers until `trunc(pos)` reaches P+1. Stop locates to the bar start unless MIDI sync is
active. Play does not move the position.

Host messages and the recording path: `docs/initial_investigation.md` §5.2 and §5.3,
`docs/song_load.md` §1.

## 4. Songs: file, load, save

An `.rbs` is an IFF `CAT ` of type `RB40`. The original writes, in this order: `HEAD` (256
bytes), `GLOB` (512), `USRI` (712 in real files; RBS42.txt wrongly says 512), `CAT DEVL`
(device chunks) and `CAT TRKL` (nine `TRAK` chunks). The reader does not require that order
inside `RB40`. Device and track order is fixed and is the device id:

| id | Device | DEVL chunk | TRAK |
|---|---|---|---|
| 0 | Mixer | `MIXR` | Mixer |
| 1 | TB-303 #1 | `303 ` | 303 1 |
| 2 | TB-303 #2 | `303 ` | 303 2 |
| 3 | TR-808 | `808 ` | 808 |
| 4 | TR-909 | `909 ` | 909 |
| 5 | Delay | `DELY` | Delay |
| 6 | Distortion | `DIST` | Dist |
| 7 | PCF | `PCF ` | PCF |
| 8 | Compressor | `COMP` | Comp |

Multi-byte values in engine messages are big-endian. File chunk sizes are big-endian, as IFF.
Before anything else is read, the file must be exactly one FORM, CAT or LIST chunk whose stored
length matches the file (task-129). Bytes after the chunk, or a chunk that claims more than the
file holds, are refused. `crates/rbs` refuses the same way.

HEAD bytes 5..7 are major, minor, revision. HEAD[4] must be 0xBC. HEAD starts with `[T[T` and
is 256 bytes. Major must be 4. Version comparisons use the key `major<<16 | minor<<8 |
revision`, and the 4.0 and 4.1 special cases are equality with revision 0. A 4.1.1 file gets
neither the 4.1 enable-forcing nor any 4.2 path. Every shipped and diagnostic song has
revision 0.

Version-dependent paths [C]:

| Where | 4.0.0 | 4.1.0 | >= 4.2.0 |
|---|---|---|---|
| MIXR | 4×7-byte channels, converted | 4×5-byte; all four enabled forced to 1 | 4×5-byte as stored |
| GLOB vintage byte | 0 | 0 | GLOB[482] |
| 808 tom tunings 10, 13, 16 | `v/2 + 32` before the range check (OB-2) | same | as stored |
| Mixer track | old converter `sub_437380` | normal | normal |
| 808 track | tom controllers get `v/2 + 32` | same | as stored |

Errata against RBS42.txt (full table in `docs/song_load.md` §10): TRAK positions are 32nd
notes (engine tick = position × 24). Mixer-track 909 delay and dist are controllers 0x1A and
0x1B, not 0x20 and 0x21. PCF device id is 0x01..0x04. 303 flag bit 4 set means rest. Delay step
mode 0 is triplets (1/3 beat), 1 is sixteenths. Selected pattern is 0..31. 303 waveform 0 is
saw, not triangle. USRI is 712 bytes: title at 0 (41), text at 41 (401, not 201), web at 442
(101), show-on-open at 543; bytes 544..711 are leftover junk the engine ignores. The exe drops
TRAK positions at or above 32000 (tick 768000). A delta position has no byte limit in the
reader; a 5-byte delta exists in a third-party song and is dropped when the tick check fails.

### Messages at song load, in order [C]

Rust: `crates/orb-engine/src/load.rs`. The Python twin is `tools/rbhost/rbsmsg.py`.
`crates/orb-engine/tests/rbsmsg_corpus.rs` checks they agree. Top-level packets are
`[type, payload…]`. Song packets are `[78, song, msg, payload…]`. Device data is
`[78, song, 41, dev, sub, payload…]`. `song` is the slot from msg 52, not a device.

1. Peek HEAD and GLOB. No message.
2. Look up the mod named in GLOB. The engine already knows it (top-level 56 or 55, after
   handshake 50).
3. Top-level **52** `{u16 mod}`: new song object, 0x1910 bytes, **new devices**. Reply `'A'`
   with the slot.
4. Optional top-level **58** when the mod has a startup sample and the preference allows it.
   Starts the preview player. It does not select a sample bank.
5. Then, in order: **36** globals (23 bytes); **41** mixer `0/143` (29 bytes); delay `5/86` (6);
   PCF `7/116` (8); distortion `6/94` (4); compressor `8/104` (5); TB-303 `1/39` then `2/39`
   (3148 each); TR-808 `3/56` (6240); TR-909 `4/73` (6241); **37** track events for tracks 0..8,
   in batches, each acknowledged.
6. **32**, no payload: relocate to the current position, which chases the tracks in song mode.

No msg 33, 35, 20 or 21 is sent at load. Msg 33 is a reset sent after an export and after the
benchmark, and it resets less than its name says (`docs/export_state.md` §6.4). Msg 21 is a
query that replies with msg 36. Msg 20 is a query `{id}` that replies with msg 35. Msg 35 is
the set.

Msg 36 bytes (`docs/song_load.md` §3): 0 song/pattern mode, 1 play/stop (0 at load, so stop),
2 record, 3 loop on, 4..7 tempo BPM×1000, 8..11 locate (0), 12..15 loop start in ticks (not
scaled), 16..19 loop end, 20 shuffle 0..127, 21 sound model, 22 always 1 and has no effect in
the DLL. Ids 4, 8, 12 and 16 carry a u32.

Pattern mode and song mode: each device keeps both. Which values the engine takes from the
tracks at export is `docs/song_load.md` §4 and §6. The 909 pattern-mode loader copies RBS byte
13 (LowTomDecay) into slot 16 and drops byte 16 (OB-1); orb copies the same way.

ReBirth 1.x songs are a MIDI-file import (`docs/song_load.md` §12, `crates/rbs/src/rb1x.rs`,
`crates/orb-engine/src/load/rb1x.rs`). Defects of that path are OB-15 and OB-16.

Song information (title, text, web, copyright) is task-117, `docs/song_load.md` §13, and
`orb info` / `crates/orb-cli/src/info.rs`. The song text's line breaks count twice against its
400 bytes in the original, so a full text is cut on open (OB-21).

The exe's device-data packers use obfuscated indices when the CD check fails. A normal load
cannot trigger that: a bad `RB20FUL.DAT` throws at start-up. rbhost and orb use the intended
layout (`docs/song_load.md` §9).

## 5. Export and the state it sees

The reference WAV is exactly the song loop. No pre-roll, no tail. WAV is RIFF/WAVE, PCM, stereo,
16-bit, 44100 Hz. AIFF uses the same samples. Driver: `sub_427988` [C].

1. Msg 35 `{1,0}` (stop).
2. Query tempo (id 4), loop start (id 12), loop end (id 16). Stop the realtime channel. No
   realtime callback runs during the export.
3. Msg 35 `{8, start}` (locate), then `{1,1}` (play). Both apply in the drain of the first
   `SafeSoundCallback` of the render loop.
4. Render loop, replicated in `crates/orb-engine/src/frame/export.rs`. `pos`, `loopStart`,
   `loopEnd` and the running end are doubles. Tempo is a 32-bit `fild` of BPM×1000. The constant
   `192.0/2646000000.0` is divided at run time. `n` truncates:

   ```
   pos = (double)loopStart
   loop:
     n = (int64)((loopEnd - pos) / (tempo * (192.0/2646000000.0)))
     if n < 2048: last = true else n = 2048
     SafeSoundCallback(buf, n, 0)     # n frames of int16 stereo
     write 4*n bytes
     pos += (tempo * n) * (192.0/2646000000.0)
   until last
   ```

   A final `n` of 0 still makes a 0-frame call. The length matches all 153 diagnostic and
   shipped references. "Fast Forward" (140 BPM, 250 bars) is 18899999 frames, one less than the
   exact 18900000 (Q-1). 152 of those 153 end on a chunk that is not a multiple of 64.
5. Afterwards: msg 35 `{1,0}`, msg 33, `SafeSoundCallback(NULL,0,0)`, then the realtime channel
   restarts.

`SafeSoundCallback` (`0x1001AE40`) drains the message port first, then renders in 64-frame
blocks. A partial block is rendered whole and the remainder is kept. In the app that leftover
is empty at every export start. A preview-sample player is mixed into the int16 buffer only
while a UI preview or the startup sound plays.

The app keeps a realtime stream open from start to quit and pauses it only around an export and
the benchmark. While stopped, every device of the **active** song still runs every block. A song
that is not active is frozen. With the MME driver (the test machine) every realtime call is
1024 frames, so idle time is a multiple of 16 blocks. DirectSound writes multiples of 512.
Messages apply only at the start of a call.

Opening a song creates a new song object. No per-song state carries over. The state at export
start is a function of the song, of `N` idle blocks between activation and export (a multiple of
16), and of a few process-global values. The 303 oscillator phase never settles. Under the
capture protocol, exports are byte-identical across runs when the 303s are silent; otherwise
the differences fit whole idle buffers of 303 phase, `k = N/16`, and are ±1 LSB for songs
without 303 notes. rbhost reproduced three songs of one session byte for byte with `N` = 9088.
Each export moves the global drum counters.

What msg 33 resets, which states survive, the recommended capture protocol and rbhost are
`docs/export_state.md` §6 to §13. The offline host is `tools/rbhost` (task-011). It drives the
DLL through the shared-memory port: ring of 32000 bytes, little-endian u16 length then the
bytes, names `"ReBirth Panel->DLL"` and `"ReBirth DLL->Panel"`.

Corpus compare fits `N` per song (`--idle-blocks`). The 11 mod songs need `--mod-dir` with the
four recrypted shipped mods.

## 6. Mods

Home: `docs/rbm.md`. Implementation: `crates/rbm`, `tools/rbm.py`.

An `.rbm` is an IFF `CAT ` of type `PRBM`: `HEAD` (256, unread except that it shifts chunk
indices), then `EMBF` chunks (NUL-terminated name, then the file), then `INFO` (1280). INFO
holds description, ftp, web, the Mods-menu name (must be non-empty and unique) and a version.
The exe requires a CAT, an INFO of 1280 bytes, a non-empty menu name, and a version major equal
to the app's.

46 sample names are looked up case-insensitively, first match. A missing name keeps the built-in
sample of that slot. Slots 1..24 are the 808 (`TR808BD.aif` ...), 25..45 the 909, 46 is
`Startup.aif`. Names and order: `docs/rbm.md` §3. The four shipped mods replace all 46.

Each EMBF payload is clear through byte 1023 and scrambled from byte 1024 [C]:

```
P    = (17 * key) % 1925 + 123     # 123..2047
base = (key & 1) << 11
for n in 1024 .. size-1:
    payload[n] = (payload[n] + table[base + (n - 1024) % P]) & 0xFF
```

`key` is the low byte of the chunk's index in the CAT, counted from 0 at HEAD, for the samples
the DLL reads. The exe's own descrambler uses the whole u16 index. They agree below index 256,
which every known mod is. The 4096-byte `table` is two 2048-byte pages of `RB20FUL.DAT`, at
offsets derived from two u32 sums over the first 1024 bytes of every EMBF payload. Each mod
needs exactly 4096 bytes of key material, and the pages depend only on the mod's bytes.
`RB20FUL.DAT` must be the one of the 2001 reissue CD; its checksum is checked. The DAT, the key
table and original mods are never committed. Tests read them from `ORB_CD_ISO`.

`orb mod recrypt` rewrites a mod so it needs only a 2047-byte window of that file, which orb
embeds (`crates/rbm/src/rb20ful_window.bin`). The original still opens the result. Recrypting
changes nothing else. An already recrypted mod is checked and copied. `render`, `compare`,
`play` and the UI load only recrypted mods (`--mod-dir` / `ORB_MOD_DIR`). The one command that
reads `RB20FUL.DAT` is `orb mod recrypt`. `--iso` reads the key from the CD image.

`orb mod pack-standard` builds `assets/Standard.rbm` ("Standard ReBirth") from ReBirth.dll's 46
AIFF resources and the CD Rebirth.exe's panel JPEGs and splash. It is embedded. `render`,
`compare`, `play` and the UI need no kit option. SHA-256 pins are in `assets/README.md`. The
installed exe's splash carries a Steinberg logo; the CD exe's does not. The 58 panel bitmaps
match.

The Mods menu changes the mod of an open song in place: song msg 19 `{id}`, which silences the
808 and 909 voices and replaces their sample table. The song plays on (`docs/rbm.md` §14).
Several open songs and msg 53 are §14.5 (Q-4).

`orb mod unpack` / `orb mod pack` round-trip a recrypted mod through a folder and `mod.toml`
(§15).

Third-party mods: `ORB_THIRDPARTY_MODS_DIR` (recrypted) and `ORB_THIRDPARTY_SONGS_DIR`. The mods
test checks the folder against `tests/refs/thirdparty_mods.json` and fails on an unlisted
`.rbm`. Regenerate with `tools/thirdparty_mods_manifest.py`. The songs test requires each song's
`orb render` to hash as rbhost's render in `tests/refs/thirdparty_songs.json`. Regenerate on the
test machine with `tools/rbhost/thirdparty_songs.py`. Details: `docs/rbm.md` §11.

Known mod and sample defects: OB-14 (the open-mod table of 200 EMBF entries has no bound),
OB-22 (the AIFF chunk walk trusts every chunk size), Q-3 (the AIFF reader trusts COMM over the
chunk sizes, and reads past a stale SSND size).

## 7. Defects of the original

orb reproduces these. Full fields (where, trigger, effect, corpus, orb's model) are
`docs/original_bugs.md`. A newly found defect goes there and in its home document.

Song loading: **OB-1** 909 pattern loader copies byte 13 into slot 16. **OB-2** pre-4.2 808 tom
tune runs before the range check. **OB-3** exe accepts ticks to 767976, DLL drops above 767231.
**OB-4** an event exactly at the loop end never fires in a loop. **OB-15** a 1.x control change
with no mapping indexes track buffers with -1. **OB-16** the 1.x song sysex is decoded past its
end, and a foreign sysex is skipped twice. **OB-20** a load the node pool refuses leaves the
half-loaded song in the engine. **OB-21** song-text line breaks count twice against 400 bytes,
so a full text is cut on open.

Devices: **OB-5** the PCF wave byte indexes pattern data with no bounds check. **OB-6** the PCF
divides by a pattern length of 0, and the render call is swallowed. **OB-7** a stale stack
quotient for a PCF resolution other than 8, 16 or 32. **OB-8** the export buffer is never
cleared, so a faulting chunk repeats the previous one. **OB-9** distortion amount and shape
above 127 index past their tables. **OB-10** delay pan above 127 indexes past `PAN[127]`.
**OB-11** PCF decay and resonance bytes past their tables. **OB-12** the delay's 25-update tempo
average is dead code. **OB-17** panel pattern and mixer-channel messages store with no bound on
pattern, step, instrument or channel. **OB-18** a change recorded just before record off is
lost. **OB-19** the merge frees the track's applied node and leaves the pointer.

Export: OB-8. Panel: **OB-13** the 303 Note/Pause key and the drum AC key are `VK_OEM_PLUS`, not
the documented `-`. **OB-23** a switch draws its value as an unclamped frame index. Mods:
**OB-14**, **OB-22**.

Quirks, not claimed as defects: **Q-1** the export's frame count is one frame short of an exact
length. **Q-2** state carried across msg 33 and across songs. **Q-3** the AIFF reader trusts
COMM. **Q-4** a mod change from the Mods menu activates another open song and back.

## 8. The UI

Spec of the original panel: `docs/ui_spec.md`. OpenRB-338 draws its own art at those positions.
The original panel is 624×510 px. Sections, in panel pixels: woods 16 px on each side, transport
592×50 at (16, 0), four rows of 115 px (pattern selector, focus bar, device, mixer channel), and
a right column (master, PCF, delay, dist, comp) at x = 526. The rack scrolls when the window is
shorter than 510. One of the four rows has focus. Pattern mode and song mode do not change the
layout. In pattern mode the bar and loop displays are disabled and the loop does nothing.
Turning song mode off turns record off.

Widget behavior (knobs drag up to increase, spans, switches, step keys, LED frames) is §2 of the
spec. Where the manual and the program differ, the program wins (§13); OB-13 is one of those.

OpenRB-338 forces a dark theme (`set_theme(ThemePreference::Dark)`), ignores the system theme,
and draws the menu bar, menus, file browser and every window with egui. Nothing is a native
dialog except the crash box (`rfd`). The Windows title bar in dark mode is unverified.

Art: `assets/ui/`, generated by `cargo run --release -p orb-ui-art -- gen`. Check with
`ui-art check` and `cargo test -p orb-ui-art`. Placement inside sections is orb's own
(`layout.tsv`), not a copy of the original's control pixels (task-085). Labels are vector paths
of a built-in stroke font. No Propellerhead or Roland artwork is in the SVGs.

Everyday macOS run, after a release build of the UI:

```
cargo build --release -p orb-ui && scripts/macos-dev-app.sh && open target/OpenRB-338.app
```

The bundle is ad-hoc signed (`codesign -s -`). There is no Developer ID and no notarization.
`scripts/macos-app.sh` writes the icon, the plist and the signature. The version string is the
UTC build time `YYYY.MMDD.HHMM` from `SOURCE_DATE_EPOCH` (default now), not the Cargo version.
A plain `cargo build` and a later `macos-dev-app.sh` can show different versions in the About
box and the plist, because each reads the variable at its own time.

Hidden UI options (task-137), absent from `--help`: `--demo`, `--size WxH`, `--ppp S`,
`--browse open|save|export|mod`, `--about`, `--info`, `--screenshot OUT.png`, `--play`, and the
`--debug-panic*` switches. `OpenRB-338 --help` prints `usage: OpenRB-338 [SONG]` and exits 0.
`SONG` is one path. A song that does not open starts a new song and shows the error. An
argument `-psn_...` is ignored.

Crash logs (task-157): `crash-YYYYMMDD-HHMMSS.log`, newest 20 kept, in
`~/Library/Logs/OpenRB-338/` on macOS, `%LOCALAPPDATA%\OpenRB-338\logs\` on Windows,
`$XDG_STATE_HOME/OpenRB-338/` (default `~/.local/state/OpenRB-338/`) on Linux. Settings file:
`OpenRB-338/ui.toml` in the configuration folder, or `ORB_UI_CONFIG`.

Keyboard shortcuts and the panel wiring are `docs/ui_spec.md` §10 and §16 to §19, and
`crates/orb-ui/src/`.

## 9. Build, packages, environment

Full text: `docs/building.md`.

- Everyday: `cargo build --release`, `cargo run --release -p orb-cli -- <args>`.
- Packages use the `dist` profile (inherits release, fat LTO, one codegen unit, strip).
  `scripts/build.sh TARGET` writes `dist/<target>/bin/`. `scripts/package.sh`, `build-all.sh`,
  `package-dmg.sh`, `package-installer.sh` and `package-appimage.sh` build with that profile.
  `SKIP_BUILD=1` packs the existing `bin/`.
- Targets: `x86_64` and `aarch64` for `unknown-linux-gnu`, `pc-windows-msvc` and `apple-darwin`.
  No universal macOS binary. One `.dmg` per architecture. Linux `.tar.gz` and `.AppImage`,
  Windows `.zip` and NSIS installer, macOS `.tar.gz` and `.dmg`.
- This Mac (arm64) builds both Darwin targets with `cargo build --target`. Windows needs
  `cargo-xwin` (`XWIN_ACCEPT_LICENSE=1`). Linux needs Docker. `scripts/build-all.sh` skips what
  this machine cannot build and says why.
- Version is the UTC build minute `YYYY.MMDD.HHMM` (`scripts/version.sh`). About box is
  `<version> (<hash>)`, with `dev-` off main and `-dirty` when tracked files are dirty.
  `ORB_BUILD_BRANCH=main` makes the hash the main form. `SOURCE_DATE_EPOCH` is the single clock
  for the binary, the plist, the file names and the tag `v<version>`.
- Packages ship `orb`, `OpenRB-338`, `LICENSE` and `README.txt`. Nothing is signed. The macOS
  app is ad-hoc signed only. First launch of an unsigned download is refused by Gatekeeper;
  the user opens it from Privacy & Security, or clears `com.apple.quarantine`.
- Linux links `libasound.so.2`. OpenRB-338 also needs X11 or Wayland and a GL or Vulkan driver
  at run time. Minimum glibc is 2.35.
- `scripts/package.sh` and the release workflow are `.github/workflows/ci.yml` and
  `release.yml`. CI runs nightly fmt, clippy, and tests on Linux, Windows and macOS. CI sets
  none of the `ORB_*` test inputs.

`ORB_*` variables (old `RB_*` and `OPENRB_*` names are not read):

| Variable | Meaning |
|---|---|
| `ORB_CD_ISO` | ReBirth 2.01 CD image. Gated tests skip when unset. `orb` commands take `--iso` instead. |
| `ORB_THIRDPARTY_MODS_DIR` | Recrypted third-party mods, read in place. |
| `ORB_THIRDPARTY_SONGS_DIR` | Third-party songs, read in place. |
| `ORB_MOD_DIR` | Recrypted mods for `render`, `compare`, `play`, `info` and the UI. `--mod-dir` wins. |
| `ORB_RB20FUL` | Path of `RB20FUL.DAT` for `orb mod recrypt`. `--rb20ful` wins. |
| `ORB_UI_CONFIG` | Settings file instead of `OpenRB-338/ui.toml`. |
| `ORB_UI_LOG_DIR` | Crash-log folder override. |
| `ORB_UI_NO_CRASH_DIALOG` | Non-empty and not `0`: no native box after a panic. |
| `ORB_LOGO_T` | Freeze the logo animation at this time in seconds. |
| `ORB_GPU_INFO` | `1` prints the wgpu adapter. |
| `ORB_UI_FRAME_STATS` | `1` prints frame rate and frame CPU time every 2 s. |
| `ORB_BUILD_BRANCH` | `main` or `other`, for the About-box hash. |
| `ORB_PYTHON` | Python 3 for tests that run `tools/*.py`. |
| `ORB_LIVE_OUT` | Output folder for `orb-player`'s `live_messages` test. |
| `ORB_VST_SONG` | Optional `.rbs` the VST loads once at startup. The built-in kit is used. |
| `ORB_REBIRTH_DLL` | `ReBirth.dll` for `tools/gen_static_tables.py`. |

Not test inputs: `RBM_TEST_DAT` and `RBM_TEST_MODS` (a cargo test sets them for a child only).
`SOURCE_DATE_EPOCH` and `WGPU_BACKEND` are not `ORB_*`.

`orb compare` is hidden from `orb --help`. It still runs.

## 10. Where the long form is

Read these when the briefing is not enough. Do not paste them back into chat.

| Document | Holds |
|---|---|
| `AGENTS.md` | Backlog, commits, build and test commands in the form the supervisor uses |
| `backlog/docs/doc-001 - agent-brief.md` | Worktree, IDA steps, test-machine permits, rebase check |
| `docs/initial_investigation.md` | Engine survey: export path, mixer block, sequencer, 303, drums, effects, numerics |
| `docs/song_load.md` | `.rbs` to messages, version paths, device chunks, 1.x import, RBS42 errata |
| `docs/export_state.md` | Realtime path, leftover buffer, msg 33, capture protocol, rbhost |
| `docs/rbm.md` | Container, scramble, key pages, recrypt, standard mod, Mods menu, third-party corpus |
| `docs/original_bugs.md` | Every known defect and quirk, with addresses and orb's model |
| `docs/ui_spec.md` | Panel geometry, widgets, keyboard, menus, how orb wires them |
| `docs/test_machine.md` | Installed ReBirth, menus, export by hand, preferences, rbhost, screenshots |
| `docs/building.md` | Profiles, scripts, CI, installers, AppImage, unsigned-download behavior |
| `docs/RBS42.txt` | Official format. Unchanged. Errata in `docs/song_load.md` §10 |
| `assets/README.md` | `Standard.rbm` pins and how it is rebuilt |
| `assets/ui/README.md` | How the SVG panel is generated |
| `.claude/agents/` | Agent-type prompts |
