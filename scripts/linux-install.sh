#!/bin/sh
# Install Einst for this user on Linux, or remove it.
#
#   ./install.sh               install what sits next to this script
#   ./install.sh --uninstall   remove what an install put in place
#
# Next to this script: Einst.vst3, Einst-CLAP/ (Einst.clap + Resources/)
# and Einst-App/ (Einst, Resources/, einst.desktop, icons/). Each part is
# installed when present:
#
#   ~/.vst3/Einst.vst3
#   ~/.clap/Einst/                        (hosts search ~/.clap recursively)
#   ~/.local/share/Einst/                 the app, with ~/.local/bin/einst
#   ~/.local/share/applications/einst.desktop
#   ~/.local/share/icons/hicolor/<size>/apps/einst.png
#
# Settings (~/.config/Einst) and content are left alone.
set -eu

app=Einst
bin=einst
here=$(cd "$(dirname "$0")" && pwd)
share=${XDG_DATA_HOME:-$HOME/.local/share}
sizes="48 64 128 256 512"

refresh() {
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q "$share/applications" || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t "$share/icons/hicolor" 2>/dev/null || true
    fi
}

remove() {
    rm -rf "$HOME/.vst3/$app.vst3" "$HOME/.clap/$app" "$share/$app"
    rm -f "$HOME/.local/bin/$bin" "$share/applications/$bin.desktop"
    for s in $sizes; do
        rm -f "$share/icons/hicolor/${s}x$s/apps/$bin.png"
    done
}

case "${1:-}" in
    --uninstall)
        remove
        refresh
        echo "$app removed (settings in ~/.config/$app and content kept)"
        exit 0
        ;;
    "") ;;
    *)
        sed -n '2,17p' "$0"
        exit 2
        ;;
esac

# A reinstall replaces everything, so files of an older version do not stay.
remove
if [ -d "$here/$app.vst3" ]; then
    mkdir -p "$HOME/.vst3"
    cp -R "$here/$app.vst3" "$HOME/.vst3/"
    echo "VST3: ~/.vst3/$app.vst3"
fi
if [ -d "$here/$app-CLAP" ]; then
    mkdir -p "$HOME/.clap"
    cp -R "$here/$app-CLAP" "$HOME/.clap/$app"
    echo "CLAP: ~/.clap/$app"
fi
if [ -d "$here/$app-App" ]; then
    mkdir -p "$share" "$HOME/.local/bin" "$share/applications"
    cp -R "$here/$app-App" "$share/$app"
    ln -sf "$share/$app/$app" "$HOME/.local/bin/$bin"
    # Absolute paths: ~/.local/bin is only on PATH if it existed at login,
    # and launchers hide entries whose program they cannot find.
    awk -v exe="$share/$app/$app" -v icon="$bin" '
        /^Exec=/ { print "TryExec=" exe; print "Exec=" exe; next }
        /^TryExec=/ { next }
        /^Icon=/ { print "Icon=" icon; next }
        { print }' "$share/$app/$bin.desktop" >"$share/applications/$bin.desktop"
    for s in $sizes; do
        if [ -f "$share/$app/icons/$s/$bin.png" ]; then
            mkdir -p "$share/icons/hicolor/${s}x$s/apps"
            cp "$share/$app/icons/$s/$bin.png" "$share/icons/hicolor/${s}x$s/apps/$bin.png"
        fi
    done
    echo "App: ~/.local/share/$app (launcher \"$app\", command $bin)"
fi
refresh
