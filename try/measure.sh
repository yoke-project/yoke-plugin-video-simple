#!/usr/bin/env bash
# Measures the stream on one delivery path, with one camera setting, for a while.
#
# Usage: measure.sh <local|web> <mjpeg|yuyv> <WxH> <fps> <seconds> [out-file] [device]
#   local   record, through the viewer channel's per-subscriber socket; out-file is the recording
#   web     measure-web.mjs, through the front and the web channel's binary WebSocket, as a page receives it
#   e.g. measure.sh local yuyv 640x480 30 20 /tmp/v.yuyv
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

path="$1" format="$2" size="$3" fps="$4" seconds="$5" out="${6:-}" device="${7:-/dev/video0}"
start_app "$format" "$size" "$fps" "$device"
case "$path" in
  local)
    "$record" --instance "$instance" --channel viewer ${out:+--out "$out"} --seconds "$seconds" || true ;;
  web)
    start_front
    node "$try_dir/web/measure-web.mjs" --base http://127.0.0.1:8080 --seconds "$seconds" ${out:+--json "$out"} || true ;;
  *) echo "the path is local or web, not $path" >&2; exit 2 ;;
esac
echo "--- what the unit said"
unit_said
