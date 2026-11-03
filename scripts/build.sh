#!/usr/bin/env bash
# Build Einst: VST3 + CLAP bundles and the standalone macOS app.
# A copy of Dreist's scripts/build.sh (../geist) with Einst's names.
#
#   scripts/build.sh            release build into target/bundled/
#   scripts/build.sh --debug    debug build
#   scripts/build.sh --install  also install to ~/Library/Audio/Plug-Ins and ~/Applications
#
# Output: target/bundled/Einst.vst3, Einst.clap, Einst.app
#
# Signing: ad-hoc by default. With EINST_SIGN_IDENTITY set (a "Developer
# ID Application: ..." identity in the keychain) the bundles are signed
# with the hardened runtime and a timestamp; with EINST_NOTARY_PROFILE
# also set (a `xcrun notarytool store-credentials` profile) they are
# notarised and stapled.
#
# Dreist must be checked out next to this repository (../geist): its crates
# build the engine, and its assets/dreist (UI graphics: skin, fonts, cursors,
# metronomes, read by geist-gui / geist-engine) and licence files go into
# the bundles. Einst's own resources (Geist v1's factory effect presets,
# effect chains, pattern and swing templates; the app icon) come from this
# repository.
set -euo pipefail

cd "$(dirname "$0")/.."
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"

profile=release
install=0
for a in "$@"; do
    case "$a" in
        --debug) profile=debug ;;
        --install) install=1 ;;
        -h|--help) sed -n '2,9p' "$0"; exit 0 ;;
        *) echo "unknown option: $a" >&2; exit 2 ;;
    esac
done
flag=$([ "$profile" = release ] && echo --release || true)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
dreist=../geist
out=target/bundled
mkdir -p "$out"
[ -d "$dreist/assets/dreist" ] || { echo "Dreist not found at $dreist" >&2; exit 1; }

echo "==> VST3 / CLAP ($profile)"
cargo run --package xtask --release -- bundle einst-plugin $flag

echo "==> Standalone app ($profile)"
cargo build -p einst-plugin --features standalone --bin einst-standalone $flag
app="$out/Einst.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "target/$profile/einst-standalone" "$app/Contents/MacOS/Einst"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>Einst</string>
    <key>CFBundleDisplayName</key><string>Einst</string>
    <key>CFBundleExecutable</key><string>Einst</string>
    <key>CFBundleIdentifier</key><string>rs.einst.standalone</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>$version</string>
    <key>CFBundleVersion</key><string>$version</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSMicrophoneUsageDescription</key><string>Einst records audio input with its sampler.</string>
</dict>
</plist>
PLIST
# App icon: Einst's placeholder (assets/einst, drawn by scripts/icon.py).
cp assets/einst/AppIcon.icns "$app/Contents/Resources/AppIcon.icns"

# Resources: Einst's factory data in Contents/Resources/Einst (where
# einst_editor::paths looks: Geist v1's effect presets, effect chains,
# pattern and swing templates, which einst-formats reads 1:1), Dreist's UI
# graphics in Contents/Resources/Dreist (geist_formats::paths::ui_dir), plus
# the Einst theme folder the editor selects by default (empty: the base skin).
echo "==> Resources"
for b in "$app" "$out/Einst.vst3" "$out/Einst.clap"; do
    mkdir -p "$b/Contents/Resources/Einst"
    rm -rf "$b/Contents/Resources/Geist2" "$b/Contents/Resources/Dreist" "$b/Contents/Resources/Einst"/*
    for d in "Effects" "Effect Chains" "Pattern Templates" "Swing Templates"; do
        cp -R "assets/geist1/System/$d" "$b/Contents/Resources/Einst/$d"
    done
    cp -R "$dreist/assets/dreist" "$b/Contents/Resources/Dreist"
    rm -f "$b/Contents/Resources/Dreist/preview.png" "$b/Contents/Resources/Dreist/AppIcon.png" "$b/Contents/Resources/Dreist/AppIcon.icns"
    mkdir -p "$b/Contents/Resources/Dreist/Skin/Styles/Einst.style"
    echo "Einst's theme: the base skin (no overrides)." > "$b/Contents/Resources/Dreist/Skin/Styles/Einst.style/README.txt"
done
# Licence and notices travel with every bundle (Dreist's: the plugin
# links nih-plug's GPLv3 VST3 bindings and Dreist's crates).
for b in "$app" "$out/Einst.vst3" "$out/Einst.clap"; do
    cp "$dreist/LICENSE" "$dreist/LICENSE-GPL-3.0" "$dreist/THIRD_PARTY_NOTICES.md" "$b/Contents/Resources/"
done
for b in "$app" "$out/Einst.vst3" "$out/Einst.clap"; do
    if [ -n "${EINST_SIGN_IDENTITY:-}" ]; then
        echo "==> Signing $b"
        codesign --force --deep --options runtime --timestamp --sign "$EINST_SIGN_IDENTITY" "$b"
    else
        codesign --force --deep --sign - "$b" >/dev/null 2>&1 || echo "warning: ad-hoc codesign failed for $b" >&2
    fi
done
if [ -n "${EINST_SIGN_IDENTITY:-}" ] && [ -n "${EINST_NOTARY_PROFILE:-}" ]; then
    for b in "$app" "$out/Einst.vst3" "$out/Einst.clap"; do
        echo "==> Notarising $b"
        zip="$out/$(basename "$b").zip"
        rm -f "$zip"
        ditto -c -k --keepParent "$b" "$zip"
        xcrun notarytool submit "$zip" --keychain-profile "$EINST_NOTARY_PROFILE" --wait
        xcrun stapler staple "$b"
        rm -f "$zip"
    done
fi

if [ "$install" = 1 ]; then
    echo "==> Installing"
    mkdir -p "$HOME/Library/Audio/Plug-Ins/VST3" "$HOME/Library/Audio/Plug-Ins/CLAP" "$HOME/Applications"
    rm -rf "$HOME/Library/Audio/Plug-Ins/VST3/Einst.vst3" "$HOME/Library/Audio/Plug-Ins/CLAP/Einst.clap" "$HOME/Applications/Einst.app"
    cp -R "$out/Einst.vst3" "$HOME/Library/Audio/Plug-Ins/VST3/"
    cp -R "$out/Einst.clap" "$HOME/Library/Audio/Plug-Ins/CLAP/"
    cp -R "$app" "$HOME/Applications/"
fi

echo "==> Done"
ls -d "$out"/Einst.*
