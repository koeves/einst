#!/usr/bin/env bash
# Build Einst on Linux (x86_64, X11): VST3 + CLAP and the standalone app.
#
#   scripts/build-linux.sh            release build into target/bundled/
#   scripts/build-linux.sh --debug    debug build
#   scripts/build-linux.sh --install  also install for this user (below)
#
# Output: target/bundled/Einst.vst3 (Contents/x86_64-linux/Einst.so,
# resources in Contents/Resources), target/bundled/Einst-CLAP/
# (Einst.clap + Resources/) and target/bundled/Einst-App/ (Einst +
# Resources/ + einst.desktop). The .clap and the app are bare files, so
# Einst's profile looks for `Resources/` next to them (as on Windows).
#
# --install: ~/.vst3/Einst.vst3, ~/.clap/Einst/ (hosts search ~/.clap
# recursively), the app in ~/.local/share/Einst/ with ~/.local/bin/einst
# linking to it, and a desktop entry and icon under ~/.local/share.
#
# Needs pkg-config and the ALSA, X11, XCB and OpenGL headers (README).
set -euo pipefail

cd "$(dirname "$0")/.."

profile=release
install=0
for a in "$@"; do
    case "$a" in
        --debug) profile=debug ;;
        --install) install=1 ;;
        -h|--help) sed -n '2,18p' "$0"; exit 0 ;;
        *) echo "unknown option: $a" >&2; exit 2 ;;
    esac
done
flag=$([ "$profile" = release ] && echo --release || true)
out=target/bundled
mkdir -p "$out"

echo "==> VST3 / CLAP ($profile)"
cargo run --package xtask --release -- bundle einst-plugin $flag

echo "==> Standalone app ($profile)"
cargo build -p einst-plugin --features standalone --bin einst-standalone $flag

clap="$out/Einst-CLAP"
app="$out/Einst-App"
rm -rf "$clap" "$app"
mkdir -p "$clap/Resources" "$app/Resources" "$out/Einst.vst3/Contents/Resources"
mv "$out/Einst.clap" "$clap/Einst.clap"
cp "target/$profile/einst-standalone" "$app/Einst"
cp assets/ui-v1/AppIcon.png "$app/einst.png"
cat > "$app/einst.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Einst
Comment=Drum sampler, sequencer and performance instrument
Exec=einst
Icon=einst
Terminal=false
Categories=AudioVideo;Audio;Music;
DESKTOP

echo "==> Resources"
for r in "$out/Einst.vst3/Contents/Resources" "$clap/Resources" "$app/Resources"; do
    rm -rf "$r/Factory" "$r/Einst"
    cp -R assets/factory "$r/Factory"
    cp -R assets/ui-v1 "$r/Einst"
    rm -f "$r/Einst/preview.png" "$r/Einst/AppIcon.png" "$r/Einst/AppIcon.icns"
done
for d in "$out/Einst.vst3/Contents/Resources" "$clap" "$app"; do
    cp LICENSE LICENSE-GPL-3.0 THIRD_PARTY_NOTICES.md "$d/"
done

if [ "$install" = 1 ]; then
    echo "==> Installing"
    share="${XDG_DATA_HOME:-$HOME/.local/share}"
    mkdir -p "$HOME/.vst3" "$HOME/.clap" "$share" "$HOME/.local/bin" "$share/applications" "$share/icons/hicolor/256x256/apps"
    rm -rf "$HOME/.vst3/Einst.vst3" "$HOME/.clap/Einst" "$share/Einst"
    cp -R "$out/Einst.vst3" "$HOME/.vst3/"
    cp -R "$clap" "$HOME/.clap/Einst"
    cp -R "$app" "$share/Einst"
    ln -sf "$share/Einst/Einst" "$HOME/.local/bin/einst"
    cp "$app/einst.desktop" "$share/applications/einst.desktop"
    cp "$app/einst.png" "$share/icons/hicolor/256x256/apps/einst.png"
fi

echo "==> Done"
ls -d "$out"/Einst*
