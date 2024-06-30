#!/bin/bash

set -euo pipefail
trap "echo 'error: Script failed: see failed command above'" ERR
export DFX_WARNING="-mainnet_plaintext_identity"
ID=vp-test

dfx canister stop bitcoin_mock --ic --identity $ID
# dfx canister delete bitcoin_mock --ic --identity $ID

dfx canister stop icp_mock --ic --identity $ID
# dfx canister delete icp_mock --ic --identity $ID

# dfx canister stop omnity_hub --ic --identity $ID
# dfx canister delete omnity_hub --ic --identity $ID

# deploy hub
dfx deploy omnity_hub  --mode reinstall --argument '(variant { Init = record { admin = principal "rv3oc-smtnf-i2ert-ryxod-7uj7v-j7z3q-qfa5c-bhz35-szt3n-k3zks-fqe"} })' --ic --identity $ID --yes
# dfx canister install --mode install --wasm ./scripts/omnity_hub.wasm.gz --argument '(variant { Init = record { admin = principal "rv3oc-smtnf-i2ert-ryxod-7uj7v-j7z3q-qfa5c-bhz35-szt3n-k3zks-fqe"} })' --yes omnity_hub

INIT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID  2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"



# sub topic
dfx canister call omnity_hub sub_directives '(opt "Bitcoin", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })' --ic --identity $ID
dfx canister call omnity_hub sub_directives '(opt "Ethereum", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })' --ic --identity $ID
dfx canister call omnity_hub sub_directives '(opt "eICP", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })' --ic --identity $ID
dfx canister call omnity_hub sub_directives '(opt "Arbitrum", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })' --ic --identity $ID
dfx canister call omnity_hub sub_directives '(opt "Optimistic", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })' --ic --identity $ID
dfx canister call omnity_hub sub_directives '(opt "Starknet", vec {variant {AddChain};variant {UpdateChain}; variant {AddToken}; variant {UpdateToken}; variant {UpdateFee} ;variant {ToggleChainState} })' --ic --identity $ID

dfx canister call omnity_hub query_subscribers '(null)' --ic --identity $ID
# add chain
# Bitcoin
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=null;counterparties=null; fee_token= null}}})' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=null;counterparties=null; fee_token= null}}})' --ic --identity $ID

# Ethereum
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Ethereum"; chain_type=variant { SettlementChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Ethereum constract address"; counterparties= opt vec {"Bitcoin"}; fee_token= null}}})' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Ethereum"; chain_type=variant { SettlementChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Ethereum constract address"; counterparties= opt vec {"Bitcoin"};  fee_token= null}}})' --ic --identity $ID

# ICP
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "bkyz2-fmaaa-aaafa-qadaab-cai"; counterparties= opt vec {"Bitcoin";"Ethereum"};  fee_token= opt "LICP" }}})' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain }; canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "bkyz2-fmaaa-aaafa-qadaab-cai"; counterparties= opt vec {"Bitcoin";"Ethereum"};  fee_token=  opt "LICP" }}})' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "eICP",null,0:nat64,5:nat64)' --ic --identity $ID

# Arbitrum
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Arbitrum"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Arbitrum constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP"}; fee_token= opt "ARB"}}} )' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Arbitrum"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Arbitrum constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP"}; fee_token= opt "ARB"}}} )' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "Arbitrum",opt variant {AddChain},0:nat64,5:nat64)' --ic --identity $ID

# Optimistic
dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Optimistic"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Optimistic constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP";"Arbitrum"}; fee_token=opt "OP"}}} )' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Optimistic"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Optimistic constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP";"Arbitrum"}; fee_token=opt "OP"}}} )' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "Optimistic",opt variant {AddChain},0:nat64,5:nat64)' --ic --identity $ID

# Starknet
dfx canister call omnity_hub validate_proposal '( vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Starknet"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Starknet constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP";"Arbitrum";"Optimistic"}; fee_token= opt "Starknet"}}} )' --ic --identity $ID
dfx canister call omnity_hub execute_proposal  '( vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Starknet"; chain_type=variant { ExecutionChain };canister_id="bkyz2-fmaaa-aaaaa-qaaaq-cai"; contract_address=opt "Starknet constract address"; counterparties= opt vec {"Bitcoin";"Ethereum";"eICP";"Arbitrum";"Optimistic"}; fee_token= opt "Starknet"}}} )' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "Starknet",opt variant {AddChain},0:nat64,5:nat64)' --ic --identity $ID

# add token

# BTC
dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "btc.logo.url"; token_id = "Bitcoin-runes-HOPE•YOU•GET•RICH"; name = "HOPE•YOU•GET•RICH"; issue_chain = "Bitcoin"; symbol = "BTC"; metadata = vec{ record {"rune_id"; "40000:846"}}; dst_chains = vec {"Ethereum"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"}}}} )' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "btc.logo.url"; token_id = "Bitcoin-runes-HOPE•YOU•GET•RICH"; name = "HOPE•YOU•GET•RICH"; issue_chain = "Bitcoin"; symbol = "BTC"; metadata = vec{ record {"rune_id"; "40000:846"}}; dst_chains = vec {"Ethereum"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"}}}} )' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "Ethereum",opt variant {AddToken},0:nat64,5:nat64)' --ic --identity $ID

# ETH
dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "eth.logo.url"; token_id = "ETH"; name = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; metadata = vec{}; dst_chains = vec {"Bitcoin"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"} }}})' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 18 : nat8; icon = opt "eth.logo.url"; token_id = "ETH"; name = "ETH"; symbol = "ETH"; issue_chain = "Ethereum"; metadata = vec{};  dst_chains = vec {"Bitcoin"; "eICP"; "Arbitrum"; "Optimistic"; "Starknet"} }}})' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "Ethereum",opt variant {AddToken},0:nat64,5:nat64)' --ic --identity $ID

# ICP
dfx canister call omnity_hub validate_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "icp.logo.url"; token_id = "LICP"; name = "LICP"; symbol = "LICP"; issue_chain = "eICP"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "Arbitrum"; "Optimistic"; "Starknet"}}}})' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "icp.logo.url"; token_id = "LICP"; name = "LICP"; symbol = "LICP"; issue_chain = "eICP"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "Arbitrum"; "Optimistic"; "Starknet"}}}})' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "eICP",opt variant {AddToken},0:nat64,5:nat64)' --ic --identity $ID

# ARB
dfx canister call omnity_hub validate_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "arb.logo.url"; token_id = "ARB"; name = "ARB"; symbol = "ARB"; issue_chain = "Arbitrum"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Optimistic"; "Starknet"}}}})' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '( vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "arb.logo.url"; token_id = "ARB"; name = "ARB"; symbol = "ARB"; issue_chain = "Arbitrum"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Optimistic"; "Starknet"}}}})' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "Arbitrum",opt variant {AddToken},0:nat64,5:nat64)' --ic --identity $ID

# OP 
dfx canister call omnity_hub validate_proposal '(vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "op.logo.url"; token_id = "OP"; name = "OP"; symbol = "OP"; issue_chain = "Optimistic"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Starknet"} }}})' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '(vec { variant { AddToken = record { decimals = 18 : nat8; icon = opt "op.logo.url"; token_id = "OP"; name = "OP"; symbol = "OP"; issue_chain = "Optimistic"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Starknet"} }}})' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "Optimistic",opt variant {AddToken},0:nat64,5:nat64)' --ic --identity $ID

# StarkNet
dfx canister call omnity_hub validate_proposal '(vec{ variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "Starknet"; name = "Starknet"; symbol = "StarkNet"; issue_chain = "Starknet"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Optimistic"}}}})' --ic --identity $ID
dfx canister call omnity_hub execute_proposal '(vec{ variant { AddToken = record { decimals = 18 : nat8; icon = null; token_id = "Starknet"; name = "Starknet"; symbol = "StarkNet"; issue_chain = "Starknet"; metadata = vec{ }; dst_chains = vec {"Bitcoin"; "Ethereum"; "eICP"; "Arbitrum"; "Optimistic"}}}})' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "Starknet",opt variant {AddToken},0:nat64,5:nat64)' --ic --identity $ID


# update fee
dfx canister call omnity_hub update_fee 'vec {variant { UpdateTargetChainFactor = record {target_chain_id="Bitcoin"; target_chain_factor=1000 : nat}}; variant { UpdateFeeTokenFactor = record { fee_token="LICP"; fee_token_factor=60000000000 : nat}}}' --ic --identity $ID
dfx canister call omnity_hub query_directives '(opt "eICP",null,0:nat64,12:nat64)' --ic --identity $ID

# A-B tansfer/redeem
# transfer from Bitcoin to Arbitrum
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc0247828f"; ticket_type = variant { Normal }; ticket_time = 1715654809737051178 : nat64; token = "Bitcoin-runes-HOPE•YOU•GET•RICH"; amount = "88888"; src_chain = "Bitcoin"; dst_chain = "Arbitrum"; action = variant { Transfer }; sender = opt "address_on_Bitcoin"; receiver = "address_on_Arbitrum"; memo = null; })' --ic --identity $ID
dfx canister call omnity_hub query_tickets '(opt "Arbitrum",0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(null,null,0:nat64,5:nat64)' --ic --identity $ID

# redeem from  Arbitrum to Bitcoin
dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21";  ticket_type = variant { Normal };  ticket_time = 1715654809737051179 : nat64; token = "Bitcoin-runes-HOPE•YOU•GET•RICH"; amount = "88888"; src_chain = "Arbitrum"; dst_chain = "Bitcoin"; action = variant { Redeem }; sender = opt "address_on_Arbitrum"; receiver = "address_on_Bitcoin"; memo = null;})' --ic --identity $ID
dfx canister call omnity_hub query_tickets '(opt "Bitcoin",0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub query_tickets '(opt "Arbitrum",0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(opt "Arbitrum",null,0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(opt "Bitcoin",null,0:nat64,5:nat64)' --ic --identity $ID

# A-B-C tansfer/redeem
# transfer from Ethereum to Optimistic
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e89-b41d-76bc024782e8f";  ticket_type = variant { Normal };  ticket_time = 1715654809737051180 : nat64; token = "ETH"; amount = "6666"; src_chain = "Ethereum"; dst_chain = "Optimistic"; action = variant { Transfer }; sender = opt "address_on_Ethereum"; receiver = "address_on_Optimistic"; memo = null;})' --ic --identity $ID
dfx canister call omnity_hub query_tickets '(opt "Optimistic",0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)' --ic --identity $ID

# transfer from  Optimistic to Starknet
dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5eg6c84c21";  ticket_type = variant { Normal };  ticket_time = 1715654809737051181 : nat64; token = "ETH"; amount = "6666"; src_chain = "Optimistic"; dst_chain = "Starknet"; action = variant { Transfer }; sender = opt "address_on_Optimistic"; receiver = "address_on_Starknet"; memo = null;})' --ic --identity $ID
dfx canister call omnity_hub query_tickets '(opt "Starknet",0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(opt "Starknet",null,0:nat64,5:nat64)' --ic --identity $ID


# redeem from Starknet to Optimistic
dfx canister call omnity_hub send_ticket '(record { ticket_id = "28b47548-55dc-4e8f9-b41d-76bc0247828f";  ticket_type = variant { Normal }; ticket_time = 1715654809737051182 : nat64; token = "ETH"; amount = "6666"; src_chain = "Starknet"; dst_chain = "Optimistic"; action = variant { Redeem }; sender = opt "address_on_Starknet"; receiver = "address_on_Optimistic"; memo = null;})' --ic --identity $ID
dfx canister call omnity_hub query_tickets '(opt "Optimistic",0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(opt "Starknet",null,0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)' --ic --identity $ID


# redeem from  Optimistic to Ethereum
dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40hea-80c2-4cf5e6c84c21";  ticket_type = variant { Normal }; ticket_time = 1715654809737051183 : nat64; token = "ETH"; amount = "6666"; src_chain = "Optimistic"; dst_chain = "Ethereum"; action = variant { Redeem }; sender = opt "address_on_Optimistic"; receiver = "address_on_Ethereum"; memo = null;})' --ic --identity $ID
dfx canister call omnity_hub query_tickets '(opt "Ethereum",0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(opt "Optimistic",null,0:nat64,5:nat64)' --ic --identity $ID
dfx canister call omnity_hub get_chain_tokens '(opt "Starknet",null,0:nat64,5:nat64)' --ic --identity $ID


# must build 
# dfx build omnity_hub
# # upgrade canister
# dfx canister install --mode upgrade --argument '(variant { Upgrade = null })' omnity_hub --ic --identity $ID --yes 
# dfx canister call omnity_hub sync_ticket_size '()' --ic --identity $ID
# dfx canister call omnity_hub sync_tickets '(0:nat64,12:nat64)' --ic --identity $ID
# dfx stop


echo "call omnity_hub query_directives_instructions '(opt "eICP",null,0:nat64,12:nat64)'"
dfx canister call omnity_hub query_directives_instructions '(opt "eICP",null,0:nat64,12:nat64)' --ic --identity $ID
echo "call omnity_hub query_directives_from_map_instructions '(opt "eICP",null,0:nat64,12:nat64)'"
dfx canister call omnity_hub query_directives_from_map_instructions '(opt "eICP",null,0:nat64,12:nat64)' --ic --identity $ID
echo "call omnity_hub query_directives_from_mix_instructions '(opt "eICP",null,0:nat64,12:nat64)'"
dfx canister call omnity_hub query_directives_from_mix_instructions '(opt "eICP",null,0:nat64,12:nat64)' --ic --identity $ID


echo "call omnity_hub query_directives_instructions '(opt "eICP",null,12:nat64,24:nat64)'"
dfx canister call omnity_hub query_directives_instructions '(opt "eICP",null,12:nat64,24:nat64)' --ic --identity $ID
echo "call omnity_hub query_directives_from_map_instructions '(opt "eICP",null,12:nat64,24:nat64)'"
dfx canister call omnity_hub query_directives_from_map_instructions '(opt "eICP",null,12:nat64,24:nat64)' --ic --identity $ID
echo "call omnity_hub query_directives_from_mix_instructions '(opt "eICP",null,12:nat64,24:nat64)'"
dfx canister call omnity_hub query_directives_from_mix_instructions '(opt "eICP",null,12:nat64,24:nat64)' --ic --identity $ID


# dfx canister call omnity_hub mock_call_query_directives '(opt "eICP",null,0:nat64,12:nat64)' --ic --identity $ID
# dfx canister call omnity_hub mock_call_query_directives '(opt "eICP",null,12:nat64,24:nat64)' --ic --identity $ID
echo "call omnity_hub query_tickets_instructions '(opt "Bitcoin",0:nat64,6:nat64)'"
dfx canister call omnity_hub query_tickets_instructions '(opt "Bitcoin",0:nat64,6:nat64)' --ic --identity $ID
echo "call omnity_hub query_tickets_from_map_instructions '(opt "Bitcoin",0:nat64,6:nat64)'"
dfx canister call omnity_hub query_tickets_from_map_instructions '(opt "Bitcoin",0:nat64,6:nat64)' --ic --identity $ID
echo "call omnity_hub query_tickets_from_mix_instructions '(opt "Bitcoin",0:nat64,6:nat64)'"
dfx canister call omnity_hub query_tickets_from_mix_instructions '(opt "Bitcoin",0:nat64,6:nat64)' --ic --identity $ID

echo "call omnity_hub query_tickets_instructions '(opt "Bitcoin",6:nat64,12:nat64)'"
dfx canister call omnity_hub query_tickets_instructions '(opt "Bitcoin",6:nat64,12:nat64)' --ic --identity $ID
echo "call omnity_hub query_tickets_from_map_instructions '(opt "Bitcoin",6:nat64,12:nat64)'"
dfx canister call omnity_hub query_tickets_from_map_instructions '(opt "Bitcoin",6:nat64,12:nat64)' --ic --identity $ID
echo "call omnity_hub query_tickets_from_mix_instructions '(opt "Bitcoin",6:nat64,12:nat64)'"
dfx canister call omnity_hub query_tickets_from_mix_instructions '(opt "Bitcoin",6:nat64,12:nat64)' --ic --identity $ID

# dfx canister call omnity_hub mock_call_query_tickets '(opt "Bitcoin",0:nat64,6:nat64)' --ic --identity $ID
# dfx canister call omnity_hub mock_call_query_tickets '(opt "Bitcoin",6:nat64,12:nat64)' --ic --identity $ID

CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "omnity_hub beging balance: $INIT_BALANCE cycles "
echo "omnity_hub current balance: $CURRENT_BALANCE cycles"

CONSUMPTION=$(bc <<< "$INIT_BALANCE - $CURRENT_BALANCE")
echo "Util now,the consumption is: $CONSUMPTION cycles"

TIME=30 
con_record="consumption.txt"
rm $con_record

PRE_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"

echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') omnity_hub current balance: $PRE_BALANCE cycles" >> $con_record
sleep $TIME 
CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') after $TIME seconds,omnity_hub current balance: $CURRENT_BALANCE cycles" >> $con_record
CONSUMPTION=$(bc <<< "$PRE_BALANCE - $CURRENT_BALANCE")
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') the consumption that without query_call is: $CONSUMPTION cycles" >> $con_record

dfx canister start bitcoin_mock --ic --identity $ID
dfx canister start icp_mock --ic --identity $ID
