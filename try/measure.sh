#!/usr/bin/env bash
# Measures the stream on one delivery path, with one camera setting, for a while.
#
# Usage: measure.sh <direct|local|web> <mjpeg|yuyv|synthetic> <WxH> <fps> <seconds> [out-file] [device]
#   direct  record, reading the stream's shared ring itself; out-file is the recording
#   local   record, through the viewer channel's per-subscriber socket, every frame copied by the Core
#   web     measure-web.mjs, through the front and the web channel's binary WebSocket, as a page receives it
#   e.g. measure.sh local yuyv 640x480 30 20 /tmp/v.yuyv
#
# synthetic is packed YUYV drawn by the Plugin, for a frame larger than the camera offers. The last line
# says the share of one CPU the Core spent while the client ran.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

path="$1" format="$2" size="$3" fps="$4" seconds="$5" out="${6:-}" device="${7:-/dev/video0}"
start_app "$format" "$size" "$fps" "$device"
core_pid="$(pgrep -n -x yoke-core)"
ticks() { awk '{print $14 + $15}' "/proc/$core_pid/stat"; }
before="$(ticks)" began="$(date +%s.%N)"
case "$path" in
  direct)
    "$record" --instance "$instance" --channel viewer --reads ring ${out:+--out "$out"} --seconds "$seconds" || true ;;
  local)
    "$record" --instance "$instance" --channel viewer --reads socket ${out:+--out "$out"} --seconds "$seconds" || true ;;
  web)
    start_front
    node "$try_dir/web/measure-web.mjs" --base http://127.0.0.1:8080 --seconds "$seconds" ${out:+--json "$out"} || true ;;
  *) echo "the path is direct, local or web, not $path" >&2; exit 2 ;;
esac
awk -v a="$before" -v b="$(ticks)" -v t0="$began" -v t1="$(date +%s.%N)" -v hz="$(getconf CLK_TCK)" \
  'BEGIN { printf "core: %.1f%% of a CPU\n", 100 * (b - a) / hz / (t1 - t0) }'
echo "--- what the unit said"
unit_said
