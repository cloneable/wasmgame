#!/bin/sh

rm -rf wbg
mkdir wbg

cargo build --release --target=wasm32-unknown-unknown
wasm-bindgen --target web --out-dir wbg --no-typescript ./target/wasm32-unknown-unknown/release/wasmgame.wasm
wasm-opt -O4 -o wbg/wasmgame_bg_opt.wasm wbg/wasmgame_bg.wasm
wasm-strip wbg/wasmgame_bg_opt.wasm
mv wbg/wasmgame_bg_opt.wasm wbg/wasmgame_bg.wasm
cp -f wbg/wasmgame_bg.wasm web/wasmgame_bg.wasm
cp -f wbg/wasmgame.js web/wasmgame.js
