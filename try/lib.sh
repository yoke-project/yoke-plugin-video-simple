# Sourced by the scripts beside it: starts the application on a local Core with one camera setting,
# grants what video-simple declares, and stops everything when the calling script exits.
#
#   start_app <mjpeg|yuyv|synthetic> <WxH> <fps> [device]     sets $instance and exports YOKE_INSTANCE

try_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
record="$try_dir/record/target/release/record"

stop_all() {
  [[ -n "${front_pid:-}" ]] && kill "$front_pid" 2>/dev/null
  if [[ -n "${run_pid:-}" ]]; then
    kill -INT "$run_pid" 2>/dev/null
    wait "$run_pid" 2>/dev/null
  fi
  [[ -n "${run_log:-}" ]] && rm -f "$run_log"
  run_pid="" front_pid=""
}
trap stop_all EXIT

# The Plugin as the application carries it: built from ../plugin, its program and Manifest copied in and
# the unit's digest written, whenever the program carried is not the one just built. A fresh clone carries
# no program, since components/ is built output.
provision_plugin() {
  local built="$try_dir/../plugin/target/release/video-simple" carried="$try_dir/components/plugins/video-simple"
  cargo build --release --quiet --manifest-path "$try_dir/../plugin/Cargo.toml"
  if ! cmp -s "$built" "$carried"; then
    mkdir -p "$(dirname "$carried")" "$try_dir/manifests"
    cp "$built" "$carried"
    cp "$try_dir/../plugin/com.yoke.video-simple.yaml" "$try_dir/manifests/"
  fi
  # Written at every run: the description kept in the repository names whatever was built last.
  yoke-compose set units.video-simple.digest "sha256:$(sha256sum "$carried" | cut -d' ' -f1)" --dir "$try_dir" > /dev/null
}

start_app() {
  local format="$1" size="$2" fps="$3" device="${4:-/dev/video0}"
  local source=camera
  [[ "$format" == synthetic ]] && source=synthetic format=yuyv
  provision_plugin
  [[ -x "$record" ]] || cargo build --release --manifest-path "$try_dir/record/Cargo.toml"
  yoke-compose set units.video-simple.bind.camera "$device" --dir "$try_dir" > /dev/null
  yoke-compose set units.video-simple.args \
    "[\"--device\", \"\${bind.camera}\", \"--format\", \"$format\", \"--source\", \"$source\", \"--size\", \"$size\", \"--fps\", \"$fps\"]" \
    --dir "$try_dir" > /dev/null
  run_log="$(mktemp)"
  yoke app run --from "$try_dir" > "$run_log" 2>&1 &
  run_pid=$!
  until grep -q 'is observable' "$run_log"; do
    kill -0 "$run_pid" 2>/dev/null || { cat "$run_log" >&2; return 1; }
    sleep 0.5
  done
  instance="$(sed -n '1s/ runs .*//p' "$run_log")"
  export YOKE_INSTANCE="$instance"
  # A run's instance starts with nothing granted; a grant reaches the unit at its next admission.
  yoke plugin grant com.yoke.video-simple stream.video.frames.publish > /dev/null
  yoke plugin grant com.yoke.video-simple query.video.format.answer > /dev/null
  yoke unit restart video-simple > /dev/null
  sleep 1
}

# The front of the page, in front of the web channel. Sets $front_pid.
start_front() {
  local listen="${1:-127.0.0.1:8080}"
  node "$try_dir/web/server.mjs" --channel http://127.0.0.1:47911 --listen "$listen" > /dev/null 2>&1 &
  front_pid=$!
  until curl -sf -o /dev/null "http://$listen/"; do
    kill -0 "$front_pid" 2>/dev/null || { echo "the front did not start" >&2; return 1; }
    sleep 0.3
  done
}

# What the unit said while it streamed: its own count of what it emitted.
unit_said() {
  yoke log query --unit video-simple 2>/dev/null | grep -E 'video-simple: ' | sed 's/^[^ ]* //' | tail -"${1:-6}"
}
