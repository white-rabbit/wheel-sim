#!/usr/bin/env zsh

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
    --out-dir ./wasm/ \
    --out-name "reinventing_the_wheel" \
    ./target/wasm32-unknown-unknown/release/reinventing_the_wheel.wasm
