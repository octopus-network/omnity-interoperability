#!/bin/bash

# start ic local network
dfx stop
dfx start --clean --background > dfx.out 2>&1
dfx canister stop omnity_hub
dfx canister delete omnity_hub

# deploy hub
#dfx deploy omnity_hub
# dfx deploy omnity_hub --mode reinstall -y --specified-id=bkyz2-fmaaa-aaaaa-qaaaq-cai
# dfx canister call omnity_hub set_whitelist '(principal "bkyz2-fmaaa-aaaaa-qaaaq-cai", true)'
# dfx deploy omnity_hub --mode reinstall -y 
dfx canister create omnity_hub
# dfx deploy omnity_hub
# dfx identity --identity default get-principal
# output: rv3oc-smtnf-i2ert-ryxod-7uj7v-j7z3q-qfa5c-bhz35-szt3n-k3zks-fqe
dfx deploy omnity_hub --argument '(variant { Init = record { admin = principal "rv3oc-smtnf-i2ert-ryxod-7uj7v-j7z3q-qfa5c-bhz35-szt3n-k3zks-fqe"} })'
# sub topic
dfx canister call omnity_hub sub_directives '(opt "Bitcoin", vec {variant {AddChain};variant {AddToken}; variant {UpdateFee} ;variant {ToggleChainState} })'
dfx canister call omnity_hub sub_directives '(opt "Ethereum", vec {variant {AddChain};variant {AddToken}; variant {UpdateFee} ;variant {ToggleChainState} })'
dfx canister call omnity_hub sub_directives '(opt "ICP", vec {variant {AddChain};variant {AddToken}; variant {UpdateFee} ;variant {ToggleChainState} })'
dfx canister call omnity_hub sub_directives '(opt "Arbitrum", vec {variant {AddChain};variant {AddToken}; variant {UpdateFee} ;variant {ToggleChainState} })'
dfx canister call omnity_hub sub_directives '(opt "Optimistic", vec {variant {AddChain};variant {AddToken}; variant {UpdateFee} ;variant {ToggleChainState}})'
dfx canister call omnity_hub sub_directives '(opt "Starknet", vec {variant {AddChain};variant {AddToken}; variant {UpdateFee} ;variant {ToggleChainState} })'

dfx canister call omnity_hub query_subscribers '(null)'
# add chain
# Bitcoin
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=null;counterparties=null; fee_token= null}}})'

dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=null;counterparties=null; fee_token= null}}})'

#dfx canister call omnity_hub query_directives '(opt "Bitcoin",null,0:nat64,5:nat64)' 

# Ethereum
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Ethereum"; chain_type=variant { SettlementChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Ethereum constract address"; counterparties= opt vec {"Bitcoin"}; fee_token= null}}})'

dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Ethereum"; chain_type=variant { SettlementChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Ethereum constract address"; counterparties= opt vec {"Bitcoin"};  fee_token= null}}})'

#dfx canister call omnity_hub query_directives '(opt "Ethereum",null,0:nat64,5:nat64)' 



# ICP
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "ICP"; chain_type=variant { ExecutionChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "bkyz2-fmaaa-aaafa-qadaab-cai"; counterparties= opt vec {"Bitcoin";"Ethereum"};  fee_token= opt "ICP" }}})'

dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "ICP"; chain_type=variant { ExecutionChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "bkyz2-fmaaa-aaafa-qadaab-cai"; counterparties= opt vec {"Bitcoin";"Ethereum"};  fee_token=  opt "ICP" }}})'

dfx canister call omnity_hub query_directives '(opt "ICP",null,0:nat64,5:nat64)' 

# Arbitrum
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Arbitrum"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Arbitrum constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"ICP"}; fee_token= opt "ARB"}}} )'

dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Arbitrum"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Arbitrum constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"ICP"}; fee_token= opt "ARB"}}} )'

dfx canister call omnity_hub query_directives '(opt "Arbitrum",opt variant {AddChain},0:nat64,5:nat64)' 

# Optimistic
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Optimistic"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Optimistic constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"ICP";"Arbitrum"}; fee_token=opt "OP"}}} )'

dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Optimistic"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Optimistic constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"ICP";"Arbitrum"}; fee_token=opt "OP"}}} )'

dfx canister call omnity_hub query_directives '(opt "Optimistic",opt variant {AddChain},0:nat64,5:nat64)' 

# Starknet
dfx canister call omnity_hub validate_proposal '( vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Starknet"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Starknet constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"ICP";"Arbitrum";"Optimistic"}; fee_token= opt "Starknet"}}} )'

dfx canister call omnity_hub execute_proposal  '( vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Starknet"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Starknet constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"ICP";"Arbitrum";"Optimistic"}; fee_token= opt "Starknet"}}} )'

dfx canister call omnity_hub query_directives '(opt "Starknet",opt variant {AddChain},0:nat64,5:nat64)' 

# add token

# BTC
dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "btc.logo.url"; token_id = "Bitcoin-RUNES-150:1"; name = "150:1"; issue_chain = "Bitcoin"; symbol = "BTC"; metadata = vec{ record {"rune_id"; "150:1"}}; dst_chains = vec {"Ethereum"; "ICP"; "Arbitrum"; "Optimistic"; "Starknet"}}}} )'

dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "btc.logo.url"; token_id = "Bitcoin-RUNES-150:1"; name = "150:1"; issue_chain = "Bitcoin"; symbol = "BTC"; metadata = vec{ record {"rune_id"; "150:1"}}; dst_chains = vec {"Ethereum"; "ICP"; "Arbitrum"; "Optimistic"; "Starknet"}}}} )'

dfx canister call omnity_hub query_directives '(opt "Ethereum",opt variant {AddToken},0:nat64,5:nat64)' 

# ETH
dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "eth.logo.url"; token_id = "ETH"; name = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; metadata = vec{}; dst_chains = vec {"Bitcoin"; "ICP"; "Arbitrum"; "Optimistic"; "Starknet"} }}})'

dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "eth.logo.url"; token_id = "ETH"; name = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; metadata = vec{};  dst_chains = vec {"Bitcoin"; "ICP"; "Arbitrum"; "Optimistic"; "Starknet"} }}})'

dfx canister call omnity_hub query_directives '(opt "Ethereum",opt variant {AddToken},0:nat64,5:nat64)' 

# ICP
dfx canister call omnity_hub validate_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "icp.logo.url"; token_id = "ICP"; name = "ICP"; symbol = "ICP"; issue_chain = "ICP"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "Arbitrum"; "Optimistic"; "Starknet"}}}})'

dfx canister call omnity_hub execute_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "icp.logo.url"; token_id = "ICP"; name = "ICP"; symbol = "ICP"; issue_chain = "ICP"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "Arbitrum"; "Optimistic"; "Starknet"}}}})'

dfx canister call omnity_hub query_directives '(opt "ICP",opt variant {AddToken},0:nat64,5:nat64)' 

# ARB
dfx canister call omnity_hub validate_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "arb.logo.url"; token_id = "ARB"; name = "ARB"; symbol = "ARB"; issue_chain = "Arbitrum"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "ICP"; "Optimistic"; "Starknet"}}}})'

dfx canister call omnity_hub execute_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "arb.logo.url"; token_id = "ARB"; name = "ARB"; symbol = "ARB"; issue_chain = "Arbitrum"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "ICP"; "Optimistic"; "Starknet"}}}})'

dfx canister call omnity_hub query_directives '(opt "Arbitrum",opt variant {AddToken},0:nat64,5:nat64)' 

# OP 
dfx canister call omnity_hub validate_proposal '(vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "op.logo.url"; token_id = "OP"; name = "OP"; symbol = "OP"; issue_chain = "Optimistic"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "ICP"; "Arbitrum"; "Starknet"} }}})'

dfx canister call omnity_hub execute_proposal '(vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "op.logo.url"; token_id = "OP"; name = "OP"; symbol = "OP"; issue_chain = "Optimistic"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "ICP"; "Arbitrum"; "Starknet"} }}})'

dfx canister call omnity_hub query_directives '(opt "Optimistic",opt variant {AddToken},0:nat64,5:nat64)' 

# StarkNet
dfx canister call omnity_hub validate_proposal '(vec{ variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "Starknet"; name = "Starknet"; symbol = "StarkNet"; issue_chain = "Starknet"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "ICP"; "Arbitrum"; "Optimistic"}}}})'

dfx canister call omnity_hub execute_proposal '(vec{ variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "Starknet"; name = "Starknet"; symbol = "StarkNet"; issue_chain = "Starknet"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "ICP"; "Arbitrum"; "Optimistic"}}}})'

dfx canister call omnity_hub query_directives '(opt "Starknet",opt variant {AddToken},0:nat64,5:nat64)' 

# change chain state
# dfx canister call omnity_hub validate_proposal '( vec { variant { ToggleChainState = record { chain_id = "Optimistic"; action = variant { Deactivate };}}})'
# dfx canister call omnity_hub execute_proposal '( vec { variant { ToggleChainState = record { chain_id = "Optimistic"; action = variant { Deactivate };}}})'
# dfx canister call omnity_hub query_directives '(opt "Starknet",opt variant {DeactivateChain},0:nat64,5:nat64)' 


# update fee
# dfx canister call omnity_hub update_fee 'vec {record {fee_token = "OP"; dst_chain_id = "Arbitrum"; target_chain_factor = 12 : nat; fee_token_factor = 12 : nat;}}'
dfx canister call omnity_hub update_fee 'vec {variant { UpdateTargetChainFactor = record {target_chain_id="Bitcoin"; target_chain_factor=1000 : nat}}; variant { UpdateFeeTokenFactor = record { fee_token="ICP"; fee_token_factor=60000000000 : nat}}}'

# dfx canister call omnity_hub query_directives '(opt "ICP",opt variant {UpdateFee=opt "ICP"},0:nat64,5:nat64)' 
dfx canister call omnity_hub query_directives '(opt "ICP",null,0:nat64,12:nat64)' 

# A-B tansfer/redeem
# transfer from Bitcoin to Arbitrum
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc0247828f"; ticket_type = variant { Normal }; ticket_time = 1715654809737051178 : nat64; token = "Bitcoin-RUNES-150:1"; amount = "88888"; src_chain = "Bitcoin"; dst_chain = "Arbitrum"; action = variant { Transfer }; sender = opt "address_on_Bitcoin"; receiver = "address_on_Arbitrum"; memo = null; status= variant { WaitingForConfirmByDest };})'
dfx canister call omnity_hub query_tickets '(opt "Arbitrum",0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(null,null,0:nat64,5:nat64)'

# redeem from  Arbitrum to Bitcoin
dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21";  ticket_type = variant { Normal };  ticket_time = 1715654809737051179 : nat64; token = "Bitcoin-RUNES-150:1"; amount = "88888"; src_chain = "Arbitrum"; dst_chain = "Bitcoin"; action = variant { Redeem }; sender = opt "address_on_Arbitrum"; receiver = "address_on_Bitcoin"; memo = null;status= variant { WaitingForConfirmByDest };})'
dfx canister call omnity_hub query_tickets '(opt "Bitcoin",0:nat64,5:nat64)'
dfx canister call omnity_hub query_tickets '(opt "Arbitrum",0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(opt "Arbitrum",null,0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(opt "Bitcoin",null,0:nat64,5:nat64)'

# A-B-C tansfer/redeem
# transfer from Ethereum to Optimistic
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc024782e8f";  ticket_type = variant { Normal };  ticket_time = 1715654809737051180 : nat64; token = "ETH"; amount = "6666"; src_chain = "Ethereum"; dst_chain = "Optimistic"; action = variant { Transfer }; sender = opt "address_on_Ethereum"; receiver = "address_on_Optimistic"; memo = null;status= variant { WaitingForConfirmByDest };})'
dfx canister call omnity_hub query_tickets '(opt "Optimistic",0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)'

# transfer from  Optimistic to Starknet
dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5eg6c84c21";  ticket_type = variant { Normal };  ticket_time = 1715654809737051181 : nat64; token = "ETH"; amount = "6666"; src_chain = "Optimistic"; dst_chain = "Starknet"; action = variant { Transfer }; sender = opt "address_on_Optimistic"; receiver = "address_on_Starknet"; memo = null;status= variant { WaitingForConfirmByDest };})'
dfx canister call omnity_hub query_tickets '(opt "Starknet",0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(opt "Starknet",null,0:nat64,5:nat64)'


# redeem from Starknet to Optimistic
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e8f9-b41d-76bc0247828f";  ticket_type = variant { Normal }; ticket_time = 1715654809737051182 : nat64; token = "ETH"; amount = "6666"; src_chain = "Starknet"; dst_chain = "Optimistic"; action = variant { Redeem }; sender = opt "address_on_Starknet"; receiver = "address_on_Optimistic"; memo = null;status= variant { WaitingForConfirmByDest };})'
dfx canister call omnity_hub query_tickets '(opt "Optimistic",0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(opt "Starknet",null,0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)'


# redeem from  Optimistic to Ethereum
dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40hea-80c2-4cf5e6c84c21";  ticket_type = variant { Normal }; ticket_time = 1715654809737051183 : nat64; token = "ETH"; amount = "6666"; src_chain = "Optimistic"; dst_chain = "Ethereum"; action = variant { Redeem }; sender = opt "address_on_Optimistic"; receiver = "address_on_Ethereum"; memo = null;status= variant { WaitingForConfirmByDest };})'
dfx canister call omnity_hub query_tickets '(opt "Ethereum",0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)'
dfx canister call omnity_hub get_chain_tokens '(opt "Starknet",null,0:nat64,5:nat64)'


# dfx stop