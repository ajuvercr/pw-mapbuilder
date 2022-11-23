#!/bin/bash

cargo build --no-default-features --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./web/out/ --target web --weak-refs --reference-types ./target/wasm32-unknown-unknown/release/mapbuilder.wasm

rm -r web/assets
cp -r assets web

cd web;
python -m http.server

