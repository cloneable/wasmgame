#!/bin/sh

set -e

BUILD_TYPE=debug
BUILD_FLAGS=
BUILD_OPT=false
if [ "$1" == "--release" ]; then
  BUILD_TYPE=release
  BUILD_FLAGS=--release
  BUILD_OPT=true
fi

cargo build $BUILD_FLAGS --target=wasm32-unknown-unknown
rm -rf wbg
mkdir wbg
wasm-bindgen --target web --out-dir wbg --no-typescript ./target/wasm32-unknown-unknown/"$BUILD_TYPE"/wasmgame.wasm
if [ "$BUILD_OPT" == true ]; then
  wasm-opt -O4 -o wbg/wasmgame_bg_opt.wasm wbg/wasmgame_bg.wasm
  wasm-strip wbg/wasmgame_bg_opt.wasm
  mv wbg/wasmgame_bg_opt.wasm wbg/wasmgame_bg.wasm
fi
cp -f wbg/wasmgame_bg.wasm web/wasmgame_bg.wasm
cp -f wbg/wasmgame.js web/wasmgame.js
