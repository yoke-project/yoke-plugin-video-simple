#!/usr/bin/env bash
# The watch: this Plugin against what Yoke last published. The newest programs are installed from the module
# proxy; the Plugin and the recorder are built against the published library; the application in try/ is run
# under a real Core with the synthetic source, which needs no camera, at a frame above 1 MiB; and the recorder
# reads the stream on both local paths. It leaves its results where the record writer reads them.
# Usage: watch.sh [results directory]
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
results="${1:-$root/.results}"
mkdir -p "$results"
date -u +%Y-%m-%dT%H:%M:%SZ > "$results/started"
CASE=yoke-plugin-video-simple:the-watch.01

finish() { # <verdict line>
  echo "$1" | tee "$results/checks.txt"
  date -u +%Y-%m-%dT%H:%M:%SZ > "$results/finished"
  [[ "$1" == pass* ]]
  exit $?
}

# Short, since the longest socket path of the instance is computed under it and bounded.
work="$(mktemp -d /tmp/w.XXX)"
mkdir -p "$work/bin" "$work/r" "$work/state" "$work/config/yoke"
export GOBIN="$work/bin" PATH="$work/bin:$PATH" XDG_RUNTIME_DIR="$work/r" XDG_STATE_HOME="$work/state" XDG_CONFIG_HOME="$work/config"
printf 'install_root: %s\n' "$work/apps" > "$work/config/yoke/manager.conf"

release="$(go list -m -f '{{.Version}}' github.com/yoke-project/yoke@latest)" || finish "FAIL  $CASE — the newest release of yoke cannot be read"
echo "watch: yoke $release"
echo "$release" > "$results/release"
for program in yoke yoke-compose yoke-core yoke-guardian yoke-auth; do
  go install "github.com/yoke-project/yoke/cmd/$program@$release" || finish "FAIL  $CASE — $program $release does not install"
done

# shellcheck source=/dev/null
source "$root/try/lib.sh" # start_app, $record, and the trap that stops the application
trap 'stop_all; rm -rf "$work"' EXIT

# std: yoke-plugin-video-simple:the-watch.01
# The device a Manifest needs is bound though the synthetic source never opens it.
start_app synthetic 1280x720 30 /dev/null > "$work/start.log" 2>&1 \
  || finish "FAIL  $CASE — the application did not come up under $release: $(tail -n 3 "$work/start.log" | tr '\n' ' ')"
for reads in ring socket; do
  said="$("$record" --instance "$instance" --channel viewer --reads "$reads" --seconds 5 2>&1)" \
    || finish "FAIL  $CASE — under $release, reading the $reads: $(tail -n 1 <<<"$said")"
  line="$(grep -E '^record: [0-9.]+ s, ' <<<"$said" | tail -n 1)"
  frames="$(sed -nE 's/^record: [0-9.]+ s, ([0-9]+) frames .*/\1/p' <<<"$line")"
  largest="$(sed -nE 's/.* ([0-9]+) KB largest.*/\1/p' <<<"$line")"
  (( ${frames:-0} >= 100 )) || finish "FAIL  $CASE — under $release, reading the $reads brought ${frames:-no} frames in 5 s: $line"
  (( ${largest:-0} >= 1800 )) || finish "FAIL  $CASE — under $release, reading the $reads brought no frame of 1.8 MB: $line"
  case "$reads" in
    ring) grep -q 'arriving on shared' <<<"$said" || finish "FAIL  $CASE — under $release the ring was not handed to a reader that offered it" ;;
    socket) grep -q 'arriving on stream_socket' <<<"$said" || finish "FAIL  $CASE — under $release the delivery was not a stream socket" ;;
  esac
done
finish "pass  $CASE"
