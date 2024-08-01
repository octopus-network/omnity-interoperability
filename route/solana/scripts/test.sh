#!/usr/bin/env bash

BITCOIN_CHAIN_ID="Bitcoin"
SOL_CHAIN_ID="Solana"
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"

# start schedule 
dfx canister call solana_route start_schedule '()' 

# wait for query directives or tickets from hub to solana route
sleep 15

echo "check sync directive from hub "
dfx canister call solana_route get_chain_list '()' 
dfx canister call solana_route get_token_list '()' 
dfx canister call solana_route get_redeem_fee '("Bitcoin")' 
echo

# A-B tansfer/redeem
echo "Transfer from Bitcoin to Solana ..."
echo 
TID="28b47548-55dc-4e89-b41d-76bc0247828f"
AMOUNT="88888"
SOL_RECEIVER="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
dfx canister call omnity_hub send_ticket "(record { ticket_id = \"${TID}\"; 
        ticket_type = variant { Normal }; 
        ticket_time = 1715654809737051178 : nat64; 
        token = \"${TOKEN_ID}\"; 
        amount = \"${AMOUNT}\"; 
        src_chain = \"${BITCOIN_CHAIN_ID}\"; 
        dst_chain = \"${SOL_CHAIN_ID}\"; 
        action = variant { Transfer }; 
        sender = null; 
        receiver = \"${SOL_RECEIVER}\"; memo = null; })"
dfx canister call omnity_hub query_tickets "(opt \"${SOL_CHAIN_ID}\",0:nat64,5:nat64)"
echo 
sleep 5

echo "canister call solana_route get_tickets_from_queue "
dfx canister call solana_route get_tickets_from_queue '()' 
echo 
sleep 10

# cannel schedule
# dfx canister call solana_route cannel_schedule '()' 

# manual handle tickets (transfer from customs to solana )
#dfx canister call solana_route handle_tickets '()' 

# TOKEN_NAME="HOPE•YOU•GET•RICH"
# TOKEN_SYMBOL="RICH"
# DECIMALS=2
# ICON="https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json"

# dfx canister call solana_route create_mint "(record {
#         name=\"${TOKEN_NAME}\";
#         symbol=\"${TOKEN_SYMBOL}\";
#         decimals=${DECIMALS}:nat8;
#         uri=\"${ICON}\";
# })"
#dfx canister call solana_route get_or_create_aossicated_account '("3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia","Gi6BuNTXVgjhXzCjHg5m8Xz2jDQAyDDBi9F77p23ekYi")' 
#dfx canister call solana_route mint_to '("3SHvMPs5kMZvV5ZE3rDfSZe1wHKbbt2ptWRvoc4t3nnF",888888:nat64,"Gi6BuNTXVgjhXzCjHg5m8Xz2jDQAyDDBi9F77p23ekYi")' 


# TODO: mock redeem from solana to customs
# burn token
# spl-token burn 3XQYiRJgCWJQbE18NnocAujKzDHwKBuiJDYziXovUogf 88  --with-memo D58qMHmDAoEaviG8s9VmGwRhcw2z1apJHt6RnPtgxdVj  --owner  ~/.config/solana/boern.json
# generate ticket
# dfx canister call solana_route generate_ticket '(record {
#         signature="zNNo7sS2JWKJwkzMZaUshVUEXTUzTcCF3BsUU7Uo9WaqkZHHyxp3fz187Ku7NukyTr2WJ1CDjhHHepMCJzt24uR";
#         action = variant { Redeem };
#         token_id = "Bitcoin-runes-HOPE•YOU•GET•RICH";
#         target_chain_id = "Bitcoin";
#         sender = "3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia";
#         receiver = "D58qMHmDAoEaviG8s9VmGwRhcw2z1apJHt6RnPtgxdVj";
#         amount = 88:nat64;
#         memo = null;
#         })'
# upgrade canister
# echo "upgrade omnity hub ..."
# dfx canister install --mode upgrade --argument '(variant { Upgrade = null })'  --upgrade-unchanged --yes omnity_hub 
