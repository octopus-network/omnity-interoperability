#!/bin/bash

if [ "$DFX_NETWORK" = "ic" ]; then
    echo "DFX_NETWORK=ic"
    cargo build --target wasm32-unknown-unknown --release -p bitcoin_customs --locked
else
    echo "DFX_NETWORK=local"
    cargo build --target wasm32-unknown-unknown --release -p bitcoin_customs --locked --features non_prod
fi
ic-wasm target/wasm32-unknown-unknown/release/bitcoin_customs.wasm -o bitcoin_customs.wasm shrink
