#!/bin/sh

# $ cargo install wasm-bindgen-cli twiggy devserver
# MacOS: $ brew install wabt binaryen sccache

set -euo pipefail

BUILD_TYPE=debug
BUILD_FLAGS=
BUILD_OPT=false
BUILD_FEATURES=console_error_panic_hook,wee_alloc

case "$1" in
  --release)
    BUILD_TYPE=release
    BUILD_FLAGS=--release
    BUILD_OPT=true
    BUILD_FEATURES=wee_alloc
    ;;
  --debug | "")
    ;;
  *)
    echo "Invalid flag: $1"
    exit 1
    ;;
esac

(cd app && cargo --offline build --features="$BUILD_FEATURES" $BUILD_FLAGS --target=wasm32-unknown-unknown)
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
