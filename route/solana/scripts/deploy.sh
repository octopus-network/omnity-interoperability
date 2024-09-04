#!/usr/bin/env bash

# get admin id
# ADMIN="rv3oc-smtnf-i2ert-ryxod-7uj7v-j7z3q-qfa5c-bhz35-szt3n-k3zks-fqe"
ADMIN=$(dfx identity get-principal)
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
# SCHNORR_CANISTER_ID=aaaaa-aa
dfx deploy schnorr_canister --mode=reinstall -y
SCHNORR_CANISTER_ID=$(dfx canister id schnorr_canister)
echo "Schnorr canister id: $SCHNORR_CANISTER_ID"
echo 

# Deploy the solana canister and set the schnorr canister id
# SOLANA_RPC_URL="devnet"
# SOLANA_RPC_URL="https://solana-mainnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
# SOLANA_RPC_URL="https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
# SOLANA_RPC_URL="http://localhost:8888/"
SOLANA_RPC_URL=https://solana-rpc-proxy-398338012986.us-central1.run.app/
SCHNORR_KEY_NAME="test_key_1"
dfx deploy ic-solana-provider --argument "( record { 
    nodesInSubnet = 28; 
    schnorr_canister = opt \"${SCHNORR_CANISTER_ID}\"; 
    schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
    rpc_url = opt \"${SOLANA_RPC_URL}\"; 
    } )" --mode=reinstall -y
SOL_PROVIDER_CANISTER_ID=$(dfx canister id ic-solana-provider)
echo "solana provide canister id: $SOL_PROVIDER_CANISTER_ID"
echo 

CHAIN_ID="Solana"
FEE_ACCOUNT="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
# Deploy solana_route
dfx deploy solana_route --argument "(variant { Init = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_canister = opt principal \"${SCHNORR_CANISTER_ID}\";\
    schnorr_key_name = null; \
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
    fee_account= opt \"${FEE_ACCOUNT}\"; 
} })" \
--mode=reinstall -y

SOLANA_ROUTE_CANISTER_ID=$(dfx canister id solana_route)
echo "Solana route canister id: $SOLANA_ROUTE_CANISTER_ID"

echo "Deploy done!"