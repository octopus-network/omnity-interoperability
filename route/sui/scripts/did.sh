#!/usr/bin/env bash

# function generate_did() {
#   local canister=$1
#   canister_root="src/$canister"

#   cargo build --manifest-path="$canister_root/Cargo.toml" \
#       --target wasm32-unknown-unknown \
#       --release --package "$canister"

#   candid-extractor "target/wasm32-unknown-unknown/release/$canister.wasm" > "$canister_root/$canister.did"
# }

# # The list of canisters of your project
# CANISTERS=$1

# for canister in $(echo $CANISTERS | sed "s/,/ /g")
# do
#     generate_did "$canister"
# done

CANISTER=sui_route
CANISTER_WASM=target/wasm32-unknown-unknown/release/$CANISTER.wasm

# Build the canister
cargo build --release --target wasm32-unknown-unknown --package $CANISTER

# Extract the did file
echo "extractor did file ..."
candid-extractor $CANISTER_WASM > ./assets/$CANISTER.did
# cp ./assets/$CANISTER.did ./

# optimize wasm file
# ic-wasm $CANISTER_WASM -o $CANISTER_WASM metadata candid:service -f $DID_PATH -v public

# gzip --no-name --force $CANISTER_WASM
# cp $CANISTER_WASM.gz ./assets/$CANISTER.wasm.gz