#!/usr/bin/env bash

# disable dfx warning
export DFX_WARNING="-mainnet_plaintext_identity"

echo "Setting up for testnet environment..."
ADMIN=$(dfx identity get-principal --ic)
# Testnet env
# HUB_CANISTER_ID=xykho-eiaaa-aaaag-qjrka-cai
HUB_CANISTER_ID=arlph-jyaaa-aaaak-ak2oa-cai
SCHNORR_KEY_NAME="test_key_1"
# SCHNORR_KEY_NAME="key_1"
# SOLANA_RPC_URL="https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
# SOLANA_RPC_URL="https://solana-rpc-proxy-398338012986.us-central1.run.app"
PROXY_URL="https://solana-rpc-proxy-398338012986.us-central1.run.app"
SOL_PROVIDER_CANISTER_ID=lzl57-kyaaa-aaaaj-qa4ya-cai
SOLANA_ROUTE_CANISTER_ID=4o543-xaaaa-aaaao-a3q3a-cai
echo "testnet environment: 
    admin id: $ADMIN
    omnity_hub canister id: $HUB_CANISTER_ID 
    schnorr key name: $SCHNORR_KEY_NAME 
    proxy url: $PROXY_URL
    ic solana provider canister id: $SOL_PROVIDER_CANISTER_ID
    solana route canister id: $SOLANA_ROUTE_CANISTER_ID"

# install or reinstall omnity hub
# create canister for omnity hub
# dfx canister create omnity_hub --ic
echo "reinstall $HUB_CANISTER_ID ..."
dfx canister install $HUB_CANISTER_ID --argument "(variant { Init = record { admin = principal \"${ADMIN}\" } })" \
  --mode=reinstall -y \
  --wasm=./assets/omnity_hub.wasm.gz \
  --ic

# change log level for debugging
dfx canister status $HUB_CANISTER_ID --ic
dfx canister call $HUB_CANISTER_ID set_logger_filter '("debug")' --ic
echo 

echo "reinstall $SOL_PROVIDER_CANISTER_ID ..."
dfx canister install $SOL_PROVIDER_CANISTER_ID --argument "( record { 
    rpc_url = opt \"${PROXY_URL}\"; 
    schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
    nodesInSubnet = opt 28; 
    } )" \
    --mode=reinstall -y \
    --wasm=./assets/ic_solana_provider.wasm.gz \
    --ic 

dfx canister status $SOL_PROVIDER_CANISTER_ID --ic
# test canister api
ankr=https://rpc.ankr.com/solana_devnet/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
test_account=3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia
test_sig=2VGvopAP2NinJ48fpPKae9svtHcAYw6K1mUyW2GDyEyW6Dp3mBtTwat1wPfbCnq2G6hkQa8yiQZTf3dEHDWa4erK
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_latestBlockhash "(opt \"${ankr}\")" --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getAccountInfo "(\"${test_account}\",opt \"${ankr}\")" --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getSignatureStatuses "(vec {\"${test_sig}\"},opt \"${ankr}\")" --ic
echo 

# solana_route canister
SOL_CHAIN_ID="eSolana"
SOL_FEE="SOL"
FEE_ACCOUNT="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
rpc1=https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ
rpc2=https://rpc.ankr.com/solana_devnet/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
rpc3=https://nd-471-475-490.p2pify.com/6de0b91c609fb3bd459e043801aa6aa4

echo "reinstall $SOLANA_ROUTE_CANISTER_ID ..."
dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument "(variant { Init = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${SOL_CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_key_name = \"${SCHNORR_KEY_NAME}\";\
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
    fee_account= opt \"${FEE_ACCOUNT}\";\
    multi_rpc_config = record { rpc_list = vec {\"${rpc1}\";\"${rpc2}\";\"${rpc3}\"};\
    minimum_response_count = 2:nat32;}; \
    forward = opt \"${rpc1}\"
    } })" \
    --mode=reinstall -y \
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
# dfx canister call $SOLANA_ROUTE_CANISTER_ID update_schnorr_info "(principal \"${SCHNORR_CANISTER_ID}\",\"${SCHNORR_KEY_NAME}\")" --ic 
dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_latest_blockhash '()' --ic 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${test_sig}\",opt \"${ankr}\")" --ic
# update schnorr info
# dfx canister call $SOLANA_ROUTE_CANISTER_ID update_schnorr_info '(principal "aaaaa-aa","key_1")' --ic

# sub topic
BITCOIN_CHAIN_ID="Bitcoin"
BITCOIN_CANISTER_ID="xykho-eiaaa-aaaag-qjrka-cai"
dfx canister call $HUB_CANISTER_ID sub_directives "(opt \"${BITCOIN_CHAIN_ID}\",
         vec {variant {AddChain};variant {UpdateChain};
         variant {AddToken}; variant {UpdateToken};
         variant {UpdateFee} ;variant {ToggleChainState} })" --ic
dfx canister call $HUB_CANISTER_ID sub_directives "(opt \"${SOL_CHAIN_ID}\",
         vec {variant {AddChain};variant {UpdateChain};
         variant {AddToken}; variant {UpdateToken};
         variant {UpdateFee} ;variant {ToggleChainState} })" --ic

dfx canister call $HUB_CANISTER_ID query_subscribers '(null)' --ic 

# add chains
echo "add bitcoin chain to hub"
dfx canister call $HUB_CANISTER_ID validate_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${BITCOIN_CHAIN_ID}\"; chain_type=variant { SettlementChain }; 
        canister_id=\"${BITCOIN_CANISTER_ID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${SOL_CHAIN_ID}\"}; 
        fee_token=null}}})" --ic 
dfx canister call $HUB_CANISTER_ID execute_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${BITCOIN_CHAIN_ID}\"; chain_type=variant { SettlementChain }; 
        canister_id=\"${BITCOIN_CANISTER_ID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${SOL_CHAIN_ID}\"};
        fee_token=null}}})" --ic 
dfx canister call $HUB_CANISTER_ID query_directives "(opt \"${SOL_CHAIN_ID}\",opt variant {AddChain},0:nat64,5:nat64)" --ic

echo  "add solana chain to hub"
dfx canister call $HUB_CANISTER_ID validate_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${SOL_CHAIN_ID}\"; 
        chain_type=variant { ExecutionChain }; 
        canister_id=\"${SOLANA_ROUTE_CANISTER_ID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${BITCOIN_CHAIN_ID}\"}; 
        fee_token=opt \"${SOL_FEE}\"}}})" \
        --ic 
dfx canister call $HUB_CANISTER_ID execute_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${SOL_CHAIN_ID}\"; 
        chain_type=variant { ExecutionChain }; 
        canister_id=\"${SOLANA_ROUTE_CANISTER_ID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${BITCOIN_CHAIN_ID}\"}; 
        fee_token=opt \"${SOL_FEE}\"}}})" \
        --ic 
dfx canister call $HUB_CANISTER_ID query_directives "(opt \"${BITCOIN_CHAIN_ID}\",opt variant {AddChain},0:nat64,5:nat64)" --ic 

# push update chain(bitcoin) to solana route
dfx canister call $HUB_CANISTER_ID validate_proposal "(vec {variant { 
        UpdateChain = record { chain_state=variant { Active }; 
        chain_id = \"${BITCOIN_CHAIN_ID}\"; 
        chain_type=variant { SettlementChain }; 
        canister_id=\"${BITCOIN_CANISTER_ID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${SOL_CHAIN_ID}\"; 
                                \"eICP\";
                                \"bevm_testnet\";
                                \"bitlayer_testnet\";
                                \"B²_testnet\";
                                \"xlayer_testnet\";
                                }; 
        fee_token=null}}})" \
        --ic 
        
dfx canister call $HUB_CANISTER_ID execute_proposal "(vec {variant { 
        UpdateChain = record { chain_state=variant { Active }; 
        chain_id = \"${BITCOIN_CHAIN_ID}\"; 
        chain_type=variant { SettlementChain }; 
        canister_id=\"${BITCOIN_CANISTER_ID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${SOL_CHAIN_ID}\"; 
                                \"eICP\";
                                \"bevm_testnet\";
                                \"bitlayer_testnet\";
                                \"B²_testnet\";
                                \"xlayer_testnet\";
                                }; 
        fee_token=null}}})" \
        --ic 

# dfx canister call $HUB_CANISTER_ID query_directives "(
#     opt \"${BITCOIN_CHAIN_ID}\",
#     opt variant {AddChain},0:nat64,5:nat64)" \
#     --ic 

# dfx canister call $HUB_CANISTER_ID query_directives "(
#     opt \"${SOL_CHAIN_ID}\",
#     opt variant {AddChain},0:nat64,5:nat64)" \
#     --ic 

# add token
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•NICE202409242101"
TOKEN_NAME="HOPE•YOU•GET•NICE202409242101"
TOKEN_SYMBOL="NICE202409242101"
DECIMALS=2
ICON="https://github.com/octopus-network/omnity-interoperability/blob/feature/solana-route/route/solana/assets/token_metadata.json"

dfx canister call $HUB_CANISTER_ID validate_proposal "( vec {variant { AddToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${ICON}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})" \
        --ic 
dfx canister call $HUB_CANISTER_ID execute_proposal "( vec {variant { AddToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${ICON}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})" \
        --ic 

dfx canister call $HUB_CANISTER_ID query_directives "(
    opt \"${BITCOIN_CHAIN_ID}\",
    opt variant {AddToken},0:nat64,5:nat64)" \
    --ic

dfx canister call $HUB_CANISTER_ID query_directives "(
    opt \"${SOL_CHAIN_ID}\",
    opt variant {AddToken},0:nat64,5:nat64)" \
    --ic

# update fee
dfx canister call $HUB_CANISTER_ID update_fee "vec {variant { UpdateTargetChainFactor = 
        record { target_chain_id=\"${BITCOIN_CHAIN_ID}\"; 
                 target_chain_factor=10000 : nat}}; 
                 variant { UpdateFeeTokenFactor = record { fee_token=\"${SOL_FEE}\"; 
                                                 fee_token_factor=1 : nat}}}" \
        --ic 

dfx canister call $HUB_CANISTER_ID query_directives "(opt \"${SOL_CHAIN_ID}\",opt variant {UpdateFee},0:nat64,12:nat64)" --ic 

# query signer
SIGNER=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic)
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current SIGNER: $SIGNER"
echo "$SIGNER balance: $(solana balance $SIGNER)"

# req airdrop
# solana airdrop 2
MASTER_KEY=$(solana address)
echo "current solana cli default address: $MASTER_KEY and balance: $(solana balance $MASTER_KEY)"
# transfer SOL to init signer
AMOUNT=0.5
echo "transfer SOL to $SIGNER from $MASTER_KEY"
solana transfer $SIGNER $AMOUNT --with-memo init_account --allow-unfunded-recipient
echo "$SIGNER balance: $(solana balance $SIGNER)"

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

echo "mock transfer from Bitcoin to Solana ..."
echo 
TID="28b47548-55dc-4e89-b41d-76bc0247828e"
AMOUNT="222222222"
SOL_RECEIVER="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
dfx canister call $HUB_CANISTER_ID send_ticket "(record { ticket_id = \"${TID}\"; 
        ticket_type = variant { Normal }; 
        ticket_time = 1715654809737051178 : nat64; 
        token = \"${TOKEN_ID}\"; 
        amount = \"${AMOUNT}\"; 
        src_chain = \"${BITCOIN_CHAIN_ID}\"; 
        dst_chain = \"${SOL_CHAIN_ID}\"; 
        action = variant { Transfer }; 
        sender = null; 
        receiver = \"${SOL_RECEIVER}\";
        memo = null; })" \
    --ic
dfx canister call $HUB_CANISTER_ID query_tickets "(opt \"${SOL_CHAIN_ID}\",0:nat64,5:nat64)" --ic
echo 

sleep 120

echo "canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req "(\"${TID}\")" --ic
echo "canister call $SOLANA_ROUTE_CANISTER_ID mint_token_status " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_status "(\"${TID}\")" --ic

echo "canister call $SOLANA_ROUTE_CANISTER_ID get_tickets_from_queue " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_tickets_from_queue '()' --ic
echo 

sleep 20

# get token mint
echo "dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account "(\"${TOKEN_ID}\")" --ic
TOKEN_MINT=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_address "(\"${TOKEN_ID}\")" --ic)
TOKEN_MINT=$(echo "$TOKEN_MINT" | awk -F'"' '{print $2}')
echo "token mint: $TOKEN_MINT"

# get aossicated account based on owner and token mint
echo "dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account "(\"${SOL_RECEIVER}\",
        \"${TOKEN_MINT}\")" --ic  
ATA=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account_address "(\"${SOL_RECEIVER}\",
        \"${TOKEN_MINT}\")" --ic)
ATA=$(echo "$ATA" | awk -F'"' '{print $2}')
while [ -z "$ATA" ]; do
  echo "ATA is empty, waiting..."
  sleep 5  
  ATA=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account_address "(\"${SOL_RECEIVER}\",\"${TOKEN_MINT}\")" --ic)
  ATA=$(echo "$ATA" | awk -F'"' '{print $2}')
done
echo "The dest address: $SOL_RECEIVER and the token address: $TOKEN_MINT aossicated account is: $ATA"

sleep 15

echo "mock redeem from solana to customs... "
# first collect fee
# get fee account
FEE_ACCOUNT=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID get_fee_account '()' --ic )
FEE_ACCOUNT=$(echo "$FEE_ACCOUNT" | awk -F'"' '{print $2}')
echo "fee account: $FEE_ACCOUNT"
# get fee amount
FEE_AMOUNT=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID get_redeem_fee "(\"${BITCOIN_CHAIN_ID}\")" --ic )
FEE_AMOUNT=$(echo "$FEE_AMOUNT" | grep -oE '[0-9_]+ ' | sed 's/_//g' | awk '{printf "%.9f\n", $1 / 1000000000}')
echo "fee account: $FEE_AMOUNT"
# collect fee
WALLET_ADDRESS=$(solana address)
echo "collect fee to $FEE_ACCOUNT from $WALLET_ADDRESS"
solana transfer $FEE_ACCOUNT $FEE_AMOUNT 

# second, burn token
CUSTOMS_RECEIVER="bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6"
OWNER=~/.config/solana/boern.json
BURN_AMOUNT=1111111
echo spl-token burn $ATA $BURN_AMOUNT  --with-memo $CUSTOMS_RECEIVER  --owner $OWNER
# echo $(spl-token burn $ATA $BURN_AMOUNT  --with-memo $CUSTOMS_RECEIVER  --owner $OWNER)
SIGNAURE=$(spl-token burn $ATA $BURN_AMOUNT  --with-memo $CUSTOMS_RECEIVER  --owner $OWNER)
SIGNAURE=$(echo "$SIGNAURE" | awk '/Signature:/ {line=$2} END {print line}')
echo "burn signature: $SIGNAURE"
sleep 10

dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIGNAURE}\",opt \"${ankr}\")" --ic


# secord,generate ticket
dfx canister call $SOLANA_ROUTE_CANISTER_ID generate_ticket "(record {
        signature=\"$SIGNAURE\";
        action = variant { Redeem };
        token_id = \"${TOKEN_ID}\";
        target_chain_id =  \"${BITCOIN_CHAIN_ID}\";
        sender =  \"${SOL_RECEIVER}\";
        receiver =  \"${CUSTOMS_RECEIVER}\";
        amount = $BURN_AMOUNT:nat64;
        memo = null;
        })" \
    --ic
dfx canister call $HUB_CANISTER_ID query_tickets "(opt \"${BITCOIN_CHAIN_ID}\",0:nat64,5:nat64)" --ic

sleep 300

# cannel schedule
dfx canister call $SOLANA_ROUTE_CANISTER_ID cancel_schedule '()' --ic

# manual operation 

# # create token mint account
# dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account "(\"${TOKEN_ID}\")" --ic
# dfx canister call $SOLANA_ROUTE_CANISTER_ID create_mint_account "(record {
#         token_id=\"${TOKEN_ID}\";
#         name=\"${TOKEN_NAME}\";
#         symbol=\"${TOKEN_SYMBOL}\";
#         decimals=${DECIMALS}:nat8;
#         uri=\"${ICON}\";
# })" --ic

# update token
# dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_account "(\"${TOKEN_ID}\")" --ic
# dfx canister call $SOLANA_ROUTE_CANISTER_ID update_token_metadata "(record {
#         token_id=\"${TOKEN_ID}\";
#         name=\"${TOKEN_NAME}\";
#         symbol=\"${TOKEN_SYMBOL}\";
#         decimals=${DECIMALS}:nat8;
#         uri=\"${ICON}\";
# })" --ic

# # get token mint
# TOKEN_MINT=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_address "(\"${TOKEN_ID}\")" --ic)
# TOKEN_MINT=$(echo "$TOKEN_MINT" | awk -F'"' '{print $2}')
# echo "token mint: $TOKEN_MINT"

# SOL_RECEIVER="FDR2mUpiHKFonnwbUujLyhuNTt7LHEjZ1hDFX4UuCngt"
# create aossicated account for user
# dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account "(\"${SOL_RECEIVER}\",
#         \"${TOKEN_MINT}\")" --ic  

# dfx canister call $SOLANA_ROUTE_CANISTER_ID create_aossicated_account "(\"${SOL_RECEIVER}\",
#         \"${TOKEN_MINT}\")" --ic  

# get ata
# ATA=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account_address "(\"${SOL_RECEIVER}\",
#         \"${TOKEN_MINT}\")" --ic)
# ATA=$(echo "$ATA" | awk -F'"' '{print $2}')
# echo "The dest address: $SOL_RECEIVER and the token address: $TOKEN_MINT aossicated account is: $ATA"

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

# test send_raw_transaction
# RAW_TX=4S6Q1Toi7GEWiadHTsc5LT6Q9askJGMp9hBWJZWDNfazH82pFVh6aURGb8MLbas2ezgDgtuj7GbV7R5CsS9aFYwi3tz8oLaScPYT5JALaAEBXJRatFHRfZtJPp4WDJ9bKDpvwD8P4dv23pDD2Kfr8vi9xW9zF4FkZqdEMq3q1J3g5risnCn7FiJkrKxG5Prc2SSPZhDUJpLsFB51SJ3BbNVL59Ztjaz5vTcTr4o7xqmUmUdnR8WBWj9MhQbGCF99T5QsTA8pYw2vviMc1Kjvmao1Wdh49ow1rEemyZPkqEE6vFQwuGTZbgXJH8d5UGcSPwG8FbJqKGsfYb
# dfx canister call $SOL_PROVIDER_CANISTER_ID sol_sendRawTransaction "(\"${RAW_TX}\")" --ic

# SIG=2VGvopAP2NinJ48fpPKae9svtHcAYw6K1mUyW2GDyEyW6Dp3mBtTwat1wPfbCnq2G6hkQa8yiQZTf3dEHDWa4erK
# dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${SIG}\,null")" --ic