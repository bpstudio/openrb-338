# OpenRB-338 VST3

OpenRB-338 is a VST3 instrument for Ableton Live on macOS. This folder is a build you can
install on your own Mac. It is not signed with a Developer ID and it is not notarized, so
macOS marks a downloaded copy as quarantined. Live will not load the plugin until that mark
is removed.

Quarantine is the extended attribute `com.apple.quarantine`. A browser download, AirDrop,
Mail, and Messages set it. The install script removes it with `xattr`.

## Which archive

There are two plugin archives. Use the one that matches the Mac. The Mac needs macOS 11 or
newer.

- `OpenRB-338-<version>-aarch64-apple-darwin-vst3.zip` for Apple silicon
- `OpenRB-338-<version>-x86_64-apple-darwin-vst3.zip` for Intel

Songs are a third archive, `OpenRB-338-<version>-songs.zip`. Unzip it wherever you keep the
songs, and open the `.rbs` files from the plugin. The install script copies the plugin only.

## What is in this folder

- `OpenRB-338.vst3`, the plugin. It is a folder. `libopenrb338.dylib` by itself is not the
  plugin.
- `install-vst.sh`, which copies the plugin into the standard user VST3 folder and clears
  quarantine on that copy.
- `VST_INSTALL.md`, the same install steps in shorter form.

The plugin uses the built-in drum kit. It follows Live's tempo. A song you load is stored in
the Live project.

## Install

1. Quit Ableton Live completely. Live must not be running.
2. Unzip the plugin archive for this Mac. Open Terminal and `cd` into the `OpenRB-338` folder.
3. Run:

   ```sh
   ./install-vst.sh
   ```

   The script copies the bundle to:

   ```text
   ~/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3
   ```

   It replaces an older copy there, then runs:

   ```sh
   xattr -dr com.apple.quarantine ~/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3
   ```

   `-d` deletes the quarantine attribute. `-r` walks every file in the bundle. This check
   prints nothing when the attribute is gone:

   ```sh
   xattr -lr ~/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3 | grep quarantine
   ```

4. Start Live. If OpenRB-338 is missing, use Plug-Ins > Rescan.

`scp`, a USB drive, or a local `cp` often leave the file without quarantine. `xattr` still
runs, and with the attribute already absent it changes nothing.

Run `xattr` only as `-dr com.apple.quarantine`. `xattr -c` and `xattr -cr` delete every
extended attribute and can break the ad-hoc signature.

## If Live still does not see it

- The installed item is a folder named `OpenRB-338.vst3`, and it contains
  `Contents/MacOS/OpenRB-338`.
- Live was fully quit before the copy. A running Live keeps the previous binary loaded.
- The archive matches the Mac: Apple silicon uses the `aarch64` zip, Intel uses the `x86_64`
  zip.
- The quarantine command was run on the copy in `~/Library/Audio/Plug-Ins/VST3`, which is
  where `install-vst.sh` runs it.
