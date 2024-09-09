#!/usr/bin/env bash

if [ -z "$1" ]; then
  echo "Usage: $0 {testnet|product}"
  exit 1
fi

ADMIN=$(dfx identity get-principal --ic)

case "$1" in
  testnet)
    echo "Setting up for testnet environment..."

    # Testnet env
    # HUB_CANISTER_ID=xykho-eiaaa-aaaag-qjrka-cai
    HUB_CANISTER_ID=arlph-jyaaa-aaaak-ak2oa-cai
    SCHNORR_CANISTER_ID=aaaaa-aa
    SCHNORR_KEY_NAME="test_key_1"
    # SCHNORR_KEY_NAME="key_1"
    # SOLANA_RPC_URL="https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
    SOLANA_RPC_URL="https://solana-rpc-proxy-398338012986.us-central1.run.app"
    SOL_PROVIDER_CANISTER_ID=l3ka6-4yaaa-aaaar-qahpa-cai
    SOLANA_ROUTE_CANISTER_ID=lvinw-hiaaa-aaaar-qahoa-cai
    echo "testnet environment: 
          admin id: $ADMIN
          omnity_hub canister id: $HUB_CANISTER_ID 
          schnorr canister id: $SCHNORR_CANISTER_ID 
          schnorr key name: $SCHNORR_KEY_NAME 
          ic solana provider rpc: $SOLANA_RPC_URL
          ic solana provider canister id: $SOL_PROVIDER_CANISTER_ID
          solana route canister id: $SOLANA_ROUTE_CANISTER_ID"

    ;;

  product)
    echo "Setting up for production environment..."

    # Production env
    HUB_CANISTER_ID=7wupf-wiaaa-aaaar-qaeya-cai
    SCHNORR_CANISTER_ID=aaaaa-aa
    SCHNORR_KEY_NAME="key_1"
    # SOLANA_RPC_URL="https://solana-mainnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
    SOLANA_RPC_URL="https://solana-rpc-proxy-398338012986.us-central1.run.app"
    SOL_PROVIDER_CANISTER_ID=l3ka6-4yaaa-aaaar-qahpa-cai
    SOLANA_ROUTE_CANISTER_ID=lvinw-hiaaa-aaaar-qahoa-cai
    
    echo "production environment: 
          admin id: $ADMIN
          omnity_hub canister id: $HUB_CANISTER_ID 
          schnorr canister id: $SCHNORR_CANISTER_ID 
          schnorr key name: $SCHNORR_KEY_NAME 
          ic solana provider rpc: $SOLANA_RPC_URL
          ic solana provider canister id: $SOL_PROVIDER_CANISTER_ID
          solana route canister id: $SOLANA_ROUTE_CANISTER_ID"
    ;;

  *)
    echo "Invalid environment specified. Use 'testnet' or 'product'."
    exit 1
    ;;
esac

# disable warning
export DFX_WARNING="-mainnet_plaintext_identity"

# install or reinstall omnity hub
# create canister for omnity hub
# dfx canister create omnity_hub --ic
echo "reinstall $HUB_CANISTER_ID ..."
dfx canister install $HUB_CANISTER_ID --argument "(variant { Init = record { admin = principal \"${ADMIN}\" } })" \
  --mode=reinstall -y \
  --wasm=./assets/omnity_hub.wasm.gz \
  --ic

# upgrade hub
# dfx canister install $HUB_CANISTER_ID --argument '(variant { Upgrade = null })' \
#     --mode=upgrade -y \
#     --wasm=./assets/omnity_hub.wasm.gz \
#     --ic 

# change log level for debugging
dfx canister call $HUB_CANISTER_ID set_logger_filter '("debug")' --ic
dfx canister status $HUB_CANISTER_ID --ic
echo 

echo "reinstall $SOL_PROVIDER_CANISTER_ID ..."
dfx canister install $SOL_PROVIDER_CANISTER_ID --argument "( record { 
    rpc_url = opt \"${SOLANA_RPC_URL}\"; 
    nodesInSubnet = opt 28; 
    schnorr_canister = opt \"${SCHNORR_CANISTER_ID}\"; 
    schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
    } )" \
    --mode=reinstall -y \
    --wasm=./assets/ic_solana_provider.wasm.gz \
    --ic 

# echo "upgrade $SOL_PROVIDER_CANISTER_ID ..."
# dfx canister install $SOL_PROVIDER_CANISTER_ID --argument "( record { 
#     rpc_url = opt \"${SOLANA_RPC_URL}\"; 
#     nodesInSubnet = opt 28; 
#     schnorr_canister = opt \"${SCHNORR_CANISTER_ID}\"; 
#     schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
#     } )" \
#     --mode=upgrade -y \
#     --wasm=./assets/ic_solana_provider.wasm.gz \
#     --ic 

dfx canister status $SOL_PROVIDER_CANISTER_ID --ic
# test get blockhash
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_latestBlockhash '()' --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getAccountInfo '("3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia")' --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getSignatureStatuses '(vec {"4kogo438gk3CT6pifHQa7d4CC7HRidnG2o6EWxwGFvAcuSC7oTeG3pWTYDy9wuCYmGxJe1pRdTHf7wMcnJupXSf4"})' --ic
echo 


# solana_route canister
SOL_CHAIN_ID="Solana"
SOL_FEE="SOL"
FEE_ACCOUNT="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"

echo "reinstall $SOLANA_ROUTE_CANISTER_ID ..."
dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument "(variant { Init = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${SOL_CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_canister = opt principal \"${SCHNORR_CANISTER_ID}\";\
    schnorr_key_name = \"${SCHNORR_KEY_NAME}\";\
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
    fee_account= opt \"${FEE_ACCOUNT}\"; 
    } })" \
    --mode=reinstall -y \
    --wasm=./assets/solana_route.wasm.gz \
    --ic 

# echo "upgrade $SOLANA_ROUTE_CANISTER_ID ..."
# dfx canister install $SOLANA_ROUTE_CANISTER_ID --argument "(variant { Upgrade = record { \
#     admin = principal \"${ADMIN}\";\
#     chain_id=\"${SOL_CHAIN_ID}\";\
#     hub_principal= principal \"${HUB_CANISTER_ID}\";\
#     chain_state= variant { Active }; \
#     schnorr_canister = opt principal \"${SCHNORR_CANISTER_ID}\";\
#     schnorr_key_name = \"${SCHNORR_KEY_NAME}\";\
#     sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
#     fee_account= opt \"${FEE_ACCOUNT}\"; 
#     } })" \
#     --mode=upgrade -y \
#     --wasm=./assets/solana_route.wasm.gz \
#     --ic 

dfx canister status $SOLANA_ROUTE_CANISTER_ID --ic

# add perms
dfx canister call $SOLANA_ROUTE_CANISTER_ID set_permissions "(
    principal \"kp4gp-pefsb-gau5l-p2hf6-pagac-3jusw-lzc2v-nsxtq-46dnk-ntffe-3qe\",\
    variant { Update }
    )" \
    --ic 

# query signer and init it
SIGNER=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic)
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current SIGNER: $SIGNER"
echo "$SIGNER balance: $(solana balance $SIGNER)"

# req airdrop
solana airdrop 2
MASTER_KEY=$(solana address)
echo "current solana cli default address: $MASTER_KEY and balance: $(solana balance $MASTER_KEY)"
# transfer SOL to init signer
AMOUNT=0.5
echo "transfer SOL to $SIGNER from $MASTER_KEY"
solana transfer $SIGNER $AMOUNT --with-memo init_account --allow-unfunded-recipient
echo "$SIGNER balance: $(solana balance $SIGNER)"

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
# add bitcoin
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

# update fee
dfx canister call $HUB_CANISTER_ID update_fee "vec {variant { UpdateTargetChainFactor = 
        record { target_chain_id=\"${BITCOIN_CHAIN_ID}\"; 
                 target_chain_factor=10000 : nat}}; 
                 variant { UpdateFeeTokenFactor = record { fee_token=\"${SOL_FEE}\"; 
                                                 fee_token_factor=1 : nat}}}" \
        --ic 

dfx canister call $HUB_CANISTER_ID query_directives "(opt \"${SOL_CHAIN_ID}\",opt variant {UpdateFee},0:nat64,12:nat64)" --ic 

# token info
TOKEN_ID_PRE="Bitcoin-runes-HOPE•YOU•GET•USBL"
TOKEN_NAME_PRE="HOPE•YOU•GET•USBL"
TOKEN_SYMBOL_PRE="USBL"
DECIMALS=2
ICON="https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json"

# ticket info
TID_PRE="28b47548-55dc-4e89-b41d-76bc0247828e"
MINT_AMOUNT="2222222222"
SOL_RECEIVER="FDR2mUpiHKFonnwbUujLyhuNTt7LHEjZ1hDFX4UuCngt"

total_calls=10
for i in $(seq 1 $total_calls); do
  TOKEN_ID=${TOKEN_ID_PRE}$i
  TOKEN_NAME=${TOKEN_NAME_PRE}$i
  TOKEN_SYMBOL=${TOKEN_SYMBOL_PRE}$i
  TID=${TID_PRE}$i
  echo ${TOKEN_ID}
  echo ${TOKEN_NAME}
  echo ${TOKEN_SYMBOL}
  echo ${TID}
  echo "Executing add token $i..."
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
   
   # add ticket 
   echo "Executing send ticket $i..."
   dfx canister call $HUB_CANISTER_ID send_ticket "(record { ticket_id = \"${TID}\"; 
        ticket_type = variant { Normal }; 
        ticket_time = 1715654809737051178 : nat64; 
        token = \"${TOKEN_ID}\"; 
        amount = \"${MINT_AMOUNT}\"; 
        src_chain = \"${BITCOIN_CHAIN_ID}\"; 
        dst_chain = \"${SOL_CHAIN_ID}\"; 
        action = variant { Transfer }; 
        sender = null; 
        receiver = \"${SOL_RECEIVER}\";
        memo = null; })" \
    --ic
done

sleep 30
# start schedule
echo "start_schedule ... " 
dfx canister call $SOLANA_ROUTE_CANISTER_ID start_schedule '()' --ic
echo "waiting for query directives or tickets from hub to solana route"

# cannel schedule
# dfx canister call $SOLANA_ROUTE_CANISTER_ID cancel_schedule '()' --ic
