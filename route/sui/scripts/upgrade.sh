#!/bin/bash

export DFX_WARNING="-mainnet_plaintext_identity"
# config network
NETWORK=local

ADMIN=$(dfx identity get-principal)
echo "admin id: $ADMIN"
echo 

# upgrade  hub
# echo deploy omnity_hub ...
# dfx deploy omnity_hub --argument "(variant { Init = record { 
#     admin = principal \"${ADMIN}\" 
#     } })" --mode=upgrade -y
HUB_CANISTER_ID=$(dfx canister id omnity_hub --network $NETWORK)
echo "Omnity hub canister id: $HUB_CANISTER_ID"
# dfx canister status omnity_hub  
# echo 

# SUI_TESTNET_RPC="https://fullnode.testnet.sui.io:443"
# SUI_MAINNET_RPC="https://fullnode.testnet.sui.io:443"

SCHNORR_KEY_NAME="dfx_test_key"
# SCHNORR_KEY_NAME="test_key_1"
# SCHNORR_KEY_NAME="key_1"

SUI_CHAIN_ID="eSui"
FEE_ACCOUNT="0xaf9306cac62396be300b175046140c392eed876bd8ac0efac6301cea286fa272"
nodes_in_subnet=34
provider=Testnet
gas_budget=10000000
echo upgrade sui_route ...

# upgrade with params
dfx deploy sui_route --mode upgrade --argument "( variant { Upgrade = opt record {
    admin = opt principal \"${ADMIN}\";
    chain_id = opt \"${SUI_CHAIN_ID}\";
    hub_principal = opt principal \"${HUB_CANISTER_ID}\";
    chain_state= opt variant { Active }; 
    schnorr_key_name = opt \"${SCHNORR_KEY_NAME}\";
    rpc_provider = opt variant { $provider };
    nodes_in_subnet = opt ${nodes_in_subnet};
    fee_account = opt \"${FEE_ACCOUNT}\"; 
    gas_budget = opt ${gas_budget} ;
    } })"  --yes  --network $NETWORK

# upgrade without params
# dfx deploy --mode upgrade --argument '(variant { Upgrade = null })'  --upgrade-unchanged --yes sui_route --network $NETWORK

dfx canister status sui_route --network $NETWORK
# dfx canister call sui_route stop_schedule '(null)' --network $NETWORK

echo "Upgrade sui route done!"
