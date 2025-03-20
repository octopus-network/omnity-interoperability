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
# dfx deploy schnorr_canister --mode=reinstall -y
# SCHNORR_CANISTER_ID=$(dfx canister id schnorr_canister)
# echo "Schnorr canister id: $SCHNORR_CANISTER_ID"
# echo 

# Deploy the solana canister and set the schnorr canister id
# SOLANA_RPC_URL="devnet"

PROXY_URL="https://solana-idempotent-proxy-219952077564.us-central1.run.app/api"
ALCHEMY_RPC_URL="https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
HELIUS_RPC_URL="https://devnet.helius-rpc.com/?api-key=b7fe7483-b790-427e-af31-0095d7f73d4e"
SNOW_RPC_URL="https://sol.nownodes.io"

# SOLANA_RPC_URL="http://localhost:8888/"
# SOLANA_RPC_URL=https://solana-rpc-proxy-398338012986.us-central1.run.app/


dfx deploy ic-solana-rpc --argument '(record {})' --mode=reinstall -y
SOL_PROVIDER_CANISTER_ID=$(dfx canister id ic-solana-rpc)
echo "solana provide canister id: $SOL_PROVIDER_CANISTER_ID"
echo 
dfx canister status $SOL_PROVIDER_CANISTER_ID 
# test rpc api 
dfx canister call ic-solana-rpc sol_getHealth '(variant{Devnet},null)' --wallet $(dfx identity get-wallet) --with-cycles 674369600
dfx canister call ic-solana-rpc sol_getHealth "(variant{Custom=vec{record{network=\"${ALCHEMY_RPC_URL}\"}}},null)" --wallet $(dfx identity get-wallet) --with-cycles 674369600
dfx canister call ic-solana-rpc sol_getLatestBlockhash "(variant{Custom=vec{record{network=\"${ALCHEMY_RPC_URL}\"}}},null,null)" --wallet $(dfx identity get-wallet) --with-cycles 676028800
dfx canister call ic-solana-rpc sol_getLatestBlockhash "(variant{Custom=vec{record{network=\"${HELIUS_RPC_URL}\"}}},null,null)" --wallet $(dfx identity get-wallet) --with-cycles 676028800
dfx canister call ic-solana-rpc sol_getLatestBlockhash "(variant{
        Custom=vec{
            record{
                network=\"${SNOW_RPC_URL}\";
                headers=opt vec{record{name=\"api-key\"; value=\"5a89f7b5-4679-4ac3-a516-000b64ed0bc8\";}};
                }
        }
    },
    null,
    null)" --wallet $(dfx identity get-wallet) --with-cycles 676028800


# echo 

CHAIN_ID="eSolana"
FEE_ACCOUNT="FDR2mUpiHKFonnwbUujLyhuNTt7LHEjZ1hDFX4UuCngt"
# rpc1=https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ
# rpc2=https://rpc.ankr.com/solana_devnet/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
# rpc3=https://nd-471-475-490.p2pify.com/6de0b91c609fb3bd459e043801aa6aa4

# Deploy solana_route
SCHNORR_KEY_NAME="dfx_test_key"
SOLANA_RPC_HOST="api.devnet.solana.com"
ALCHEMY_RPC_HOST="solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
HELIUS_RPC_HOST="devnet.helius-rpc.com"
HELIUS_API_KEY="api-key=b7fe7483-b790-427e-af31-0095d7f73d4e"
SNOW_RPC_HOST="sol.nownodes.io"
# SOLANA_RPC_HOST="solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
dfx deploy solana_route --argument "(variant { Init = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_key_name = opt \"${SCHNORR_KEY_NAME}\"; \
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
    fee_account= opt \"${FEE_ACCOUNT}\"; 
    providers= vec{
        record{host=\"${HELIUS_RPC_HOST}\"; api_key_param=opt \"${HELIUS_API_KEY}\"; headers=null;};
        record{host=\"${ALCHEMY_RPC_HOST}\"; api_key_param=null; headers=null;};
        record{host=\"${SOLANA_RPC_HOST}\"; api_key_param=null; headers=null;}; 
         record{host=\"${SNOW_RPC_HOST}\"; 
                api_key_param=null; 
                headers=opt vec{record{name=\"api-key\"; value=\"5a89f7b5-4679-4ac3-a516-000b64ed0bc8\";}};
                }; 
        };
    proxy= \"${PROXY_URL}\";
    minimum_response_count=2:nat32;
} })" \
--mode=reinstall -y


# dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument '(null)' \
#     --mode=upgrade -y \

dfx deploy solana_route --argument '(variant { Upgrade = null})' --mode=upgrade -y 

SOLANA_ROUTE_CANISTER_ID=$(dfx canister id solana_route)
echo "Solana route canister id: $SOLANA_ROUTE_CANISTER_ID"
dfx canister status $SOLANA_ROUTE_CANISTER_ID 
dfx canister call $SOLANA_ROUTE_CANISTER_ID provider '()'
dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '(variant { ChainKey })' 
dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '(variant { Native })' 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_latest_blockhash '()'
account="FDR2mUpiHKFonnwbUujLyhuNTt7LHEjZ1hDFX4UuCngt"
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_account_info "(\"${account}\")"
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_balance "(\"${account}\")"

sig="JScFmwgn2LNW6UcmLFnBZpWHnkANLftGC7unD7T58TU2DntyvkQkrRiZpjHJzdqMDF91YxharucP8uZhM28GZhJ"
# dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${sig}\")" 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_raw_transaction "(\"${sig}\")" 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_signature_status "(vec {\"${sig}\"})" 


echo "Deploy done!"