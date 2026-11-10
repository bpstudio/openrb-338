# Installing the OpenRB-338 VST3 on another Mac

You receive a zip of the `OpenRB-338.vst3` folder. The plugin is not signed with a Developer
ID and is not notarized. macOS can quarantine the copy, and Ableton Live will not load it
until that quarantine is removed.

Quarantine is a macOS extended attribute named `com.apple.quarantine`. A browser download,
AirDrop, Mail, and Messages set it. Remove it with `xattr`, in the install steps below.

## Install

1. Quit Ableton Live completely. Closing the window is not enough; Live must not be running.
2. Unzip the plugin archive for this Mac (`aarch64` on Apple silicon, `x86_64` on Intel).
   You get a folder `OpenRB-338` with `OpenRB-338.vst3`, `VST-README.md`, this file, and
   `install-vst.sh`.
3. In Terminal, from that folder:

   ```sh
   ./install-vst.sh
   ```

   The script copies the bundle to `~/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3`,
   replacing an older copy, and runs:

   ```sh
   xattr -dr com.apple.quarantine ~/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3
   ```

   `-d` deletes the quarantine attribute, `-r` walks every file in the bundle. A check
   that prints nothing means it is gone:

   ```sh
   xattr -lr ~/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3 | grep quarantine
   ```

4. Start Live again. If the plugin does not appear, use Plug-Ins > Rescan.

Songs are a separate zip, `OpenRB-338-<version>-songs.zip`. Unzip it and open the `.rbs`
files from the plugin. The script copies the plugin only.

Do not run `xattr -c` or `xattr -cr`. Those delete every extended attribute and can break the
ad-hoc signature. Delete only `com.apple.quarantine`.

`scp`, a USB drive, or a local `cp` often do not quarantine the file. `xattr` still runs, and
if the attribute is absent it changes nothing.

## If Live still does not see it

- The copied item is a folder named `OpenRB-338.vst3`, and it contains
  `Contents/MacOS/OpenRB-338`.
- Live was fully quit before the replacement. Otherwise the old copy stays loaded.
- The archive matches the Mac. Apple silicon uses the `aarch64` zip, Intel uses the `x86_64`
  zip. The Mac needs macOS 11 or newer.
- The quarantine command was run on the copy in the VST3 folder, not on the zip left in
  Downloads.
