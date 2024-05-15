#!/bin/bash

cargo build --target wasm32-unknown-unknown --release -p bitcoin_customs --locked --features non_prod
ic-wasm target/wasm32-unknown-unknown/release/bitcoin_customs.wasm -o bitcoin_customs.wasm shrink
