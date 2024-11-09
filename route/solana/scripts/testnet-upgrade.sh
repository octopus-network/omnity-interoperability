#!/usr/bin/env bash

# disable dfx warning
export DFX_WARNING="-mainnet_plaintext_identity"

echo "Setting up for testnet environment..."
ADMIN=$(dfx identity get-principal --ic)

# Testnet env
HUB_CANISTER_ID=xbuoc-ciaaa-aaaar-qajba-cai
SOL_PROVIDER_CANISTER_ID=xgviw-pqaaa-aaaar-qajbq-cai
SOLANA_ROUTE_CANISTER_ID=xtsz3-oyaaa-aaaar-qajca-cai

SCHNORR_KEY_NAME="test_key_1"
PROXY_URL="https://solana-rpc-proxy-398338012986.us-central1.run.app"
alchemy_d="https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
helius_d="https://devnet.helius-rpc.com/?api-key=174a6ec2-4439-4fca-9277-b12900c71fa5"

echo "testnet environment: 
    admin id: $ADMIN
    schnorr key name: $SCHNORR_KEY_NAME 
    alchemy rpc :  ${alchemy_d}
    proxy url: $PROXY_URL
    omnity_hub canister id: $HUB_CANISTER_ID 
    ic solana provider canister id: $SOL_PROVIDER_CANISTER_ID
    solana route canister id: $SOLANA_ROUTE_CANISTER_ID"


# install or reinstall omnity hub
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
    nodesInSubnet = opt 34; 
    } )" \
    --mode=reinstall -y \
    --wasm=./assets/ic_solana_provider.wasm.gz \
    --ic 

dfx canister status $SOL_PROVIDER_CANISTER_ID --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID debug '(true)' --ic
# test canister api

test_account=3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia
test_sig=2VGvopAP2NinJ48fpPKae9svtHcAYw6K1mUyW2GDyEyW6Dp3mBtTwat1wPfbCnq2G6hkQa8yiQZTf3dEHDWa4erK
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_latestBlockhash "(opt \"${helius_d}\")" --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getAccountInfo "(\"${test_account}\",opt \"${helius_d}\")" --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getSignatureStatuses "(vec {\"${test_sig}\"},opt \"${helius_d}\")" --ic
test_account=B3zfZ9CvfCHd23jzM7UqrVR2sid4y4eJYtxzZA4azqaD
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getBalance "(\"${test_account}\",null)" --ic
echo 

# solana_route canister
SOL_CHAIN_ID="eSolana"
SOL_FEE="SOL"
FEE_ACCOUNT="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
rpc1=https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ
rpc2="https://devnet.helius-rpc.com/?api-key=174a6ec2-4439-4fca-9277-b12900c71fa5"
rpc3=https://rpc.ankr.com/solana_devnet/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
# rpc3=https://nd-471-475-490.p2pify.com/6de0b91c609fb3bd459e043801aa6aa4

echo "reinstall $SOLANA_ROUTE_CANISTER_ID ..."
dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument "(variant { Init = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${SOL_CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_key_name = opt \"${SCHNORR_KEY_NAME}\";\
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
    fee_account= opt \"${FEE_ACCOUNT}\";\
    } })" \
    --mode=reinstall -y \
    --wasm=./assets/solana_route.wasm.gz \
    --ic 

dfx canister status $SOLANA_ROUTE_CANISTER_ID --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID debug '(true)' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID forward '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_forward "(opt \"${helius_d}\")" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID forward '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID multi_rpc_config '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID update_multi_rpc "(record { 
    rpc_list = vec {\"${rpc1}\";
                     \"${rpc2}\";
                     \"${rpc3}\";};\
    minimum_response_count = 2:nat32;})" --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID multi_rpc_config '()' --ic

# test 
KEYTYPE="variant { ChainKey }"
dfx canister call $SOLANA_ROUTE_CANISTER_ID signer "($KEYTYPE)"  --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_latest_blockhash '()' --ic 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction "(\"${test_sig}\",opt \"${helius_d}\")" --ic
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


# add token
PROTO="Bitcoin-runes"
TOKEN_NAME="RUNES•X•BITCOIN"
TIMESTAMP=$(date +"%Y%m%d%H%M")
TOKEN_ID="${PROTO}-${TOKEN_NAME}${TIMESTAMP}"
# TOKEN_SYMBOL=$(echo "$TOKEN_NAME" | grep -oE 'NICE[0-9]+')
TOKEN_SYMBOL="X"
DECIMALS=0
TOKEN_URI="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/x_meta.json"

echo $TOKEN_ID
echo $TOKEN_NAME
echo $TOKEN_SYMBOL
echo $DECIMALS
echo $TOKEN_URI

dfx canister call $HUB_CANISTER_ID validate_proposal "( vec {variant { AddToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${TOKEN_URI}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})" \
        --ic 
dfx canister call $HUB_CANISTER_ID execute_proposal "( vec {variant { AddToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${TOKEN_URI}\"; 
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
KEYTYPE="variant { ChainKey }"
SIGNER=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID signer "($KEYTYPE)"  --ic)
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current SIGNER: $SIGNER"
echo "$SIGNER balance: $(solana balance $SIGNER)"

# req airdrop
# solana airdrop 2
MASTER_KEY=$(solana address)
echo "current solana cli default address: $MASTER_KEY and balance: $(solana balance $MASTER_KEY)"
# transfer SOL to init signer
AMOUNT=0.2
echo "transfer SOL to $SIGNER from $MASTER_KEY"
solana transfer $SIGNER $AMOUNT --with-memo init_account --allow-unfunded-recipient
echo "$SIGNER balance: $(solana balance $SIGNER)"

# start schedule
echo "start_schedule ... " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID start_schedule '(null)' --ic
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
AMOUNT="999999"
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

sleep 60

echo "upgrade $HUB_CANISTER_ID ..."
dfx canister install $HUB_CANISTER_ID --argument '(variant { Upgrade = null })' \
 --mode upgrade -y \
 --wasm=./assets/omnity_hub.wasm.gz \
 --ic
dfx canister status $HUB_CANISTER_ID --ic
dfx canister call $HUB_CANISTER_ID set_logger_filter '("debug")' --ic
echo 

echo "upgrade $SOL_PROVIDER_CANISTER_ID ..."
dfx canister install $SOL_PROVIDER_CANISTER_ID --argument "( record { 
    rpc_url = opt \"${PROXY_URL}\"; 
    schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
    nodesInSubnet = opt 34; 
    } )" \
    --mode=upgrade -y \
    --wasm=./assets/ic_solana_provider.wasm.gz \
    --ic 
dfx canister status $SOL_PROVIDER_CANISTER_ID --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID debug '(true)' --ic
echo

echo "upgrade $SOLANA_ROUTE_CANISTER_ID ..."
dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument '(null)' \
    --mode=upgrade -y \
    --wasm=./assets/solana_route.wasm.gz \
    --ic 

dfx canister status $SOLANA_ROUTE_CANISTER_ID --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID debug '(true)' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID forward '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID multi_rpc_config '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_chain_list '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_token_list '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_redeem_fee '("Bitcoin")' --ic
echo
echo "canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req "(\"${TID}\")" --ic
echo "canister call $SOLANA_ROUTE_CANISTER_ID mint_token_status " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_status "(\"${TID}\")" --ic

echo "canister call $SOLANA_ROUTE_CANISTER_ID get_tickets_from_queue " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_tickets_from_queue '()' --ic
