#!/bin/bash

BITCOIN_CHAIN_ID="Bitcoin"
BITCOIN_CID="be2us-64aaa-aaaaa-qaabq-cai"

SOL_CHAIN_ID="Solana"
SOL_CID=$(dfx canister id solana_route)
SOL_FEE="SOL"

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
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•POWER"
TOKEN_NAME="HOPE•YOU•GET•POWER"
TOKEN_SYMBOL="POWER"
DECIMALS=2
ICON="https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json"
dfx canister call omnity_hub validate_proposal "( vec {variant { AddToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${ICON}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})"
dfx canister call omnity_hub execute_proposal "( vec {variant { AddToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${ICON}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})"
dfx canister call omnity_hub query_directives "(opt \"${SOL_CHAIN_ID}\",opt variant {AddToken},0:nat64,5:nat64)"

# update fee
dfx canister call omnity_hub update_fee "vec {variant { UpdateTargetChainFactor = 
        record { target_chain_id=\"${BITCOIN_CHAIN_ID}\"; 
                 target_chain_factor=10000 : nat}}; 
                 variant { UpdateFeeTokenFactor = record { fee_token=\"${SOL_FEE}\"; 
                                                 fee_token_factor=1 : nat}}}"

dfx canister call omnity_hub query_directives "(opt \"${SOL_CHAIN_ID}\",null,0:nat64,12:nat64)"

# req airdrop
solana airdrop 2
MASTER_KEY=$(solana address)
echo "current solana cli default address: $MASTER_KEY and balance: $(solana balance $MASTER_KEY)"
# get payer and init it
PAYER=$(dfx canister call solana_route payer '()' --candid ./assets/solana_route.did)
PAYER=$(echo "$PAYER" | awk -F'"' '{print $2}')
echo "current payer: $PAYER"
# transfer SOL to init payer
AMOUNT=0.2
echo "transfer SOL to $PAYER from $MASTER_KEY"
solana transfer $PAYER $AMOUNT --with-memo init_account --allow-unfunded-recipient
echo "$PAYER balance: $(solana balance $PAYER)"

echo "Init done!"