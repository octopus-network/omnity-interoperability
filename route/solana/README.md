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
# Deploy schnorr canister
dfx deploy schnorr_canister --ic
SCHNORR_CANISTER_ID=$(dfx canister id schnorr_canister --ic)
echo "Schnorr canister id: $SCHNORR_CANISTER_ID" 
# for product
# SCHNORR_CANISTER_ID="aaaaa-aa"
# SCHNORR_KEY_NAME="key_1"
# 
# for testnet
SCHNORR_KEY_NAME="test_key_1"

# Deploy the ic solana provider canister
# for product
# SOLANA_RPC_URL="mainet"
# 
# for testnet
SOLANA_RPC_URL="testnet"
dfx deploy ic-solana-provider --argument "( record { 
    nodesInSubnet = 28; 
    schnorr_canister = opt \"${SCHNORR_CANISTER_ID}\"; 
    schnorr_key_name= opt \"${SCHNORR_KEY_NAME}\"; 
    rpc_url = opt \"${SOLANA_RPC_URL}\"; 
    } )" --ic 
SOL_PROVIDER_CANISTER_ID=$(dfx canister id ic-solana-provider --ic)
echo "solana provide canister id: $SOL_PROVIDER_CANISTER_ID"

# Deploy solana_route
# get admin id
ADMIN=$(dfx identity get-principal --ic)
echo "admin id: $ADMIN"

# get omnity hub canister id
HUB_CANISTER_ID=$(dfx canister id omnity_hub --ic)
echo "Omnity hub canister id: $HUB_CANISTER_ID"
echo 

CHAIN_ID="Solana"
# replace the fee account
FEE_ACCOUNT="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
dfx deploy solana_route --argument "(variant { Init = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_canister = opt principal \"${SCHNORR_CANISTER_ID}\";\
    schnorr_key_name = null; \
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
    fee_account= opt \"${FEE_ACCOUNT}\"; 
} })" --ic 

SOLANA_ROUTE_CANISTER_ID=$(dfx canister id solana_route --ic)
echo "Solana route canister id: $SOLANA_ROUTE_CANISTER_ID"

```

### Init the signer account
```bash
# get signer from solana route
SIGNER=$(dfx canister call solana_route signer '()')
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current signer: $SIGNER"
# init the signer via cli
# Note: install solana-cli first or transfer SOL to signer from wallet app,like Phantom
AMOUNT=2
solana transfer $SIGNER $AMOUNT --with-memo init_account --allow-unfunded-recipient
echo "$SIGNER balance: $(solana balance $SIGNER)"
```

### Start solana route schedule to query directives and tickets from hub 
```bash
dfx canister call solana_route start_schedule '()' 
```

### Upgrade the solana route canister
```bash

dfx deploy solana_route --argument "(variant { UpgradeArgs = record { \
    admin = principal \"${ADMIN}\";\
    chain_id=\"${CHAIN_ID}\";\
    hub_principal= principal \"${HUB_CANISTER_ID}\";\
    chain_state= variant { Active }; \
    schnorr_canister = principal \"${SCHNORR_CANISTER_ID}\";\
    schnorr_key_name = null; \
    sol_canister = principal \"${SOL_PROVIDER_CANISTER_ID}\";\
     fee_account= opt \"${FEE_ACCOUNT}\"; 
} })" --mode upgrade --ic

```
