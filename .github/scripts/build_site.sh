#!/usr/bin/env bash
# Build the docusite and pre-render (SSG) all its static routes into
# target/dx/reticulate-docusite/release/web/public.
#
# Usage: build_site.sh [base_path]
#
#   base_path  Sub-path the site is served from, e.g. "reticulate" for
#              https://<host>/reticulate/. Empty (default) means the root.
#              It is written to Dioxus.toml for the duration of the build and
#              restored afterwards, so the committed file is never changed.
#
# `dx build --ssg` requests /api/static_routes from the server it starts, but
# when a base_path is set the server mounts everything under that prefix, the
# request 404s and the build fails. So we build the app, then run the server and
# request each route ourselves; the server's incremental renderer writes each
# page to its public dir.
set -euo pipefail
cd "$(dirname "$0")/../.."

base=${1:-}
base=${base#/}
base=${base%/}
prefix=${base:+/$base}
config=reticulate-docusite/Dioxus.toml
web=target/dx/reticulate-docusite/release/web
addr=127.0.0.1:9991

server=
backup=$(mktemp)
cp "$config" "$backup"
cleanup() {
  [ -z "$server" ] || kill "$server" 2>/dev/null || true
  cp "$backup" "$config"
  rm -f "$backup"
}
trap cleanup EXIT

if [ -n "$base" ]; then
  # Drop any existing base_path and set ours right under [web.app].
  awk -v base="$base" '
    /^base_path[ \t]*=/ { next }
    { print }
    /^\[web\.app\]/ { print "base_path = \"" base "\"" }
  ' "$backup" > "$config"
fi

dx build -p reticulate-docusite --release

IP=127.0.0.1 PORT=9991 "$web/server" &
server=$!

url="http://$addr$prefix/api/static_routes"
routes=
for _ in $(seq 1 30); do
  routes=$(curl -sf -X POST -H 'content-type: application/json' -d '{}' "$url") && break
  sleep 1
done
[ -n "$routes" ] || { echo "no response from $url" >&2; exit 1; }

echo "$routes" | python3 -c 'import json,sys; print("\n".join(json.load(sys.stdin)))' |
  while read -r route; do
    echo "Rendering $prefix$route"
    curl -sf -o /dev/null "http://$addr$prefix$route"
  done
