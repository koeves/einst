#!/bin/sh
# The version a build shows. Release builds in CI write the tag's version
# into Cargo.toml (release.yml); other builds show the last tag and the
# commits since (0.3.0+5), or the last tag with +dev when it is not in this
# branch's history.
set -eu
cd "$(dirname "$0")/.."
v=$(sed -n 's/^version = "\([^"]*\)".*/\1/p' Cargo.toml | head -n 1)
if [ "$v" != 0.0.0 ]; then
    echo "$v"
    exit 0
fi
d=$(git describe --tags --match 'v[0-9]*' 2>/dev/null || true)
case "$d" in
    "")
        t=$(git tag --list 'v[0-9]*' --sort=-v:refname 2>/dev/null | head -n 1)
        if [ -n "$t" ]; then echo "${t#v}+dev"; else echo "0.0.0+dev"; fi
        ;;
    *-*-g*)
        n=${d%-g*}
        echo "${n%-*}" | sed "s/^v//; s/\$/+${n##*-}/"
        ;;
    *) echo "${d#v}" ;;
esac
