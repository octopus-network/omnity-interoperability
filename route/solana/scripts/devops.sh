#!/usr/bin/env bash

# disable dfx warning
export DFX_WARNING="-mainnet_plaintext_identity"

echo "Setting up for production environment..."
ADMIN=$(dfx identity get-principal --ic)

# Production env
HUB_CANISTER_ID=7wupf-wiaaa-aaaar-qaeya-cai
SCHNORR_KEY_NAME="key_1"
PROXY_URL="https://solana-rpc-proxy-398338012986.us-central1.run.app"
SOL_PROVIDER_CANISTER_ID=l3ka6-4yaaa-aaaar-qahpa-cai
SOLANA_ROUTE_CANISTER_ID=lvinw-hiaaa-aaaar-qahoa-cai
triton_m=https://png.rpcpool.com/13a5c61c672e6cd88357abf3709a
alchemy_m=https://solana-mainnet.g.alchemy.com/v2/t25IzpcIjBXhP-LOurqrTWLWmhPuBwsk
snownodes=https://sol.nownodes.io
ankr_m=https://rpc.ankr.com/solana/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3

echo "product environment: 
    admin id: $ADMIN
    omnity_hub canister id: $HUB_CANISTER_ID 
    schnorr key name: $SCHNORR_KEY_NAME 
    proxy url: $PROXY_URL
    ic solana provider canister id: $SOL_PROVIDER_CANISTER_ID
    solana route canister id: $SOLANA_ROUTE_CANISTER_ID"

# install or reinstall ic provider canister
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
# test canister api
nownodes=https://sol.nownodes.io
# ankr_d=https://rpc.ankr.com/solana_devnet/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
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
# rpc1=https://solana-mainnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ
ankr_m=https://rpc.ankr.com/solana/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
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
#     multi_rpc_config = record { rpc_list = vec {\"${rpc2}\"};\
#     minimum_response_count = 1:nat32;}; \
#     forward = null
#     } })" \
#     --mode=reinstall -y \
#     --wasm=./assets/solana_route.wasm.gz \
#     --ic 

echo "upgrade $SOLANA_ROUTE_CANISTER_ID ..."
dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument "(opt variant { Upgrade = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${SOL_CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_key_name = \"${SCHNORR_KEY_NAME}\";\
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
    fee_account= opt \"${FEE_ACCOUNT}\";\
    multi_rpc_config = record { rpc_list = vec {\"${alchemy_m}\"};\
    minimum_response_count = 1:nat32;}; \
    } })" \
    --mode=upgrade -y \
    --wasm=./assets/solana_route.wasm.gz \
    --ic 

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
# test 
dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID multi_rpc_config '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID forward '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_latest_blockhash '()' --ic 
dfx canister call $SOLANA_ROUTE_CANISTER_ID debug '(true)' --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction '("4kogo438gk3CT6pifHQa7d4CC7HRidnG2o6EWxwGFvAcuSC7oTeG3pWTYDy9wuCYmGxJe1pRdTHf7wMcnJupXSf4",null)' --ic


# update schnorr info
# dfx canister call $SOLANA_ROUTE_CANISTER_ID update_schnorr_key '("key_1")' --ic

# update multi_rpc_config
triton_m=https://png.rpcpool.com/13a5c61c672e6cd88357abf3709a
alchemy_m=https://solana-mainnet.g.alchemy.com/v2/t25IzpcIjBXhP-LOurqrTWLWmhPuBwsk
helius_m=https://mainnet.helius-rpc.com/?api-key=174a6ec2-4439-4fca-9277-b12900c71fa5
snownodes=https://sol.nownodes.io
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

signature=334PcrvBjAcjqMubimWAjy6Gsh8wDa57xw4yaFdhEa1L8qux2C9qyzrKRxTQCsfGoJGudLGWz3fQhnfQA8VvqenE
dfx canister call $SOLANA_ROUTE_CANISTER_ID valid_tx_from_multi_rpc "(\"${signature}\")" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID multi_rpc_config '()' --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIG}\",null)" --ic

# update forward
dfx canister call $SOLANA_ROUTE_CANISTER_ID forward '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_forward "(opt \"${helius_m}\")" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID forward '()' --ic




# query signer
SIGNER=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic)
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current SIGNER: $SIGNER"
echo "$SIGNER balance: $(solana balance $SIGNER -u m)"

# req airdrop
# solana airdrop 2
MASTER_KEY=$(solana address)
echo "current solana cli default address: $MASTER_KEY and balance: $(solana balance $MASTER_KEY)"
# transfer SOL to init signer
AMOUNT=0.5
echo "transfer SOL to $SIGNER from $MASTER_KEY"
solana transfer $SIGNER $AMOUNT --with-memo init_account --allow-unfunded-recipient -u m
echo "$SIGNER balance: $(solana balance $SIGNER -u m)"

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

# manual operation 

# # create token mint account
# TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
# TOKEN_NAME="HOPE•YOU•GET•RICH"
# TOKEN_SYMBOL="RICH"
# DECIMALS=2
# ICON="https://github.com/octopus-network/omnity-interoperability/blob/feature/solana-route/route/solana/assets/token_metadata.json"

# dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account "(\"${TOKEN_ID}\")" --ic
# dfx canister call $SOLANA_ROUTE_CANISTER_ID create_mint_account "(record {
#         token_id=\"${TOKEN_ID}\";
#         name=\"${TOKEN_NAME}\";
#         symbol=\"${TOKEN_SYMBOL}\";
#         decimals=${DECIMALS}:nat8;
#         uri=\"${ICON}\";
# })" --ic

# update token
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
TOKEN_NAME="HOPE•YOU•GET•RICH"
TOKEN_SYMBOL="RICH.OT"
DECIMALS=2
ICON="https://arweave.net/zr3QxOefTtsfd_651Q1LlkWcISat5NtyfD9ENPNkXDk"

# ICON="https://arweave.net/G058Vw4fqZqpcCHvYxjmQ_dgK_abkL-GjcR-p3os0Jc"
# ICON="https://raw.githubusercontent.com/octopus-network/omnity-interoperability/feature/solana-route/route/solana/assets/rich.json"
# ICON="https://github.com/octopus-network/omnity-interoperability/blob/feature/solana-route/route/solana/assets/token.png"
# ICON="https://geniidata.com/content/d66de939cb3ddb4d94f0949612e06e7a84d4d0be381d0220e2903aad68135969i0"
# ICON="https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json"
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


# manally mint token
# TID=28b47548-55dc-4e89-b41d-76bc0247828e1
# ATA=H8ESHFzzCki2c6dKbkUA8y5N7UpnbcW2THx91mhaoGfG
# MINT_AMOUNT=55555
# TOKEN_MINT=4PY24Vzmd4tCm24yekAW8tnv1oQ9SLbufo2WXT7xYhq1
# dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req "(\"${TID}\")" --ic
# dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_status "(\"${TID}\")" --ic

# dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token "(record{
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


dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token "(record{
        ticket_id=\"${TID}\";
        associated_account=\"${ATA}\";
        amount=${MINT_AMOUNT}:nat64;
        token_mint=\"${TOKEN_MINT}\";
        status=variant { New };
        signature=null;
        retry=0;})" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_tx_hash "(\"${TID}\")" --ic

dfx canister call $SOLANA_ROUTE_CANISTER_ID start_schedule '()' --ic


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