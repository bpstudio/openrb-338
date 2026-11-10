#!/usr/bin/env bash
# Wraps the VST3 you just built into OpenRB-338.vst3 and installs it where Ableton
# Live scans on this Mac:
#
#   cargo build --release -p orb-vst && scripts/macos-vst3.sh
#
#   scripts/macos-vst3.sh [--no-install] [LIBOPENRB338_DYLIB]
#
# The dylib defaults to target/release/libopenrb338.dylib. The bundle is
# target/OpenRB-338.vst3 (ad-hoc signed). A copy is installed at
# ~/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3. --no-install only writes the
# bundle, which the release zip uses for the Intel build. Run the script again
# after each build, then rescan plug-ins in Live. The instrument is specified
# in PLAN.md.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
target=${CARGO_TARGET_DIR:-$root/target}
install=1
if [ "${1:-}" = --no-install ]; then
    install=0
    shift
fi
lib=${1:-$target/release/libopenrb338.dylib}
[ -f "$lib" ] || { echo "no $lib: cargo build --release -p orb-vst" >&2; exit 1; }

bundle=$target/OpenRB-338.vst3
rm -rf "$bundle"
mkdir -p "$bundle/Contents/MacOS"
cp "$lib" "$bundle/Contents/MacOS/OpenRB-338"
chmod 755 "$bundle/Contents/MacOS/OpenRB-338"
printf 'BNDL????' > "$bundle/Contents/PkgInfo"
cat > "$bundle/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleExecutable</key>
	<string>OpenRB-338</string>
	<key>CFBundleIdentifier</key>
	<string>net.emphased.openrb338.vst3</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>OpenRB-338</string>
	<key>CFBundlePackageType</key>
	<string>BNDL</string>
	<key>CFBundleShortVersionString</key>
	<string>0.1.0</string>
	<key>CFBundleSignature</key>
	<string>????</string>
	<key>CFBundleSupportedPlatforms</key>
	<array>
		<string>MacOSX</string>
	</array>
	<key>CFBundleVersion</key>
	<string>0.1.0</string>
	<key>LSMinimumSystemVersion</key>
	<string>11.0</string>
</dict>
</plist>
EOF
plutil -lint "$bundle/Contents/Info.plist"
codesign --force --sign - "$bundle"
codesign --verify --strict --verbose=2 "$bundle"

echo "made $bundle"
if [ "$install" -eq 1 ]; then
    dest="$HOME/Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3"
    mkdir -p "$HOME/Library/Audio/Plug-Ins/VST3"
    rm -rf "$dest"
    cp -R "$bundle" "$dest"
    echo "installed $dest"
fi
