#!/usr/bin/env bash

# get admin id
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
# dfx deploy schnorr_canister --mode=reinstall -y
# SCHNORR_CANISTER_ID=$(dfx canister id schnorr_canister)
# echo "Schnorr canister id: $SCHNORR_CANISTER_ID"
# echo 

# Deploy the solana canister and set the schnorr canister id
SOLANA_RPC_URL="devnet"

SCHNORR_KEY_NAME="dfx_test_key"
# SCHNORR_KEY_NAME="test_key_1"
dfx deploy ic-solana-provider --argument "( record { 
    schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
    rpc_url = opt \"${SOLANA_RPC_URL}\"; 
    nodesInSubnet = 28; 
    } )" --mode=reinstall -y
SOL_PROVIDER_CANISTER_ID=$(dfx canister id ic-solana-provider)
echo "solana provide canister id: $SOL_PROVIDER_CANISTER_ID"
echo 
dfx canister status $SOL_PROVIDER_CANISTER_ID 
# test canister api
ankr=testnet
test_account=3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia
test_sig=4e1gA4YvTt95DYY5kdwSWpGr2oiMqRX2nk4XenF1aiJSz69cbLBMeTfV6HG4jG7jHtdcHwwjGCSw5zepgpC8n5g7
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_latestBlockhash "(opt \"${ankr}\")" 
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getAccountInfo "(\"${test_account}\",opt \"${ankr}\")" 
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getSignatureStatuses "(vec {\"${test_sig}\"},opt \"${ankr}\")"
echo 

CHAIN_ID="eSolana"
FEE_ACCOUNT="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"

# Deploy solana_route
dfx deploy solana_route --argument "(variant { Init = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_key_name = opt \"${SCHNORR_KEY_NAME}\"; \
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
    fee_account= opt \"${FEE_ACCOUNT}\"; 
} })" \
--mode=reinstall -y

SOLANA_ROUTE_CANISTER_ID=$(dfx canister id solana_route)
echo "Solana route canister id: $SOLANA_ROUTE_CANISTER_ID"
dfx canister status $SOLANA_ROUTE_CANISTER_ID 
dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_latest_blockhash '()'
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${test_sig}\",opt \"${ankr}\")" 

# update_multi_rpc 
rpc1=devnet1
rpc2=devnet2

dfx canister call $SOLANA_ROUTE_CANISTER_ID update_multi_rpc "(record { 
    rpc_list = vec {\"${rpc1}\";
                     \"${rpc2}\";};\
    minimum_response_count = 2:nat32;})"
dfx canister call $SOLANA_ROUTE_CANISTER_ID multi_rpc_config '()'

echo "Deploy done!"