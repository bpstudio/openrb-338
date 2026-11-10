# Building the OpenRB-338 VST3 to send to another Mac

The plugin is not signed with a Developer ID and is not notarized. An ad-hoc signature is
enough on the Mac that built it. The person who receives the bundle follows `VST-README.md`
and `VST_INSTALL.md`.

On this Mac:

```sh
scripts/build_zip_release.sh
```

That builds both plugins first:

```sh
cargo build --release -p orb-vst --target aarch64-apple-darwin
cargo build --release -p orb-vst --target x86_64-apple-darwin
```

Then it writes three zips, and copies `VST-README.md` into `dist/` as well. The version is
the UTC minute from `scripts/version.sh`.

- `dist/OpenRB-338-<version>-aarch64-apple-darwin-vst3.zip`, Apple silicon
- `dist/OpenRB-338-<version>-x86_64-apple-darwin-vst3.zip`, Intel
- `dist/OpenRB-338-<version>-songs.zip`, the files from `songs/rb-songs/songs`

Each plugin zip holds one folder, `OpenRB-338`, with:

- `OpenRB-338.vst3`, the bundle only (`libopenrb338.dylib` by itself is not a plugin)
- `install-vst.sh`, which copies it to `~/Library/Audio/Plug-Ins/VST3` and runs `xattr`
- `VST-README.md` and `VST_INSTALL.md`

The Apple silicon wrap also installs a copy at
`~/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3` for the local copy of Ableton Live. The Intel
bundle is packed only. The other Mac needs macOS 11 or newer.
