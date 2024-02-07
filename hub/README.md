
## deploy omnity hub

```bash
# start ic local network
dfx start --clean
# open other terminal
dfx deploy omnity_hub

```

## add chain

```bash
dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_name = "Bitcoin"; chain_type=variant { SettlementChain };}})'

dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_name = "Ethereum"; chain_type=variant { SettlementChain };}})'

dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_name = "Near"; chain_type=variant { ExecutionChain };}})'

dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_name = "Otto"; chain_type=variant { ExecutionChain };}})'

```

## add token

```bash
dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = opt "btc"; name = "BTC"; issue_chain = "Bitcoin"; symbol = "BTC";}})'

dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = null; name = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; }})'

dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = null; name = "OCT"; symbol = "OCT"; issue_chain = "Near"; }})'

dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = null; name = "OTTO"; symbol = "OTTO"; issue_chain = "Otto"; }})'

```

## change chain state  

```bash
dfx canister call omnity_hub build_directive '(variant { ChangeChainState = record { chain_id = "Otto"; state = variant { Suspend };}})'


```

## update fee

```bash
dfx canister call omnity_hub build_directive '(variant { UpdateFee = record { fee_token = "OTTO"; dst_chain_id = "Near"; factor = 12 : int64;}})'

or

dfx canister call omnity_hub update_fee 'record {fee_token = "OTTO"; dst_chain_id = "Near"; factor = 12 : int64;}'

```

## query directives

```bash
dfx canister call omnity_hub query_directives '("Near",0:nat64,5:nat64)'
```

## send ticket

```bash
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc0247828f"; created_time = 1707291817947 : nat64; token = "ODR"; amount = "88888"; src_chain = "Bitcoin"; dst_chain = "Near"; action = variant { Transfer }; sender = "sdsdfsyiesdfsdfds"; receiver = "sdfsdfsdffdrytrrr"; memo = null;})'

dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21"; created_time = 1707291817947 : nat64; token = "WODR"; amount = "88888"; src_chain = "Near"; dst_chain = "Bitcoin"; action = variant { Redeem }; sender = "sdfsdfsdffdrytrrr"; receiver = "sdsdfsyiesdfsdfds"; memo = null;})'


```

## query tickets

```bash
dfx canister call omnity_hub query_tickets '("Near",0:nat64,5:nat64)'
dfx canister call omnity_hub query_tickets '("Bitcoin",0:nat64,5:nat64)'

```
