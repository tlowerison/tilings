#!/bin/sh
set -e

CONFIG=release
mkdir -p www/pkg

cd rust/client
rustup target add wasm32-unknown-unknown

if [ -z "$(which wasm-pack)" ]
then
	cargo install wasm-pack
fi

cd wasm
wasm-pack build --out-dir ../../../www/pkg --release

cd ../../../www
yarn add ./pkg
yarn
yarn start
