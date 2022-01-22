#!/bin/bash
set -e
rustup target add wasm32-unknown-unknown
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cd ..
cp target/wasm32-unknown-unknown/release/session_vault.wasm ./res/session_vault_local.wasm
cd -