#!/usr/bin/env bash


export DFX_WARNING="-mainnet_plaintext_identity"
CANISTER=sui_route
CANISTER_WASM=target/wasm32-unknown-unknown/release/$CANISTER.wasm

# Build the canister
cargo build --release --target wasm32-unknown-unknown --package $CANISTER

# Extract the did file
echo "extractor did file ..."
candid-extractor $CANISTER_WASM > ./assets/$CANISTER.did

# dfx canister create sui_route
dfx build sui_route
cp ./.dfx/local/canisters/sui_route/sui_route.wasm.gz ./assets/sui_route.wasm.gz
cp ./.dfx/local/canisters/sui_route/service.did ./assets/sui_route.did
cp ./.dfx/local/canisters/sui_route/service.did.d.ts ./assets/service.did.d.ts
cp ./.dfx/local/canisters/sui_route/service.did.js ./assets/service.did.js

echo "Build done !"
