#!/bin/bash

cargo build --no-default-features --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web --weak-refs --reference-types ./target/wasm32-unknown-unknown/release/mapbuilder.wasm

rm -r web
mkdir web && cp favicon.png web && cp script.js web && cp -r out web && cp -r assets web

cd web;
ln -s ../index.html ./
python -m http.server



