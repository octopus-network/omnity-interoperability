#!/bin/bash

# start ic local network
dfx start --clean --background > dfx.out 2>&1
dfx canister stop omnity_hub
dfx canister delete omnity_hub

# deploy hub
#dfx deploy omnity_hub
# dfx deploy omnity_hub --mode reinstall -y --specified-id=bkyz2-fmaaa-aaaaa-qaaaq-cai
# dfx canister call omnity_hub set_whitelist '(principal "bkyz2-fmaaa-aaaaa-qaaaq-cai", true)'
# dfx deploy omnity_hub --mode reinstall -y 
dfx canister create omnity_hub
dfx deploy omnity_hub
# add authed canister id

# add chain
# Bitcoin
dfx canister call omnity_hub validate_proposal '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };}})'
dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };}})'
#dfx canister call omnity_hub query_directives '("Bitcoin",null,0:nat64,5:nat64)' 

# Ethereum
dfx canister call omnity_hub validate_proposal '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Ethereum"; chain_type=variant { SettlementChain };}})'
dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Ethereum"; chain_type=variant { SettlementChain };}})'
#dfx canister call omnity_hub query_directives '("Ethereum",null,0:nat64,5:nat64)' 


# ICP
dfx canister call omnity_hub validate_proposal '(variant { AddChain = record { chain_state=variant { Active };chain_id = "ICP"; chain_type=variant { SettlementChain };}})'
dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_id = "ICP"; chain_type=variant { SettlementChain };}})'
#dfx canister call omnity_hub query_directives '("ICP",null,0:nat64,5:nat64)' 

# Arbitrum
dfx canister call omnity_hub validate_proposal '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Arbitrum"; chain_type=variant { ExecutionChain };}})'
dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Arbitrum"; chain_type=variant { ExecutionChain };}})'
dfx canister call omnity_hub query_directives '("Arbitrum",opt variant {AddChain=null},0:nat64,5:nat64)' 

# Optimistic
dfx canister call omnity_hub validate_proposal '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Optimistic"; chain_type=variant { ExecutionChain };}})'
dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Optimistic"; chain_type=variant { ExecutionChain };}})'
dfx canister call omnity_hub query_directives '("Optimistic",opt variant {AddChain=opt variant {ExecutionChain}},0:nat64,5:nat64)' 

# Starknet
dfx canister call omnity_hub validate_proposal '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Starknet"; chain_type=variant { ExecutionChain };}})'
dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_id = "Starknet"; chain_type=variant { ExecutionChain };}})'
dfx canister call omnity_hub query_directives '("Starknet",opt variant {AddChain=opt variant {SettlementChain}},0:nat64,5:nat64)' 

# add token
dfx canister call omnity_hub validate_proposal '(variant { AddToken = record { decimals = 18 : nat8; icon = opt "btc"; token_id = "BTC"; issue_chain = "Bitcoin"; symbol = "BTC";}})'
dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = opt "btc"; token_id = "BTC"; issue_chain = "Bitcoin"; symbol = "BTC";}})'
dfx canister call omnity_hub query_directives '("Ethereum",opt variant {AddToken=null},0:nat64,5:nat64)' 

dfx canister call omnity_hub validate_proposal '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; }})'
dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; }})'
dfx canister call omnity_hub query_directives '("Ethereum",opt variant {AddToken=null},0:nat64,5:nat64)' 

dfx canister call omnity_hub validate_proposal '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "ICP"; symbol = "ICP"; issue_chain = "ICP"; }})'
dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "ICP"; symbol = "ICP"; issue_chain = "ICP"; }})'
dfx canister call omnity_hub query_directives '("ICP",opt variant {AddToken=opt "ETH"},0:nat64,5:nat64)' 

dfx canister call omnity_hub validate_proposal '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "ARB"; symbol = "ARB"; issue_chain = "Arbitrum"; }})'
dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "ARB"; symbol = "ARB"; issue_chain = "Arbitrum"; }})'
dfx canister call omnity_hub query_directives '("Arbitrum",opt variant {AddToken=opt "ICP"},0:nat64,5:nat64)' 

dfx canister call omnity_hub validate_proposal '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "OP"; symbol = "OP"; issue_chain = "Optimistic"; }})'
dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "OP"; symbol = "OP"; issue_chain = "Optimistic"; }})'
dfx canister call omnity_hub query_directives '("Optimistic",opt variant {AddToken=opt "ARB"},0:nat64,5:nat64)' 

dfx canister call omnity_hub validate_proposal '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "StarkNet"; symbol = "StarkNet"; issue_chain = "Starknet"; }})'
dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "StarkNet"; symbol = "StarkNet"; issue_chain = "Starknet"; }})'
dfx canister call omnity_hub query_directives '("Starknet",opt variant {AddToken=opt "OP"},0:nat64,5:nat64)' 

# change chain state
dfx canister call omnity_hub validate_proposal '(variant { ToggleChainState = record { chain_id = "Optimistic"; action = variant { Deactivate };}})'
dfx canister call omnity_hub build_directive '(variant { ToggleChainState = record { chain_id = "Optimistic"; action = variant { Deactivate };}})'
dfx canister call omnity_hub query_directives '("Starknet",opt variant {DeactivateChain},0:nat64,5:nat64)' 


# update fee
dfx canister call omnity_hub update_fee 'record {fee_token = "OP"; dst_chain_id = "Arbitrum"; factor = 12 : int64;}'
dfx canister call omnity_hub query_directives '("Arbitrum",opt variant {UpdateFee=opt "OP"},0:nat64,5:nat64)' 

# A-B tansfer/redeem
# transfer from Bitcoin to Arbitrum
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc0247828f"; ticket_time = 1707291817947 : nat64; token = "BTC"; amount = "88888"; src_chain = "Bitcoin"; dst_chain = "Arbitrum"; action = variant { Transfer }; sender = "address_on_Bitcoin"; receiver = "address_on_Arbitrum"; memo = null;})'
# query ticket
dfx canister call omnity_hub query_tickets '("Arbitrum",0:nat64,5:nat64)'

# redeem from  Arbitrum to Bitcoin
dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21"; ticket_time = 1707291817947 : nat64; token = "BTC"; amount = "88888"; src_chain = "Arbitrum"; dst_chain = "Bitcoin"; action = variant { Redeem }; sender = "address_on_Arbitrum"; receiver = "address_on_Bitcoin"; memo = null;})'
# query ticket
dfx canister call omnity_hub query_tickets '("Bitcoin",0:nat64,5:nat64)'

# A-B-C tansfer/redeem
# transfer from Ethereum to Optimistic
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc0247828f"; ticket_time = 1707291817947 : nat64; token = "ETH"; amount = "6666"; src_chain = "Ethereum"; dst_chain = "Optimistic"; action = variant { Transfer }; sender = "address_on_Ethereum"; receiver = "address_on_Optimistic"; memo = null;})'
# query ticket
dfx canister call omnity_hub query_tickets '("Optimistic",0:nat64,5:nat64)'

# transfer from  Optimistic to Starknet
dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21"; ticket_time = 1707291817947 : nat64; token = "ETH"; amount = "6666"; src_chain = "Optimistic"; dst_chain = "Starknet"; action = variant { Transfer }; sender = "address_on_Optimistic"; receiver = "address_on_Starknet"; memo = null;})'
# query ticket
dfx canister call omnity_hub query_tickets '("Starknet",0:nat64,5:nat64)'

# redeem from Starknet to Optimistic
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc0247828f"; ticket_time = 1707291817947 : nat64; token = "ETH"; amount = "6666"; src_chain = "Starknet"; dst_chain = "Optimistic"; action = variant { Redeem }; sender = "address_on_Starknet"; receiver = "address_on_Optimistic"; memo = null;})'
# query ticket
dfx canister call omnity_hub query_tickets '("Optimistic",0:nat64,5:nat64)'

# redeem from  Optimistic to Ethereum
dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21"; ticket_time = 1707291817947 : nat64; token = "ETH"; amount = "6666"; src_chain = "Optimistic"; dst_chain = "Ethereum"; action = variant { Redeem }; sender = "address_on_Optimistic"; receiver = "address_on_Ethereum"; memo = null;})'
# query ticket
dfx canister call omnity_hub query_tickets '("Ethereum",0:nat64,5:nat64)'

dfx stop