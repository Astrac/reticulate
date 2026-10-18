#!/usr/bin/env bash
# Build the bevy demo for the web into <out-dir> (default: demo-dist). The docs
# site loads reticulate_bevy.js / reticulate_bevy_bg.wasm relative to the page,
# and bevy fetches its shaders from assets/. Requires the wasm32-unknown-unknown
# target, wasm-bindgen (matching Cargo.lock) and wasm-opt on PATH.
set -euo pipefail
cd "$(dirname "$0")/../.."

out=${1:-demo-dist}
rm -rf "$out"
mkdir -p "$out"

cargo build -p reticulate-bevy --profile release --target wasm32-unknown-unknown

wasm-bindgen --target web --no-typescript \
  --out-dir "$out" --out-name reticulate_bevy \
  target/wasm32-unknown-unknown/release/reticulate-bevy.wasm

wasm-opt -Oz \
  --enable-bulk-memory --enable-nontrapping-float-to-int \
  -o "$out/reticulate_bevy_bg.wasm" "$out/reticulate_bevy_bg.wasm"

cp -r reticulate-bevy/assets "$out/assets"
