#!/usr/bin/env bash
# Measures every camera setting worth comparing on both delivery paths, and writes one table.
#
# Usage: measure-all.sh [seconds] [results.md] [device]
#
# Each row is one run of measure.sh. "Emitted" is what the Plugin says it put on the stream, "Arrived" what
# the client counted; a row that arrived with nothing while the Plugin emitted is a delivery that failed.
# A camera that lowers its rate in dim light (exposure_dynamic_framerate) measures the light, not Yoke:
# the script turns it off for the runs and restores it.
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
seconds="${1:-15}" results="${2:-$here/results.md}" device="${3:-/dev/video0}"

settings=(
  "mjpeg 640x480 30"
  "mjpeg 1280x720 30"
  "yuyv 320x240 30"
  "yuyv 424x240 30"
  "yuyv 640x360 30"
  "yuyv 640x480 30"
)

dynamic="$(v4l2-ctl -d "$device" -C exposure_dynamic_framerate 2>/dev/null | awk '{print $2}')"
[[ -n "$dynamic" ]] && v4l2-ctl -d "$device" -c exposure_dynamic_framerate=0
restore() { [[ -n "$dynamic" ]] && v4l2-ctl -d "$device" -c exposure_dynamic_framerate="$dynamic"; }
trap restore EXIT

{
  echo "# video-simple measurements"
  echo
  echo "$(date -u +%Y-%m-%dT%H:%MZ) on $(uname -n), $(uname -r), $(yoke --version 2>/dev/null), ${seconds} s per run, $device."
  echo "\`net.core.wmem_default\` $(cat /proc/sys/net/core/wmem_default), \`wmem_max\` $(cat /proc/sys/net/core/wmem_max)."
  echo
  echo "| Path | Setting | Emitted | Arrived | Lost | Latency (avg / worst) |"
  echo "| --- | --- | --- | --- | ---: | --- |"
} > "$results"

for path in local web; do
  for s in "${settings[@]}"; do
    read -r format size fps <<< "$s"
    echo "== $path $format $size $fps" >&2
    out="$("$here/measure.sh" "$path" "$format" "$size" "$fps" "$seconds" "" "$device" 2>&1)"
    echo "$out" >&2
    arrived="$(grep -E '^(record|measure-web): [0-9.]+ s, ' <<< "$out" | tail -1)"
    emitted="$(grep -E 'video-simple: [0-9.]+ fps' <<< "$out" | tail -1 | sed -E 's/.*video-simple: ([0-9.]+ fps), ([0-9.]+ MB\/s), ([0-9]+ KB) average.*/\1, \2, \3/')"
    fps_in="$(sed -nE 's/.*: ([0-9.]+ fps), ([0-9.]+ MB\/s), ([0-9]+ KB) average.*/\1, \2, \3/p' <<< "$arrived")"
    lost="$(sed -nE 's/.* ([0-9]+) lost \(([0-9.]+)%\).*/\1 (\2%)/p' <<< "$arrived")"
    [[ "$fps_in" == 0.0\ fps* ]] && fps_in="" lost="" 
    latency="$(sed -nE 's/.*latency ([0-9.]+) ms average, ([0-9.]+) ms worst.*/\1 \/ \2 ms/p' <<< "$arrived")"
    [[ -z "$fps_in" ]] && latency=""
    echo "| $path | $format $size @ $fps | ${emitted:-—} | ${fps_in:-**nothing**} | ${lost:-—} | ${latency:-—} |" >> "$results"
  done
done
echo >&2
cat "$results"
