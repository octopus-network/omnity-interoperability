#!/usr/bin/env bash

CANISTER=solana_route
CANISTER_WASM=target/wasm32-unknown-unknown/release/$CANISTER.wasm

# Build the canister
cargo build --release --target wasm32-unknown-unknown --package $CANISTER

# Extract the did file
echo "extractor did file ..."
candid-extractor $CANISTER_WASM > ./assets/$CANISTER.did

# optimize wasm file
# ic-wasm $CANISTER_WASM -o $CANISTER_WASM metadata candid:service -f $DID_PATH -v public

gzip --no-name --force $CANISTER_WASM
cp $CANISTER_WASM.gz ./assets/$CANISTER.wasm.gz

echo "Build done !"
