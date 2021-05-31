#!/bin/bash
set -e

CONFIG=release
mkdir -p www/pkg

rustup target add wasm32-unknown-unknown

if [ -z "$(which wasm-pack)" ]
then
	cargo install wasm-pack
fi

cd wasm
if [ "${CONFIG}" = "release" ]; then
  wasm-pack build --out-dir ../www/pkg --release
else
  wasm-pack build --out-dir ../www/pkg
fi

cd ../www
yarn add ./pkg && yarn && yarn start
