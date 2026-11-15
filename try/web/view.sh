#!/usr/bin/env bash
# Starts the application and the page's front, and leaves both up until interrupted: open the address it
# prints in a browser on this host.
#
# Usage: view.sh [mjpeg|yuyv] [WxH] [fps] [device] [listen]
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../lib.sh"

format="${1:-mjpeg}" size="${2:-1280x720}" fps="${3:-30}" device="${4:-/dev/video0}" listen="${5:-127.0.0.1:8080}"
start_app "$format" "$size" "$fps" "$device"
start_front "$listen"
echo "open http://$listen/ — $format $size at $fps fps from $device; Ctrl-C stops it"
wait "$run_pid"
