#!/usr/bin/env bash

# disable dfx warning
export DFX_WARNING="-mainnet_plaintext_identity"

echo "Setting up for production environment..."
ADMIN=$(dfx identity get-principal --ic)

# Production env
HUB_CANISTER_ID=7wupf-wiaaa-aaaar-qaeya-cai
SOL_PROVIDER_CANISTER_ID=l3ka6-4yaaa-aaaar-qahpa-cai
SOLANA_ROUTE_CANISTER_ID=lvinw-hiaaa-aaaar-qahoa-cai

SCHNORR_KEY_NAME="key_1"
PROXY_URL="https://solana-rpc-proxy-398338012986.us-central1.run.app"
alchemy_m=https://solana-mainnet.g.alchemy.com/v2/t25IzpcIjBXhP-LOurqrTWLWmhPuBwsk
helius_m="https://mainnet.helius-rpc.com/?api-key=174a6ec2-4439-4fca-9277-b12900c71fa5"
snownodes=https://sol.nownodes.io
triton_m=https://png.rpcpool.com/13a5c61c672e6cd88357abf3709a
ankr_m=https://rpc.ankr.com/solana/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3

echo "product environment: 
    admin id: $ADMIN
    omnity_hub canister id: $HUB_CANISTER_ID 
    schnorr key name: $SCHNORR_KEY_NAME 
    proxy url: $PROXY_URL
    ic solana provider canister id: $SOL_PROVIDER_CANISTER_ID
    solana route canister id: $SOLANA_ROUTE_CANISTER_ID"

###########################################################################
### install/reinstall/upgrade/config solana provider canister and solana route
###########################################################################

# install /reinstall/upgrade ic provider canister
echo "reinstall $SOL_PROVIDER_CANISTER_ID ..."
dfx canister install $SOL_PROVIDER_CANISTER_ID --argument "( record { 
    rpc_url = opt \"${PROXY_URL}\"; 
    schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
    nodesInSubnet = opt 34; 
    } )" \
    --mode=reinstall -y \
    --wasm=./assets/ic_solana_provider.wasm.gz \
    --ic 

# echo "upgrade $SOL_PROVIDER_CANISTER_ID ..."
# dfx canister install $SOL_PROVIDER_CANISTER_ID --argument "( record { 
#     rpc_url = opt \"${PROXY_URL}\"; 
#     schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
#     nodesInSubnet = opt 28; 
#     } )" \
#     --mode=upgrade -y \
#     --wasm=./assets/ic_solana_provider.wasm.gz \
#     --ic 

dfx canister status $SOL_PROVIDER_CANISTER_ID --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID debug '(true)' --ic
# check canister api
nownodes=https://sol.nownodes.io
ankr_m=https://rpc.ankr.com/solana/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
test_account=3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia
test_sig=4e1gA4YvTt95DYY5kdwSWpGr2oiMqRX2nk4XenF1aiJSz69cbLBMeTfV6HG4jG7jHtdcHwwjGCSw5zepgpC8n5g7
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_latestBlockhash "(opt \"${ankr_m}\")" --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_latestBlockhash "(opt \"${nownodes}\")" --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getAccountInfo "(\"${test_account}\",opt \"${ankr_m}\")" --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getAccountInfo "(\"${test_account}\",opt \"${nownodes}\")" --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getSignatureStatuses "(vec {\"${test_sig}\"},opt \"${ankr_m}\")" --ic
echo 

# solana_route canister
SOL_CHAIN_ID="eSolana"
SOL_FEE="SOL"
FEE_ACCOUNT="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
ankr_m=https://rpc.ankr.com/solana/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
# rpc1=https://solana-mainnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ
# rpc3=https://nd-471-475-490.p2pify.com/6de0b91c609fb3bd459e043801aa6aa4

# echo "reinstall $SOLANA_ROUTE_CANISTER_ID ..."
# dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument "(variant { Init = record { \
#     admin = principal \"${ADMIN}\";\
#     chain_id=\"${SOL_CHAIN_ID}\";\
#     hub_principal= principal \"${HUB_CANISTER_ID}\";\
#     chain_state= variant { Active }; \
#     schnorr_key_name = \"${SCHNORR_KEY_NAME}\";\
#     sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
#     fee_account= opt \"${FEE_ACCOUNT}\";\
#     } })" \
#     --mode=reinstall -y \
#     --wasm=./assets/solana_route.wasm.gz \
#     --ic 

# echo "upgrade $SOLANA_ROUTE_CANISTER_ID ..."
# dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument "(opt variant { Upgrade = record { \
#     admin = principal \"${ADMIN}\";\
#     chain_id=\"${SOL_CHAIN_ID}\";\
#     hub_principal= principal \"${HUB_CANISTER_ID}\";\
#     chain_state= variant { Active }; \
#     schnorr_key_name = \"${SCHNORR_KEY_NAME}\";\
#     sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
#     fee_account= opt \"${FEE_ACCOUNT}\";\
#     } })" \
#     --mode=upgrade -y \
#     --wasm=./assets/solana_route.wasm.gz \
#     --ic 
echo "upgrade $SOLANA_ROUTE_CANISTER_ID ..."
dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument '(null)' \
    --mode=upgrade -y \
    --wasm=./assets/solana_route.wasm.gz \
    --ic 

dfx canister status $SOLANA_ROUTE_CANISTER_ID --ic

# add perms
# dfx canister call $SOLANA_ROUTE_CANISTER_ID set_permissions "(
#     principal \"kp4gp-pefsb-gau5l-p2hf6-pagac-3jusw-lzc2v-nsxtq-46dnk-ntffe-3qe\",\
#     variant { Update }
#     )" \
#     --ic 
# check 
dfx canister call $SOLANA_ROUTE_CANISTER_ID debug '(true)' --ic
KEYTYPE="variant { ChainKey }"
dfx canister call $SOLANA_ROUTE_CANISTER_ID signer "($KEYTYPE)"  --ic
# dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID multi_rpc_config '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID forward '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_latest_blockhash '()' --ic 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction '("4kogo438gk3CT6pifHQa7d4CC7HRidnG2o6EWxwGFvAcuSC7oTeG3pWTYDy9wuCYmGxJe1pRdTHf7wMcnJupXSf4",null)' --ic


# update schnorr info
# dfx canister call $SOLANA_ROUTE_CANISTER_ID update_schnorr_key '("key_1")' --ic

###########################################################################
### update multi_rpc_config
###########################################################################

alchemy_m=https://solana-mainnet.g.alchemy.com/v2/t25IzpcIjBXhP-LOurqrTWLWmhPuBwsk
helius_m="https://mainnet.helius-rpc.com/?api-key=174a6ec2-4439-4fca-9277-b12900c71fa5"
snownodes=https://sol.nownodes.io
triton_m=https://png.rpcpool.com/13a5c61c672e6cd88357abf3709a
ankr_m=https://rpc.ankr.com/solana/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
SIG=334PcrvBjAcjqMubimWAjy6Gsh8wDa57xw4yaFdhEa1L8qux2C9qyzrKRxTQCsfGoJGudLGWz3fQhnfQA8VvqenE

dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIG}\",opt \"${triton_m}\")" --ic --output json | jq '.Ok | fromjson'
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIG}\",opt \"${alchemy_m}\")" --ic --output json | jq '.Ok | fromjson'
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIG}\",opt \"${helius_m}\")" --ic --output json | jq '.Ok | fromjson'
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIG}\",opt \"${snownodes}\")" --ic --output json | jq '.Ok | fromjson'
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIG}\",null)" --ic --output json | jq '.Ok | fromjson'


dfx canister call $SOLANA_ROUTE_CANISTER_ID multi_rpc_config '()' --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID update_multi_rpc "(record { 
    rpc_list = vec {\"${alchemy_m}\";\"${triton_m}\"};\
    minimum_response_count = 1:nat32;})" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID update_multi_rpc "(record { 
    rpc_list = vec {\"${alchemy_m}\";\"${helius_m}\"};\
    minimum_response_count = 1:nat32;})" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID update_multi_rpc "(record { 
    rpc_list = vec {\"${alchemy_m}\";\"${snownodes}\"};\
    minimum_response_count = 1:nat32;})" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID update_multi_rpc "(record { 
    rpc_list = vec {\"${alchemy_m}\";
                     \"${snownodes}\";
                     \"${triton_m}\";};\
    minimum_response_count = 2:nat32;})" --ic

# check 
signature=334PcrvBjAcjqMubimWAjy6Gsh8wDa57xw4yaFdhEa1L8qux2C9qyzrKRxTQCsfGoJGudLGWz3fQhnfQA8VvqenE
dfx canister call $SOLANA_ROUTE_CANISTER_ID valid_tx_from_multi_rpc "(\"${signature}\")" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID multi_rpc_config '()' --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIG}\",null)" --ic

###########################################################################
### update forward
### user defferent rpc between the solana route and omnity frontend
###########################################################################
dfx canister call $SOLANA_ROUTE_CANISTER_ID forward '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_forward "(opt \"${helius_m}\")" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID forward '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_forward '(null)' --ic


###########################################################################
### query solana route chain key and transfer 
###########################################################################
# query solana route chain key 
SIGNER=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic)
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current SIGNER: $SIGNER"
echo "$SIGNER balance: $(solana balance $SIGNER -u m)"

MASTER_KEY=$(solana address)
echo "current solana cli default address: $MASTER_KEY and balance: $(solana balance $MASTER_KEY -u m)"
# transfer SOL to solana route chain key 
AMOUNT=0.5
echo "transfer SOL to $SIGNER from $MASTER_KEY"
solana transfer $SIGNER $AMOUNT --with-memo init_account --allow-unfunded-recipient -u m
echo "$SIGNER balance: $(solana balance $SIGNER -u m)"
SIGNER_BALANCE=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID get_balance "(\"${SIGNER}\")" --ic)
echo "$SIGNER balance: $SIGNER_BALANCE via get_balance"
###########################################################################
### solana route schedule
###########################################################################

# start schedule
echo "start_schedule ... " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID start_schedule '()' --ic
echo "waiting for query directives or tickets from hub to solana route"
sleep 90

echo "check sync directive from hub "
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_chain_list '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_token_list '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_redeem_fee '("Bitcoin")' --ic
echo
# cannel schedule
dfx canister call $SOLANA_ROUTE_CANISTER_ID cancel_schedule '()' --ic


###########################################################################
### mint account 
###########################################################################

# # create token mint account
# TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
# TOKEN_NAME="HOPE•YOU•GET•RICH"
# TOKEN_SYMBOL="RICH"
# DECIMALS=2
# ICON="https://github.com/octopus-network/omnity-interoperability/blob/feature/solana-route/route/solana/assets/token_metadata.json"

# dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account "(\"${TOKEN_ID}\")" --ic
# (
#   opt record {
#     100_394_802 = variant { 1_066_763_494 };
#     359_375_608 = opt "4QFCua1ZQzPffEXzFvvW5H3FsGcpMUgeRLuUQCZy1pJJQ7H4rMZULh23Qi94ujX88ymYcmrUpHLPskRrF2sZps1g";
#     2_707_029_165 = "5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx";
#     3_871_938_408 = 6 : nat64;
#   },
# )
# dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_address "(\"${TOKEN_ID}\")" --ic
# dfx canister call $SOLANA_ROUTE_CANISTER_ID create_mint_account "(record {
#         token_id=\"${TOKEN_ID}\";
#         name=\"${TOKEN_NAME}\";
#         symbol=\"${TOKEN_SYMBOL}\";
#         decimals=${DECIMALS}:nat8;
#         uri=\"${ICON}\";
# })" --ic

# update token metadata
# origin
# TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
# TOKEN_NAME="HOPE•YOU•GET•RICH"
# TOKEN_SYMBOL="RICH.OT"
# DECIMALS=2
# RUNE_ID="840000:846"
# ICON="https://arweave.net/G058Vw4fqZqpcCHvYxjmQ_dgK_abkL-GjcR-p3os0Jc"

TOKEN_MINT=5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
TOKEN_NAME="RICH(old)"
TOKEN_SYMBOL="OT(old)"
DECIMALS=2
RUNE_ID="840000:846"
ICON=""
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_token22_metadata "(
        \"${TOKEN_MINT}\",
        record {
                token_id=\"${TOKEN_ID}\";
                name=\"${TOKEN_NAME}\";
                symbol=\"${TOKEN_SYMBOL}\";
                decimals=${DECIMALS}:nat8;
                uri=\"${ICON}\";
})" --ic

# remove token and mint account
dfx canister call $SOLANA_ROUTE_CANISTER_ID cancel_schedule '()' --ic
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
dfx canister call $SOLANA_ROUTE_CANISTER_ID remove_token_and_account "(\"${TOKEN_ID}\")" --ic

# add new RICH token
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
TOKEN_NAME="HOPE•YOU•GET•RICH"
TOKEN_SYMBOL="RICH.OT"
DECIMALS=2
RUNE_ID="840000:846"
ICON="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/rich_ot_meta.json"

dfx canister call $SOLANA_ROUTE_CANISTER_ID add_token "(record {
        token_id=\"${TOKEN_ID}\";
        name=\"${TOKEN_NAME}\";
        symbol=\"${TOKEN_SYMBOL}\";
        decimals=${DECIMALS}:nat8;
        icon=opt \"${ICON}\";
        metadata = vec{ record {\"rune_id\" ; \"840000:846\"}};
})" --ic

# update token mint with metaplex
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
TOKEN_NAME="HOPE•YOU•GET•RICH"
TOKEN_SYMBOL="RICH.OT"
DECIMALS=2
RUNE_ID="840000:846"
ICON="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/rich_meta.json"
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_token_metaplex "(
        record {
                token_id=\"${TOKEN_ID}\";
                name=\"${TOKEN_NAME}\";
                symbol=\"${TOKEN_SYMBOL}\";
                decimals=${DECIMALS}:nat8;
                uri=\"${ICON}\";
})" --ic

# start schedule and creat token mint
dfx canister call $SOLANA_ROUTE_CANISTER_ID start_schedule '()' --ic

# # get token mint
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account "(\"${TOKEN_ID}\")" --ic
TOKEN_MINT=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_address "(\"${TOKEN_ID}\")" --ic)
TOKEN_MINT=$(echo "$TOKEN_MINT" | awk -F'"' '{print $2}')
echo "token mint: $TOKEN_MINT"


# add new X token
# {
#       "decimals": 0,
#       "icon": [
#         "https://github.com/ordinals/ord/assets/8003221/d1481aa1-56db-4b00-b890-447a436199d3"
#       ],
#       "name": "RUNES•X•BITCOIN",
#       "rune_id": [
#         "840000:142"
#       ],
#       "symbol": "X",
#       "token_id": "Bitcoin-runes-RUNES•X•BITCOIN"
# }
TOKEN_ID="Bitcoin-runes-RUNES•X•BITCOIN"
TOKEN_NAME="RUNES•X•BITCOIN"
TOKEN_SYMBOL="RUNES.X"
DECIMALS=0
# ICON="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/x.json"
ICON="https://arweave.net/iLV2-ApjrXPDNoHldzB4a0fVVL-0hXR6dZGgudknz7c"

dfx canister call $SOLANA_ROUTE_CANISTER_ID add_token "(record {
        token_id=\"${TOKEN_ID}\";
        name=\"${TOKEN_NAME}\";
        symbol=\"${TOKEN_SYMBOL}\";
        decimals=${DECIMALS}:nat8;
        icon=opt \"${ICON}\";
        metadata = vec{ record {\"rune_id\" ; \"840000:142\"}};
})" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID create_mint_account "(record {
        token_id=\"${TOKEN_ID}\";
        name=\"${TOKEN_NAME}\";
        symbol=\"${TOKEN_SYMBOL}\";
        decimals=${DECIMALS}:nat8;
        uri=\"${ICON}\";
})" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account "(\"${TOKEN_ID}\")" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_token_metadata "(record {
        token_id=\"${TOKEN_ID}\";
        name=\"${TOKEN_NAME}\";
        symbol=\"${TOKEN_SYMBOL}\";
        decimals=${DECIMALS}:nat8;
        uri=\"${ICON}\";
})" --ic

# # get token mint
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account "(\"${TOKEN_ID}\")" --ic
TOKEN_MINT=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_address "(\"${TOKEN_ID}\")" --ic)
TOKEN_MINT=$(echo "$TOKEN_MINT" | awk -F'"' '{print $2}')
echo "token mint: $TOKEN_MINT"

# remove token info and token mint account for Bitcoin-runes-RUNES•X•BITCOIN
# token info in solana route
#  record {
#       decimals = 0 : nat8;
#       token_id = "Bitcoin-runes-RUNES•X•BITCOIN";
#       icon = opt "https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/x.json";
#       rune_id = opt "840000:142";
#       symbol = "RUNES.X";
#     };

# token mint account
# (
#   opt record {
#     100_394_802 = variant { 1_066_763_494 };
#     359_375_608 = opt "uXDCEeLZ5YZ3jU6p2mbee4UG4zux1ZorujVuNq6bQe79GYoveBJg1b932qajMeDU5GeZsJ4b7SgUmz2m3RsYRhd";
#     2_707_029_165 = "4eKCcgJLjTKDxpKic8craAs2wvvWhnh36z29FQVuetZV";
#     3_871_938_408 = 1 : nat64;
#   },
# )
dfx canister call $SOLANA_ROUTE_CANISTER_ID cancel_schedule '()' --ic
TOKEN_ID="Bitcoin-runes-RUNES•X•BITCOIN"
dfx canister call $SOLANA_ROUTE_CANISTER_ID remove_token_and_account "(\"${TOKEN_ID}\")" --ic


# create_mint_account for Bitcoin-runes-RUNES•X•BITCOIN2
# TOKEN_ID="Bitcoin-runes-RUNES•X•BITCOIN2"
# TOKEN_NAME="RUNES•X•BITCOIN"
# TOKEN_SYMBOL="X"
# DECIMALS=0
# ICON="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/x.json"
# # ICON="https://arweave.net/iLV2-ApjrXPDNoHldzB4a0fVVL-0hXR6dZGgudknz7c"
# dfx canister call $SOLANA_ROUTE_CANISTER_ID create_mint_account "(record {
#         token_id=\"${TOKEN_ID}\";
#         name=\"${TOKEN_NAME}\";
#         symbol=\"${TOKEN_SYMBOL}\";
#         decimals=${DECIMALS}:nat8;
#         uri=\"${ICON}\";
# })" --ic

# dfx canister call $SOLANA_ROUTE_CANISTER_ID start_schedule '()' --ic

# update token id from Bitcoin-runes-RUNES•X•BITCOIN2 to  Bitcoin-runes-RUNES•X•BITCOIN

dfx canister call $SOLANA_ROUTE_CANISTER_ID cancel_schedule '()' --ic
TOKEN_ID="Bitcoin-runes-RUNES•X•BITCOIN"
TOKEN_NAME="RUNES•X•BITCOIN"
TOKEN_SYMBOL="X"
DECIMALS=0
ICON="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/x_meta.json"
# ICON="https://arweave.net/iLV2-ApjrXPDNoHldzB4a0fVVL-0hXR6dZGgudknz7c"

# re add token for Bitcoin-runes-RUNES•X•BITCOIN
dfx canister call $SOLANA_ROUTE_CANISTER_ID add_token "(record {
        token_id=\"${TOKEN_ID}\";
        name=\"${TOKEN_NAME}\";
        symbol=\"${TOKEN_SYMBOL}\";
        decimals=${DECIMALS}:nat8;
        icon=opt \"${ICON}\";
        metadata = vec{ record {\"rune_id\" ; \"840000:142\"}};
})" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID start_schedule '()' --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account "(\"${TOKEN_ID}\")" --ic
# ACCOUNT=""
# RETRY=1
# SIGNATURE=""

# update mint token account for Bitcoin-runes-RUNES•X•BITCOIN
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_mint_account "(
    \"${TOKEN_ID}\",\
    record {
        account=\"${ACCOUNT}\";
        retry=${RETRY}:nat64;
        signature=\"${SIGNATURE}\";
        symbol=\"${TOKEN_SYMBOL}\";
        status=variant { Finalized };
})" --ic

# dfx canister call $SOLANA_ROUTE_CANISTER_ID start_schedule '()' --ic
# query token position
CHAIN_ID=eSolana
dfx canister call $HUB_CANISTER_ID get_chain_tokens "(opt \"${CHAIN_ID}\",null,0:nat64,5:nat64)" --ic
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
dfx canister call $HUB_CANISTER_ID get_chain_tokens "(opt \"${CHAIN_ID}\",opt \"${TOKEN_ID}\",0:nat64,5:nat64)" --ic
TOKEN_ID="Bitcoin-runes-RUNES•X•BITCOIN"
dfx canister call $HUB_CANISTER_ID get_chain_tokens "(opt \"${CHAIN_ID}\",opt \"${TOKEN_ID}\",0:nat64,5:nat64)" --ic


###########################################################################
### aossicated  account 
###########################################################################

# get ata addresss
TOKEN_MINT=8j45TBhQU6DQhRvoYd9dpQWzTNKstB6kpnfZ3pKDCxff
WALLET_ADDRESS=Gzt3ihQgUT6NmpVTxzyCqchC45HhaVxX8UqN7xne2x7k
dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account "(\"${WALLET_ADDRESS}\",
        \"${TOKEN_MINT}\")" --ic
# remove ata
dfx canister call $SOLANA_ROUTE_CANISTER_ID remove_associated_account "(\"${WALLET_ADDRESS}\",
        \"${TOKEN_MINT}\")" --ic

ATA=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account_address "(\"${WALLET_ADDRESS}\",
        \"${TOKEN_MINT}\")" --ic)
ATA=$(echo "$ATA" | awk -F'"' '{print $2}')
echo "The dest address: $WALLET_ADDRESS and the token address: $TOKEN_MINT aossicated account is: $ATA"

# create ata
dfx canister call $SOLANA_ROUTE_CANISTER_ID cancel_schedule '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID create_aossicated_account "(\"${WALLET_ADDRESS}\",
        \"${TOKEN_MINT}\")" --ic  
dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account "(\"${WALLET_ADDRESS}\",
        \"${TOKEN_MINT}\")" --ic
# update ata
TOKEN_MINT=8j45TBhQU6DQhRvoYd9dpQWzTNKstB6kpnfZ3pKDCxff
WALLET_ADDRESS=8ALeC77dTQTrvf1gEG7xr2Lpu6UQC1ARtXNrsE3svyfE
ATA=APtHuv7vW3t9Kefxjy5j3bWygjd2KUV9my9ivFy3DwgS
sig=2bgLj5FKuxpr1pySwvwKeYE1v5sDvWx49AT1zqipZ9FmGhb4Z3Rz4EwJRmpFVhwoqkVh6YfMVQ4LDfxcd81c7EFp
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_associated_account "(
        \"${WALLET_ADDRESS}\",
        \"${TOKEN_MINT}\",
        record {
                account=\"${ATA}\";
                retry=0:nat64;
                token_mint=\"${TOKEN_MINT}\";
                status=variant { Finalized };
                signature=opt \"${sig}\";
        }
)" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID create_aossicated_account "(\"${WALLET_ADDRESS}\",
        \"${TOKEN_MINT}\")" --ic  

# mint to user ata
TOKEN_MINT=8j45TBhQU6DQhRvoYd9dpQWzTNKstB6kpnfZ3pKDCxff
ATA=APtHuv7vW3t9Kefxjy5j3bWygjd2KUV9my9ivFy3DwgS
AMOUNT=1800
dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_to "(
        \"${ATA}\",
        \"${TOKEN_MINT}\",
        ${AMOUNT}:nat64
)" --ic


# query and create ATA
# SOL_RECEIVER="FDR2mUpiHKFonnwbUujLyhuNTt7LHEjZ1hDFX4UuCngt"
# create aossicated account for user
# dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account "(\"${SOL_RECEIVER}\",
#         \"${TOKEN_MINT}\")" --ic  

# dfx canister call $SOLANA_ROUTE_CANISTER_ID create_aossicated_account "(\"${SOL_RECEIVER}\",
#         \"${TOKEN_MINT}\")" --ic  
SOL_RECEIVER=aboTTUwwPpkfRSiWS7WP97sj9dqtEsrE7kprDos7wj2
SOL_RECEIVER="6fprKjprjWKKLFEyiX7f7kHb2EVxpK1eYfMTSM1SkTkk"
dfx canister call $SOLANA_ROUTE_CANISTER_ID derive_aossicated_account "(\"${SOL_RECEIVER}\",
        \"${TOKEN_MINT}\")" --ic  

dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account "(\"${SOL_RECEIVER}\",
        \"${TOKEN_MINT}\")" --ic  

# get ata addresss
ATA=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account_address "(\"${SOL_RECEIVER}\",
        \"${TOKEN_MINT}\")" --ic)
ATA=$(echo "$ATA" | awk -F'"' '{print $2}')
echo "The dest address: $SOL_RECEIVER and the token address: $TOKEN_MINT aossicated account is: $ATA"

# create ata
dfx canister call $SOLANA_ROUTE_CANISTER_ID create_aossicated_account "(\"${SOL_RECEIVER}\",
        \"${TOKEN_MINT}\")" --ic  

###########################################################################
### mint token
###########################################################################

# manally mint token
# TID=28b47548-55dc-4e89-b41d-76bc0247828e1
# ATA=H8ESHFzzCki2c6dKbkUA8y5N7UpnbcW2THx91mhaoGfG
# MINT_AMOUNT=55555
# TOKEN_MINT=4PY24Vzmd4tCm24yekAW8tnv1oQ9SLbufo2WXT7xYhq1
# dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req "(\"${TID}\")" --ic
# dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_status "(\"${TID}\")" --ic

# dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_with_req "(record{
#         ticket_id=\"${TID}\";
#         associated_account=\"${ATA}\";
#         amount=${MINT_AMOUNT}:nat64;
#         token_mint=\"${TOKEN_MINT}\";
#         status=variant { Unknown };
#         signature=null;
#         retry=0;})" --ic

# dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_status "(\"${TID}\")" --ic

#  send_raw_transaction from ic-solana-provider
# RAW_TX=4S6Q1Toi7GEWiadHTsc5LT6Q9askJGMp9hBWJZWDNfazH82pFVh6aURGb8MLbas2ezgDgtuj7GbV7R5CsS9aFYwi3tz8oLaScPYT5JALaAEBXJRatFHRfZtJPp4WDJ9bKDpvwD8P4dv23pDD2Kfr8vi9xW9zF4FkZqdEMq3q1J3g5risnCn7FiJkrKxG5Prc2SSPZhDUJpLsFB51SJ3BbNVL59Ztjaz5vTcTr4o7xqmUmUdnR8WBWj9MhQbGCF99T5QsTA8pYw2vviMc1Kjvmao1Wdh49ow1rEemyZPkqEE6vFQwuGTZbgXJH8d5UGcSPwG8FbJqKGsfYb
# dfx canister call $SOL_PROVIDER_CANISTER_ID sol_sendRawTransaction "(\"${RAW_TX}\")" --ic

# SIG=2VGvopAP2NinJ48fpPKae9svtHcAYw6K1mUyW2GDyEyW6Dp3mBtTwat1wPfbCnq2G6hkQa8yiQZTf3dEHDWa4erK
# dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIG}\,null")" --ic

# update_mint_token_req and remint 
dfx canister call $SOLANA_ROUTE_CANISTER_ID cancel_schedule '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_tickets_from_queue '()' --ic

TID=b472a93294435ee522389150251bf58a65c5c5b11c42f3ac25c7b69b41b5ab69
ATA=BbDheYkCrEbvHj3QswhBTMmcDM4aQ7r9cG9fxzpdfSXM
MINT_AMOUNT=120000
TOKEN_MINT=5HmvdqEM3e7bYKTUix8dJSZaMhx9GNkQV2vivsiC3Tdx
dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req "(\"${TID}\")" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_mint_token_req "(record{
        ticket_id=\"${TID}\";
        associated_account=\"${ATA}\";
        amount=${MINT_AMOUNT}:nat64;
        token_mint=\"${TOKEN_MINT}\";
        status=variant { New };
        signature=null;
        retry=0;})" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req "(\"${TID}\")" --ic


dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_with_req "(record{
        ticket_id=\"${TID}\";
        associated_account=\"${ATA}\";
        amount=${MINT_AMOUNT}:nat64;
        token_mint=\"${TOKEN_MINT}\";
        status=variant { New };
        signature=null;
        retry=0:nat64;})" --ic

SIG=3KGwVoVwKmREZHMvm24Q99giSwdjZo9woH66rK5C7XZuWmjTxazpoDDSrbTTssEjsZ55VpEDTk2MuCeGLRhcfCbp
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_signature_status "(vec {\"${SIG}\"})" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_mint_token_req "(record{
        ticket_id=\"${TID}\";
        associated_account=\"${ATA}\";
        amount=${MINT_AMOUNT}:nat64;
        token_mint=\"${TOKEN_MINT}\";
        status=variant { Finalized };
        signature=opt \"${SIG}\";
        retry=1:nat64;})" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req "(\"${TID}\")" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID update_tx_hash_to_hub "(\"${SIG}\",\"${TID}\")" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID remove_ticket_from_quene "(\"${TID}\")" --ic
remove_ticket_from_quene
dfx canister call $SOLANA_ROUTE_CANISTER_ID start_schedule '()' --ic


###########################################################################
### gen ticket and send it to hub
###########################################################################

SIG=334PcrvBjAcjqMubimWAjy6Gsh8wDa57xw4yaFdhEa1L8qux2C9qyzrKRxTQCsfGoJGudLGWz3fQhnfQA8VvqenE
dfx canister call $SOLANA_ROUTE_CANISTER_ID valid_tx_from_multi_rpc "(\"${SIG}\")" --ic

# gen ticket and send it to hub
signature=2ZD98V6XEMqmv5hveWyHx29HPjgxCEAvDQnntNxMJYrUq8jffGeKe8varfVEHF9EbScPZruAsWke4k9gfFWo77Wm
target_chain_id=Bitcoin
sender=3duAFv2j7VvKUpUWEK1p9itMvCkZxF6P5PArdU2G7z3W
receiver=bc1qvtcrsrsgpl443z3s7k0fez0dw7dn08fnqjhaz6
token_id=Bitcoin-runes-HOPE•YOU•GET•RICH
amount=400000
# action=variant { Redeem }
memo=bc1qvtcrsrsgpl443z3s7k0fez0dw7dn08fnqjhaz6
dfx canister call $SOLANA_ROUTE_CANISTER_ID generate_ticket "(record {
    signature=\"${signature}\";
    target_chain_id=\"${target_chain_id}\";
    sender=\"${sender}\";
    receiver=\"${receiver}\";
    token_id=\"${token_id}\";
    amount=${amount}:nat64;
    action=variant { Redeem };
    memo=opt \"${memo}\";
})" --ic


dfx canister call $SOLANA_ROUTE_CANISTER_ID get_fee_account '()' --ic

FEE_ACCOUNT="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_fee_account "(\"${FEE_ACCOUNT}\")" --ic

# schnorr test
dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic
SIGNER=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic)
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current SIGNER: $SIGNER"
echo "$SIGNER balance: $(solana balance $SIGNER -u m)"

dfx canister call $SOLANA_ROUTE_CANISTER_ID sign '("Hi,Boern")' --ic

#ISSUES

# 1. Transaction simulation failed: Blockhash not found 
# reason and solution:
# 1) solana route 所在的子网延时，比如chainkey签名，http outcall请求等因素导致的构建tx时间过长
# 从而导致blockhash expiration，需要重新构建tx，然后重试
# 2) solana mainnet 负载过高或者网络拥堵，导致blockhash expiration，需要重新构建tx，然后重试


# 2. solana route 发送tx 只rpc后，正常返回交易签名，但是查询此签名状态时，返回签名状态为空
# reason and solution：
# 1) rpc 将tx 提交给了solana slot leader，但是此时tx尚未被处理，查询签名状态时返回null，此时只需重试即可；
# 2) rpc 将tx 提交给了solana slot leader，由于blockhash expiration，优先级以及其他因素导致tx被丢弃，查询签名状态时返回null，
# 这种情况首先要确认这笔tx确实被丢弃了，然后再重建tx再次尝试，最重要的也是比较困难的地方在于如何确认tx被丢弃了，不同的交易需要确认的方法可能也不太一样
# 如果创建账户的tx被丢弃了，可以通过查询账户是否存在，依此确认tx是否成功或者被丢弃；
# 如果tx 是关于mint_to的操作，可能需要检查这笔tx详情，如果该tx存在且跟预期数据一致就认为tx成功，如果tx确实不存在，那么此tx被丢弃；

# 3. metadata limit for name, symbol
# 	token name  max 32 bytes
# 	token symbol max 10 bytes
# 	token uri 200 bytes