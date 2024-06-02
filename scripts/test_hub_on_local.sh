#!/bin/bash

set -euo pipefail
trap "echo 'error: Script failed: see failed command above'" ERR

export timestamp=$(date -u +%s)

# get canister id from cid file
HUB_CANISTER_ID="$(bat process | grep cid | cut -d'"' -f 2)"
HUB_CANDID=../hub/omnity_hub.did

# sub topic
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID sub_directives '(opt "Bitcoin", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID sub_directives '(opt "Ethereum", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID sub_directives '(opt "eICP", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID sub_directives '(opt "Arbitrum", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID sub_directives '(opt "Optimistic", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID sub_directives '(opt "Starknet", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_subscribers '(null)'
# add chain
# Bitcoin
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=null;counterparties=null; fee_token= null}}})'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=null;counterparties=null; fee_token= null}}})'

#dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Bitcoin",null,0:nat64,5:nat64)' 

# Ethereum
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Ethereum"; chain_type=variant { SettlementChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Ethereum constract address"; counterparties= opt vec {"Bitcoin"}; fee_token= null}}})'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Ethereum"; chain_type=variant { SettlementChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Ethereum constract address"; counterparties= opt vec {"Bitcoin"};  fee_token= null}}})'

#dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Ethereum",null,0:nat64,5:nat64)' 



# ICP
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "bkyz2-fmaaa-aaafa-qadaab-cai"; counterparties= opt vec {"Bitcoin";"Ethereum"};  fee_token= opt "LICP" }}})'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "bkyz2-fmaaa-aaafa-qadaab-cai"; counterparties= opt vec {"Bitcoin";"Ethereum"};  fee_token=  opt "LICP" }}})'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "eICP",null,0:nat64,5:nat64)' 

# Arbitrum
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Arbitrum"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Arbitrum constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP"}; fee_token= opt "ARB"}}} )'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Arbitrum"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Arbitrum constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP"}; fee_token= opt "ARB"}}} )'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Arbitrum",opt variant {AddChain},0:nat64,5:nat64)' 

# Optimistic
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Optimistic"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Optimistic constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP";"Arbitrum"}; fee_token=opt "OP"}}} )'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Optimistic"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Optimistic constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP";"Arbitrum"}; fee_token=opt "OP"}}} )'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Optimistic",opt variant {AddChain},0:nat64,5:nat64)' 

# Starknet
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '( vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Starknet"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Starknet constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP";"Arbitrum";"Optimistic"}; fee_token= opt "Starknet"}}} )'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal  '( vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Starknet"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Starknet constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP";"Arbitrum";"Optimistic"}; fee_token= opt "Starknet"}}} )'

dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Starknet",opt variant {AddChain},0:nat64,5:nat64)' 

# add token

# BTC
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "btc.logo.url"; token_id = "Bitcoin-runes-HOPE•YOU•GET•RICH"; name = "HOPE•YOU•GET•RICH"; issue_chain = "Bitcoin"; symbol = "BTC"; metadata = vec{ record {"rune_id"; "40000:846"}}; dst_chains = vec {"Ethereum"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"}}}} )'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "btc.logo.url"; token_id = "Bitcoin-runes-HOPE•YOU•GET•RICH"; name = "HOPE•YOU•GET•RICH"; issue_chain = "Bitcoin"; symbol = "BTC"; metadata = vec{ record {"rune_id"; "40000:846"}}; dst_chains = vec {"Ethereum"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"}}}} )'
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID add_token  '( vec { record { decimals = 18 : nat8; icon = opt "btc.logo.url"; token_id = "Bitcoin-RUNES-150:1"; name = "150:1"; issue_chain = "Bitcoin"; symbol = "BTC"; metadata = vec{ record {"rune_id"; "150:1"}}; dst_chains = vec {"Ethereum"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"}}} )'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Ethereum",opt variant {AddToken},0:nat64,5:nat64)' 

# ETH
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "eth.logo.url"; token_id = "ETH"; name = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; metadata = vec{}; dst_chains = vec {"Bitcoin"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"} }}})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "eth.logo.url"; token_id = "ETH"; name = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; metadata = vec{};  dst_chains = vec {"Bitcoin"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"} }}})'
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID add_token '( vec { record { decimals = 18 : nat8; icon = opt "eth.logo.url"; token_id = "ETH"; name = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; metadata = vec{};  dst_chains = vec {"Bitcoin"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"} }})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Ethereum",opt variant {AddToken},0:nat64,5:nat64)' 

# ICP
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "icp.logo.url"; token_id = "LICP"; name = "LICP"; symbol = "LICP"; issue_chain = "eICP"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "Arbitrum"; "Optimistic"; "Starknet"}}}})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "icp.logo.url"; token_id = "LICP"; name = "LICP"; symbol = "LICP"; issue_chain = "eICP"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "Arbitrum"; "Optimistic"; "Starknet"}}}})'
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID add_token '( vec {  record { decimals = 18 : nat8; icon = opt "icp.logo.url"; token_id = "LICP"; name = "LICP"; symbol = "LICP"; issue_chain = "eICP"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "Arbitrum"; "Optimistic"; "Starknet"}}})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "eICP",opt variant {AddToken},0:nat64,5:nat64)' 

# ARB
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "arb.logo.url"; token_id = "ARB"; name = "ARB"; symbol = "ARB"; issue_chain = "Arbitrum"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Optimistic"; "Starknet"}}}})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "arb.logo.url"; token_id = "ARB"; name = "ARB"; symbol = "ARB"; issue_chain = "Arbitrum"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Optimistic"; "Starknet"}}}})'
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID add_token '( vec {  record { decimals = 18 : nat8; icon = opt "arb.logo.url"; token_id = "ARB"; name = "ARB"; symbol = "ARB"; issue_chain = "Arbitrum"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Optimistic"; "Starknet"}}})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Arbitrum",opt variant {AddToken},0:nat64,5:nat64)' 

# OP 
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '(vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "op.logo.url"; token_id = "OP"; name = "OP"; symbol = "OP"; issue_chain = "Optimistic"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Starknet"} }}})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '(vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "op.logo.url"; token_id = "OP"; name = "OP"; symbol = "OP"; issue_chain = "Optimistic"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Starknet"} }}})'
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID add_token '(vec { record { decimals = 18 : nat8; icon = opt "op.logo.url"; token_id = "OP"; name = "OP"; symbol = "OP"; issue_chain = "Optimistic"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Starknet"} }})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Optimistic",opt variant {AddToken},0:nat64,5:nat64)' 

# StarkNet
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '(vec{ variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "Starknet"; name = "Starknet"; symbol = "StarkNet"; issue_chain = "Starknet"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Optimistic"}}}})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '(vec{ variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "Starknet"; name = "Starknet"; symbol = "StarkNet"; issue_chain = "Starknet"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Optimistic"}}}})'
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID add_token '(vec{ record { decimals = 18 : nat8; icon = null; token_id = "Starknet"; name = "Starknet"; symbol = "StarkNet"; issue_chain = "Starknet"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Optimistic"}}})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Starknet",opt variant {AddToken},0:nat64,5:nat64)' 

# change chain state
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID validate_proposal '( vec { variant { ToggleChainState = record { chain_id = "Optimistic"; action = variant { Deactivate };}}})'
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID execute_proposal '( vec { variant { ToggleChainState = record { chain_id = "Optimistic"; action = variant { Deactivate };}}})'
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "Starknet",opt variant {DeactivateChain},0:nat64,5:nat64)' 


# update fee
# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID update_fee 'vec {record {fee_token = "OP"; dst_chain_id = "Arbitrum"; target_chain_factor = 12 : nat; fee_token_factor = 12 : nat;}}'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID update_fee 'vec {variant { UpdateTargetChainFactor = record {target_chain_id="Bitcoin"; target_chain_factor=1000 : nat}}; variant { UpdateFeeTokenFactor = record { fee_token="LICP"; fee_token_factor=60000000000 : nat}}}'

# dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "ICP",opt variant {UpdateFee=opt "ICP"},0:nat64,5:nat64)' 
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_directives '(opt "eICP",null,0:nat64,12:nat64)' 

# A-B tansfer/redeem
# transfer from Bitcoin to Arbitrum
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc0247828f"; ticket_type = variant { Normal }; ticket_time = 1715654809737051178 : nat64; token = "Bitcoin-runes-HOPE•YOU•GET•RICH"; amount = "88888"; src_chain = "Bitcoin"; dst_chain = "Arbitrum"; action = variant { Transfer }; sender = opt "address_on_Bitcoin"; receiver = "address_on_Arbitrum"; memo = null; })'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_tickets '(opt "Arbitrum",0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(null,null,0:nat64,5:nat64)'

# redeem from  Arbitrum to Bitcoin
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21";  ticket_type = variant { Normal };  ticket_time = 1715654809737051179 : nat64; token = "Bitcoin-runes-HOPE•YOU•GET•RICH"; amount = "88888"; src_chain = "Arbitrum"; dst_chain = "Bitcoin"; action = variant { Redeem }; sender = opt "address_on_Arbitrum"; receiver = "address_on_Bitcoin"; memo = null;})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_tickets '(opt "Bitcoin",0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_tickets '(opt "Arbitrum",0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(opt "Arbitrum",null,0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(opt "Bitcoin",null,0:nat64,5:nat64)'

# A-B-C tansfer/redeem
# transfer from Ethereum to Optimistic
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc024782e8f";  ticket_type = variant { Normal };  ticket_time = 1715654809737051180 : nat64; token = "ETH"; amount = "6666"; src_chain = "Ethereum"; dst_chain = "Optimistic"; action = variant { Transfer }; sender = opt "address_on_Ethereum"; receiver = "address_on_Optimistic"; memo = null;})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_tickets '(opt "Optimistic",0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)'

# transfer from  Optimistic to Starknet
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5eg6c84c21";  ticket_type = variant { Normal };  ticket_time = 1715654809737051181 : nat64; token = "ETH"; amount = "6666"; src_chain = "Optimistic"; dst_chain = "Starknet"; action = variant { Transfer }; sender = opt "address_on_Optimistic"; receiver = "address_on_Starknet"; memo = null;})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_tickets '(opt "Starknet",0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(opt "Starknet",null,0:nat64,5:nat64)'


# redeem from Starknet to Optimistic
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID send_ticket '(record { ticket_id = "28b47548-55dc-4e8f9-b41d-76bc0247828f";  ticket_type = variant { Normal }; ticket_time = 1715654809737051182 : nat64; token = "ETH"; amount = "6666"; src_chain = "Starknet"; dst_chain = "Optimistic"; action = variant { Redeem }; sender = opt "address_on_Starknet"; receiver = "address_on_Optimistic"; memo = null;})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_tickets '(opt "Optimistic",0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(opt "Starknet",null,0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)'


# redeem from  Optimistic to Ethereum
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID send_ticket '(record { ticket_id = "f8aee1cc-db7a-40hea-80c2-4cf5e6c84c21";  ticket_type = variant { Normal }; ticket_time = 1715654809737051183 : nat64; token = "ETH"; amount = "6666"; src_chain = "Optimistic"; dst_chain = "Ethereum"; action = variant { Redeem }; sender = opt "address_on_Optimistic"; receiver = "address_on_Ethereum"; memo = null;})'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID query_tickets '(opt "Ethereum",0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID get_chain_tokens '(opt "Starknet",null,0:nat64,5:nat64)'

# must build 
# dfx build $HUB_CANISTER_ID
# upgrade canister
# dfx canister install --mode upgrade --argument '(variant { Upgrade = null })' --yes $HUB_CANISTER_ID 
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID sync_ticket_size '()'
dfx canister call --candid $HUB_CANDID $HUB_CANISTER_ID sync_tickets '(0:nat64,12:nat64)'
# dfx stop

# profiling
ic-repl ./perf_hub_on_local.sh;