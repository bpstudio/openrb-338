#!/usr/bin/env bash
# Builds both macOS VST3s, then writes three zips:
#
#   dist/OpenRB-338-<version>-aarch64-apple-darwin-vst3.zip   Apple silicon
#   dist/OpenRB-338-<version>-x86_64-apple-darwin-vst3.zip    Intel
#   dist/OpenRB-338-<version>-songs.zip                       songs/rb-songs/songs
#
# Each plugin zip is one folder, OpenRB-338:
#
#   OpenRB-338.vst3    the plugin bundle only (not the raw dylib)
#   install-vst.sh     copies it to ~/Library/Audio/Plug-Ins/VST3 and runs xattr
#   VST-README.md
#   VST_INSTALL.md
#
# The version is the UTC minute from scripts/version.sh. VST-README.md is also
# copied to dist/ on every run. The Apple silicon bundle is installed locally
# for Ableton Live. The Intel bundle is packed only.
#
#   scripts/build_zip_release.sh
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
songs=$root/songs/rb-songs/songs
version=$("$root/scripts/version.sh")
stage=$(mktemp -d "${TMPDIR:-/tmp}/openrb-vst-release.XXXXXX")
trap 'rm -rf "$stage"' EXIT

[ -d "$songs" ] || {
    echo "missing $songs" >&2
    exit 1
}

mkdir -p "$root/dist"
# COPYFILE_DISABLE keeps Finder's AppleDouble files (._*) out of the zip.
export COPYFILE_DISABLE=1

cargo build --release -p orb-vst --target aarch64-apple-darwin
cargo build --release -p orb-vst --target x86_64-apple-darwin

# triple, lipo arch name, install|no-install
pack_plugin() {
    triple=$1
    want_arch=$2
    install_flag=$3
    lib=$root/target/$triple/release/libopenrb338.dylib
    [ -f "$lib" ] || {
        echo "missing $lib" >&2
        exit 1
    }
    got=$(lipo -archs "$lib")
    [ "$got" = "$want_arch" ] || {
        echo "$lib is '$got', expected $want_arch" >&2
        exit 1
    }
    if [ "$install_flag" = install ]; then
        "$root/scripts/macos-vst3.sh" "$lib"
    else
        "$root/scripts/macos-vst3.sh" --no-install "$lib"
    fi

    pack=$stage/OpenRB-338
    rm -rf "$pack"
    mkdir -p "$pack"
    cp -R "$root/target/OpenRB-338.vst3" "$pack/OpenRB-338.vst3"
    cp "$root/VST-README.md" "$pack/VST-README.md"
    cp "$root/VST_INSTALL.md" "$pack/VST_INSTALL.md"
    cp "$root/scripts/install-vst.sh" "$pack/install-vst.sh"
    chmod 755 "$pack/install-vst.sh"
    find "$pack" \( -name .DS_Store -o -name '._*' \) -delete

    out=$root/dist/OpenRB-338-$version-$triple-vst3.zip
    rm -f "$out"
    ditto -c -k --norsrc --keepParent "$pack" "$out"
    echo "$out"
}

# Intel first. The silicon wrap installs the local copy afterwards.
pack_plugin x86_64-apple-darwin x86_64 no-install
pack_plugin aarch64-apple-darwin arm64 install

cp "$root/VST-README.md" "$root/dist/VST-README.md"

songs_stage=$stage/songs
rm -rf "$songs_stage"
mkdir -p "$songs_stage"
# The song files only. A .git directory in the collection root stays behind.
cp -R "$songs/." "$songs_stage/"
find "$songs_stage" \( -name .DS_Store -o -name '._*' \) -delete
songs_out=$root/dist/OpenRB-338-$version-songs.zip
rm -f "$songs_out"
ditto -c -k --norsrc --keepParent "$songs_stage" "$songs_out"
echo "$songs_out"
