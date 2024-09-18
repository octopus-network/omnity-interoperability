## Local deployment and testing
### make build
    build the solana route canister
### make deploy
    deploy the schnorr_canister,ic-solana-provider and solana_route
### make init
    init test data,includes chain,token,fee etc.
### make test
    mock transfer and redeem 

## Mainnet deployment and upgrade
### Build solana route canister 
```bash
cd route/solana
dfx canister create solana_route --ic
dfx build ic_solana --ic
candid-extractor ./target/wasm32-unknown-unknown/release/solana_route.wasm > ./assets/solana_route.did
```

### Deploy solana route and it`s deps
```bash
SCHNORR_CANISTER_ID="aaaaa-aa"
SCHNORR_KEY_NAME="key_1"
SOLANA_RPC_URL="https://solana-rpc-proxy-398338012986.us-central1.run.app"

# deploy solana provider
dfx deploy ic-solana-provider --argument "( record { 
    rpc_url = opt \"${SOLANA_RPC_URL}\"; 
    nodesInSubnet = opt 28; 
    schnorr_canister = opt \"${SCHNORR_CANISTER_ID}\"; 
    schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
    } )" --ic 
SOL_PROVIDER_CANISTER_ID=$(dfx canister id ic-solana-provider --ic)
echo "solana provide canister id: $SOL_PROVIDER_CANISTER_ID"

# test solana provider
dfx canister status $SOL_PROVIDER_CANISTER_ID --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_latestBlockhash '()' --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getAccountInfo '("3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia")' --ic
dfx canister call $SOL_PROVIDER_CANISTER_ID sol_getSignatureStatuses '(vec {"4kogo438gk3CT6pifHQa7d4CC7HRidnG2o6EWxwGFvAcuSC7oTeG3pWTYDy9wuCYmGxJe1pRdTHf7wMcnJupXSf4"})' --ic
echo 

# deploy solana_route
# get admin id
ADMIN=$(dfx identity get-principal --ic)
echo "admin id: $ADMIN"

# get omnity hub canister id
HUB_CANISTER_ID=7wupf-wiaaa-aaaar-qaeya-cai
echo "Omnity hub canister id: $HUB_CANISTER_ID"
echo 

SOL_CHAIN_ID="Solana"
# TODO:replace the fee account
FEE_ACCOUNT="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
dfx deploy solana_route --argument "(variant { Init = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${SOL_CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_canister = opt principal \"${SCHNORR_CANISTER_ID}\";\
    schnorr_key_name = null; \
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
    fee_account= opt \"${FEE_ACCOUNT}\"; 
} })" --ic 

SOLANA_ROUTE_CANISTER_ID=$(dfx canister id solana_route --ic)
echo "Solana route canister id: $SOLANA_ROUTE_CANISTER_ID"

# test solana route
dfx canister status $SOLANA_ROUTE_CANISTER_ID --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_latest_blockhash '()' --ic 
dfx canister call $SOLANA_ROUTE_CANISTER_ID get_transaction '("4kogo438gk3CT6pifHQa7d4CC7HRidnG2o6EWxwGFvAcuSC7oTeG3pWTYDy9wuCYmGxJe1pRdTHf7wMcnJupXSf4",null)' --ic

```

### Add solana chain info to hub
```bash
# sub directives for solana
dfx canister call $HUB_CANISTER_ID sub_directives "(opt \"${SOL_CHAIN_ID}\",
         vec {variant {AddChain};variant {UpdateChain};
         variant {AddToken}; variant {UpdateToken};
         variant {UpdateFee} ;variant {ToggleChainState} })" --ic
# check solana sub 
dfx canister call $HUB_CANISTER_ID query_subscribers '(null)' --ic 

# Add solana chain to hub
# TODO: replace real counterparty chain info
COUNTERPARTY_CHAIN_ID="Bitcoin"
#COUNTERPARTY_CHAIN_CANISTER_ID="xykho-eiaaa-aaaag-qjrka-cai"
SOL_FEE="SOL"
dfx canister call $HUB_CANISTER_ID validate_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${SOL_CHAIN_ID}\"; 
        chain_type=variant { ExecutionChain }; 
        canister_id=\"${SOLANA_ROUTE_CANISTER_ID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${COUNTERPARTY_CHAIN_ID}\"}; 
        fee_token=opt \"${SOL_FEE}\"}}})" \
        --ic 
dfx canister call $HUB_CANISTER_ID execute_proposal "(vec {variant { 
        AddChain = record { chain_state=variant { Active }; 
        chain_id = \"${SOL_CHAIN_ID}\"; 
        chain_type=variant { ExecutionChain }; 
        canister_id=\"${SOLANA_ROUTE_CANISTER_ID}\"; 
        contract_address=null; 
        counterparties=opt vec {\"${COUNTERPARTY_CHAIN_ID}\"}; 
        fee_token=opt \"${SOL_FEE}\"}}})" \
        --ic 
# check
dfx canister call $HUB_CANISTER_ID query_directives "(opt \"${COUNTERPARTY_CHAIN_ID}\",opt variant {AddChain},0:nat64,5:nat64)" --ic 

```


### Push counterparty chain info to solana route
```bash
# TODO: replace real counterparty chain info
COUNTERPARTY_CHAIN_ID="Bitcoin"
COUNTERPARTY_CHAIN_CANISTER_ID="xykho-eiaaa-aaaag-qjrka-cai"
dfx canister call $HUB_CANISTER_ID validate_proposal "(vec {variant { 
        UpdateChain = record { chain_state=variant { Active }; 
        chain_id = \"${COUNTERPARTY_CHAIN_ID}\"; 
        chain_type=variant { SettlementChain }; 
        canister_id=\"${COUNTERPARTY_CHAIN_CANISTER_ID}\"; 
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
        chain_id = \"${COUNTERPARTY_CHAIN_ID}\"; 
        chain_type=variant { SettlementChain }; 
        canister_id=\"${COUNTERPARTY_CHAIN_CANISTER_ID}\"; 
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

```

### Push token info to solana
```bash
# token info
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
TOKEN_NAME="HOPE•YOU•GET•RICH"
TOKEN_SYMBOL="RICH"
DECIMALS=2
ICON="https://github.com/octopus-network/omnity-interoperability/blob/feature/solana-route/route/solana/assets/token_metadata.json"

ISSUE_CHAIN_ID="Bitcoin"

dfx canister call $HUB_CANISTER_ID validate_proposal "( vec {variant { UpdateToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${ISSUE_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${ICON}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${ISSUE_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})" \
        --ic 
dfx canister call $HUB_CANISTER_ID execute_proposal "( vec {variant { UpdateToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${ISSUE_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${ICON}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${ISSUE_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})" \
        --ic 

dfx canister call $HUB_CANISTER_ID query_directives "(
    opt \"${ISSUE_CHAIN_ID}\",
    opt variant {AddToken},0:nat64,5:nat64)" \
    --ic

dfx canister call $HUB_CANISTER_ID query_directives "(
    opt \"${SOL_CHAIN_ID}\",
    opt variant {AddToken},0:nat64,5:nat64)" \
    --ic

```

### Push fee info to solana
```bash
dfx canister call $HUB_CANISTER_ID update_fee "vec {variant { UpdateTargetChainFactor = 
        record { target_chain_id=\"${BITCOIN_CHAIN_ID}\"; 
                 target_chain_factor=10000 : nat}}; 
                 variant { UpdateFeeTokenFactor = record { fee_token=\"${SOL_FEE}\"; 
                                                 fee_token_factor=1 : nat}}}" \
        --ic 

dfx canister call $HUB_CANISTER_ID query_directives "(opt \"${SOL_CHAIN_ID}\",opt variant {UpdateFee},0:nat64,12:nat64)" --ic 


```


### Init the signer account
```bash
# get signer from solana route
SIGNER=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID signer '()' --ic)
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current SIGNER: $SIGNER"
echo "$SIGNER balance: $(solana balance $SIGNER -u m)"

# init the signer via cli
# Note: install solana-cli first or transfer SOL to signer from wallet app,like Phantom
AMOUNT=2
solana transfer $SIGNER $AMOUNT --with-memo init_account --allow-unfunded-recipient -u m
echo "$SIGNER balance: $(solana balance $SIGNER -u m)"
```

### Start solana route schedule to query directives and tickets from hub 
```bash
dfx canister call solana_route start_schedule '()' --ic 
```

### Upgrade the solana route canister
```bash

dfx deploy solana_route --argument "(opt variant { UpgradeArgs = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${SOL_CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_canister = principal \"${SCHNORR_CANISTER_ID}\";\
    schnorr_key_name = null; \
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
     fee_account= opt \"${FEE_ACCOUNT}\"; 
} })" --mode upgrade --ic

# or without parameters
#dfx deploy solana_route --argument '(null)' --mode upgrade --ic
```
