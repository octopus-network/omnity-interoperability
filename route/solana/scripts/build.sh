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

# gzip --no-name --force $CANISTER_WASM
# cp $CANISTER_WASM.gz ./assets/$CANISTER.wasm.gz
dfx canister create solana_route
dfx build solana_route
cp ./.dfx/local/canisters/solana_route/solana_route.wasm.gz ./assets/solana_route.wasm.gz
cp ./.dfx/local/canisters/solana_route/service.did.d.ts ./assets/service.did.d.ts
cp ./.dfx/local/canisters/solana_route/service.did.js ./assets/service.did.js

echo "Build done !"
