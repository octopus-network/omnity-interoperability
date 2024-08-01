#!/usr/bin/env bash

# get admin id
ADMIN="rv3oc-smtnf-i2ert-ryxod-7uj7v-j7z3q-qfa5c-bhz35-szt3n-k3zks-fqe"
echo "admin id: $ADMIN"
echo 


# Deploy hub
# dfx canister create omnity_hub
dfx deploy omnity_hub --argument "(variant { Init = record { admin = principal \"${ADMIN}\" } })" --mode=reinstall -y
HUB_CANISTER_ID=$(dfx canister id omnity_hub)
echo "Omnity hub canister id: $HUB_CANISTER_ID"
echo 

# TODO: deploy customs

# Deploy schnorr canister
dfx deploy schnorr_canister --mode=reinstall -y
SCHNORR_CANISTER_ID=$(dfx canister id schnorr_canister)
echo "Schnorr canister id: $SCHNORR_CANISTER_ID"
echo 

# Deploy the solana canister and set the schnorr canister id
dfx deploy ic-solana-provider --argument "( record { nodesInSubnet = 28; schnorr_canister = opt \"${SCHNORR_CANISTER_ID}\" } )" --mode=reinstall -y
SOL_PROVIDER_CANISTER_ID=$(dfx canister id ic-solana-provider)
echo "solana provide canister id: $SOL_PROVIDER_CANISTER_ID"
echo 

CHAIN_ID="Solana"
# Deploy solana_route
dfx deploy solana_route --argument "(variant { Init = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_canister = principal \"${SCHNORR_CANISTER_ID}\";\
    schnorr_key_name = null; \
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
} })" \
--mode=reinstall -y

SOLANA_ROUTE_CANISTER_ID=$(dfx canister id solana_route)
echo "Solana route canister id: $SOLANA_ROUTE_CANISTER_ID"

echo "Deploy done!"