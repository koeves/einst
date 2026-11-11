#!/usr/bin/env bash
# Build Einst on Windows (Git Bash): VST3 + CLAP and the standalone .exe.
#
#   scripts/build-windows.sh            release build into target/bundled/
#   scripts/build-windows.sh --debug    debug build
#
# Output: target/bundled/Einst.vst3 (resources in Contents/Resources),
# target/bundled/Einst-CLAP/ (Einst.clap + Resources/) and
# target/bundled/Einst-App/ (Einst.exe + Resources/). The .clap and .exe
# are bare files, so Einst's profile looks for `Resources/` next to them.
set -euo pipefail

cd "$(dirname "$0")/.."

profile=release
for a in "$@"; do
    case "$a" in
        --debug) profile=debug ;;
        -h|--help) sed -n '2,10p' "$0"; exit 0 ;;
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
cp "target/$profile/einst-standalone.exe" "$app/Einst.exe"

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

echo "==> Done"
ls -d "$out"/Einst*
