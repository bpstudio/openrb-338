#!/usr/bin/env bash
# Copies OpenRB-338.vst3 from this folder into the user VST3 directory and clears
# the macOS quarantine flag on that copy. Run it from the unzipped release folder,
# after quitting Ableton Live. VST_INSTALL.md is the same steps in prose.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
src=$here/OpenRB-338.vst3
dest=$HOME/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3

[ -d "$src/Contents/MacOS" ] || {
    echo "missing $src (run this script from the unzipped folder)" >&2
    exit 1
}

mkdir -p "$HOME/Library/Audio/Plug-Ins/VST3"
rm -rf "$dest"
cp -R "$src" "$dest"
# -d deletes one attribute, -r walks the bundle. Do not use xattr -c: that
# strips every attribute and can break the ad-hoc signature.
xattr -dr com.apple.quarantine "$dest"

echo "installed $dest"
echo "Quit was required before this copy. Start Ableton Live and rescan plug-ins if OpenRB-338 is missing."
