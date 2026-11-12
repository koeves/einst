#!/usr/bin/env bash
# Build Einst on Linux (x86_64, X11): VST3 + CLAP and the standalone app.
#
#   scripts/build-linux.sh            release build into target/bundled/
#   scripts/build-linux.sh --debug    debug build
#   scripts/build-linux.sh --install  also install for this user (below)
#   scripts/build-linux.sh --uninstall  remove what --install put in place
#
# Output: target/bundled/Einst.vst3 (Contents/x86_64-linux/Einst.so,
# resources in Contents/Resources), target/bundled/Einst-CLAP/
# (Einst.clap + Resources/), target/bundled/Einst-App/ (Einst +
# Resources/ + einst.desktop + icons/) and target/bundled/install.sh. The
# .clap and the app are bare files, so Einst's profile looks for
# `Resources/` next to them (as on Windows).
#
# --install and --uninstall run that install.sh (scripts/linux-install.sh,
# which the release tarball also carries; it lists what goes where).
#
# Needs pkg-config and the ALSA, X11, XCB and OpenGL headers (README).
set -euo pipefail

cd "$(dirname "$0")/.."

profile=release
install=0
uninstall=0
for a in "$@"; do
    case "$a" in
        --debug) profile=debug ;;
        --install) install=1 ;;
        --uninstall) uninstall=1 ;;
        -h|--help) sed -n '2,19p' "$0"; exit 0 ;;
        *) echo "unknown option: $a" >&2; exit 2 ;;
    esac
done

if [ "$uninstall" = 1 ]; then
    exec sh scripts/linux-install.sh --uninstall
fi
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
cp -R assets/linux/icons "$app/icons"
cp scripts/linux-install.sh "$out/install.sh"
chmod +x "$out/install.sh"
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
    sh "$out/install.sh"
fi

echo "==> Done"
ls -d "$out"/Einst*
