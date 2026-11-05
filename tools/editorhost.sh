#!/bin/bash
# Open the editors of several plugin bundles in one host process, one after
# the other and then together, without anything coming to the front:
#
#   tools/editorhost.sh target/bundled/Einst.clap ../dreist/target/bundled/Dreist.clap
#
# The host (tools/editorhost.m) runs in background mode with a scratch HOME,
# and this script fails when the editors fail, the host crashes or the host
# became the frontmost app (switching apps yourself meanwhile is fine).
set -u
here=$(cd "$(dirname "$0")" && pwd)
out=${TMPDIR:-/tmp}/editorhost.$$
mkdir -p "$out/home"
trap 'rm -rf "$out"' EXIT
bundles=()
for b in "$@"; do bundles+=("$(cd "$(dirname "$b")" && pwd)/$(basename "$b")"); done
clang -fobjc-arc -framework Cocoa -o "$out/editorhost" "$here/editorhost.m" || exit 2
front() { lsappinfo info -only name "$(lsappinfo front)" | sed 's/.*="\(.*\)"/\1/'; }
HOME=$out/home DREIST_CONTENT=$out/home/none DREIST_BACKGROUND_GUI=1 \
  perl -e 'alarm 120; exec @ARGV' "$out/editorhost" "${bundles[@]}" > "$out/log" 2>&1 &
pid=$!
changed=""
while kill -0 $pid 2>/dev/null; do
  f=$(front)
  [ "$f" = editorhost ] && changed=yes
  sleep 0.1
done
wait $pid
code=$?
grep -E "^(==|open|close|  |OK|FAILED)|panicked|could not create" "$out/log"
if [ $code -ne 0 ]; then
  echo "editorhost: exit $code"
  tail -20 "$out/log"
  exit 1
fi
if [ -n "$changed" ]; then
  echo "editorhost: the host came to the front"
  exit 1
fi
echo "editorhost: ok (never frontmost)"
