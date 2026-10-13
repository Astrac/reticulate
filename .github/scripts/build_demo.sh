#!/usr/bin/env bash
# Build the bevy demo for the web and drop it into the Zola static dir, where
# the Demo page loads it from (site/static/demo/). Requires the
# wasm32-unknown-unknown target, wasm-bindgen (matching Cargo.lock) and
# wasm-opt on PATH.
set -euo pipefail
cd "$(dirname "$0")/../.."

out=site/static/demo
rm -rf "$out"
mkdir -p "$out"

cargo build -p reticulate-bevy --profile release --target wasm32-unknown-unknown

wasm-bindgen --target web --no-typescript \
  --out-dir "$out" --out-name reticulate_bevy \
  target/wasm32-unknown-unknown/release/reticulate-bevy.wasm

wasm-opt -Oz \
  --enable-bulk-memory --enable-nontrapping-float-to-int \
  -o "$out/reticulate_bevy_bg.wasm" "$out/reticulate_bevy_bg.wasm"

# Bevy loads assets (shaders) relative to the page, i.e. /demo/assets/.
cp -r reticulate-bevy/assets "$out/assets"
