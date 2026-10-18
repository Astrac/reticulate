#!/usr/bin/env bash
# Build the docusite and pre-render (SSG) all its static routes into
# target/dx/reticulate-docusite/release/web/public.
#
# `dx build --ssg` requests /api/static_routes from the server it starts, but
# when Dioxus.toml sets `base_path` the server mounts everything under that
# prefix, the request 404s and the build fails. So we build the app, then run
# the server and request each route ourselves; the server's incremental
# renderer writes each page to its public dir.
#
# Usage: build_site.sh [base_path]   (no leading slash, e.g. "reticulate")
set -euo pipefail
cd "$(dirname "$0")/../.."

base=${1:-}
prefix=${base:+/$base}
web=target/dx/reticulate-docusite/release/web
addr=127.0.0.1:9991

dx build -p reticulate-docusite --release

IP=127.0.0.1 PORT=9991 "$web/server" &
server=$!
trap 'kill "$server" 2>/dev/null || true' EXIT

for _ in $(seq 1 30); do
  routes=$(curl -sf -X POST -H 'content-type: application/json' -d '{}' "http://$addr$prefix/api/static_routes") && break
  sleep 1
done
[ -n "${routes:-}" ] || { echo "server did not return static routes" >&2; exit 1; }

echo "$routes" | python3 -c 'import json,sys; print("\n".join(json.load(sys.stdin)))' |
  while read -r route; do
    echo "Rendering $route"
    curl -sf -o /dev/null "http://$addr$prefix$route"
  done
