#!/bin/bash

BITCOIN_CHAIN_ID="Bitcoin"
BITCOIN_CID="be2us-64aaa-aaaaa-qaabq-cai"

SOL_CHAIN_ID="Solana"
SOL_CID=$(dfx canister id solana_route)
SOL_FEE="SOL"

# req airdrop
solana airdrop 2
MASTER_KEY=$(solana address)
echo "current solana cli default address: $MASTER_KEY and balance: $(solana balance $MASTER_KEY)"
# get signer and init it
# KEYTYPE="variant { Native }"
KEYTYPE="variant { ChainKey }"
dfx canister call solana_route update_key_type "($KEYTYPE)" 
dfx canister call solana_route query_key_type "($KEYTYPE)" 
SIGNER=$(dfx canister call solana_route signer "($KEYTYPE)" --candid ./assets/solana_route.did)
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current SIGNER: $SIGNER"
# transfer SOL to init signer
AMOUNT=0.2
echo "transfer SOL to $SIGNER from $MASTER_KEY"
solana transfer $SIGNER $AMOUNT --with-memo init_account --allow-unfunded-recipient
echo "$SIGNER balance: $(solana balance $SIGNER)"

echo "Init done!"

# change log level for debugging
dfx canister call omnity_hub set_logger_filter '("debug")'

# sub topic
dfx canister call omnity_hub sub_directives "(opt \"${BITCOIN_CHAIN_ID}\", 
        vec {variant {AddChain};variant {UpdateChain}; 
        variant {AddToken}; variant {UpdateToken}; 
        variant {UpdateFee} ;variant {ToggleChainState} })"
dfx canister call omnity_hub sub_directives "(opt \"${SOL_CHAIN_ID}\", 
        vec {variant {AddChain};variant {UpdateChain}; 
        variant {AddToken}; variant {UpdateToken}; 
        variant {UpdateFee} ;variant {ToggleChainState} })"
dfx canister call omnity_hub query_subscribers '(null)'

# add bitcoin
dfx canister call omnity_hub validate_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${BITCOIN_CHAIN_ID}\"; chain_type=variant { SettlementChain }; 
        canister_id=\"${BITCOIN_CID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${SOL_CHAIN_ID}\"}; 
        fee_token=null}}})"
dfx canister call omnity_hub execute_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${BITCOIN_CHAIN_ID}\"; chain_type=variant { SettlementChain }; 
        canister_id=\"${BITCOIN_CID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${SOL_CHAIN_ID}\"};
        fee_token=null}}})"
dfx canister call omnity_hub query_directives "(opt \"${SOL_CHAIN_ID}\",opt variant {AddChain},0:nat64,5:nat64)"

# add solana
dfx canister call omnity_hub validate_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${SOL_CHAIN_ID}\"; chain_type=variant { ExecutionChain }; 
        canister_id=\"${SOL_CID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${BITCOIN_CHAIN_ID}\"}; 
        fee_token=opt \"${SOL_FEE}\"}}})"
dfx canister call omnity_hub execute_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${SOL_CHAIN_ID}\"; chain_type=variant { ExecutionChain }; 
        canister_id=\"${SOL_CID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${BITCOIN_CHAIN_ID}\"}; 
        fee_token=opt \"${SOL_FEE}\"}}})"
dfx canister call omnity_hub query_directives "(opt \"${BITCOIN_CHAIN_ID}\",opt variant {AddChain},0:nat64,5:nat64)"

# add token
PROTO="Bitcoin-runes"
TOKEN_NAME_PRE="RUNES•X•BITCOIN"
TOKEN_ID_PRE="${PROTO}-${TOKEN_NAME}"
# TOKEN_SYMBOL=$(echo "$TOKEN_NAME" | grep -oE 'NICE[0-9]+')
TOKEN_SYMBOL_PRE="X"
DECIMALS=0
TOKEN_URI="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/x_meta.json"

# ticket info
TID_PRE="28b47548-55dc-4e89-b41d-76bc0247828e"
MINT_AMOUNT="999999"
SOL_RECEIVER="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"

total_calls=10
for i in $(seq 1 $total_calls); do
  TOKEN_ID=${TOKEN_ID_PRE}$i
  TOKEN_NAME=${TOKEN_NAME_PRE}$i
  TOKEN_SYMBOL=${TOKEN_SYMBOL_PRE}$i
  echo ${TOKEN_ID}
  echo ${TOKEN_NAME}
  echo ${TOKEN_SYMBOL}

  # add token
  echo "Executing add token $TOKEN_ID ..."
  dfx canister call omnity_hub execute_proposal "( vec {variant { AddToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${TOKEN_URI}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})" 
   
  # add tickets
 for i in $(seq 1 $total_calls); do
        TID=${TOKEN_ID}-${TID_PRE}$i
        echo "Executing send ticket $TID ..."
        dfx canister call omnity_hub send_ticket "(record { ticket_id = \"${TID}\"; 
                ticket_type = variant { Normal }; 
                ticket_time = 1715654809737051178 : nat64; 
                token = \"${TOKEN_ID}\"; 
                amount = \"${MINT_AMOUNT}\"; 
                src_chain = \"${BITCOIN_CHAIN_ID}\"; 
                dst_chain = \"${SOL_CHAIN_ID}\"; 
                action = variant { Transfer }; 
                sender = null; 
                receiver = \"${SOL_RECEIVER}\";
                memo = null; })" 
 done

done

sleep 30
# start schedule
echo "start_schedule ... " 
dfx canister call solana_route start_schedule '()' 
echo "waiting for query directives or tickets from hub to solana route"
