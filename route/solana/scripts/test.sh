#!/usr/bin/env bash

BITCOIN_CHAIN_ID="Bitcoin"
SOL_CHAIN_ID="Solana"
TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•FIVE"

# start schedule 
dfx canister call solana_route start_schedule '()' 

# wait for query directives or tickets from hub to solana route
sleep 60

echo "check sync directive from hub "
dfx canister call solana_route get_chain_list '()' 
dfx canister call solana_route get_token_list '()' 
dfx canister call solana_route get_redeem_fee '("Bitcoin")' 
echo

# A-B tansfer/redeem
echo "mock: transfer from Bitcoin to Solana ..."
echo 
TID="28b47548-55dc-4e89-b41d-76bc0247828f"
AMOUNT="22222222"
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
        receiver = \"${SOL_RECEIVER}\";
        memo = null; })"
dfx canister call omnity_hub query_tickets "(opt \"${SOL_CHAIN_ID}\",0:nat64,5:nat64)"
echo 

sleep 35

echo "canister call solana_route get_tickets_from_queue "
dfx canister call solana_route get_tickets_from_queue '()' 
echo 

sleep 90

# get token mint
TOKEN_MINT=$(dfx canister call solana_route query_mint_address "(\"${TOKEN_ID}\")")
TOKEN_MINT=$(echo "$TOKEN_MINT" | awk -F'"' '{print $2}')
echo "token mint: $TOKEN_MINT"

# get aossicated account based on owner and token mint
ATA=$(dfx canister call solana_route query_aossicated_account_address "(\"${SOL_RECEIVER}\",\"${TOKEN_MINT}\")" )
ATA=$(echo "$ATA" | awk -F'"' '{print $2}')
while [ -z "$ATA" ]; do
  echo "ATA is empty, waiting..."
  sleep 5  
  ATA=$(dfx canister call solana_route query_aossicated_account_address "(\"${SOL_RECEIVER}\",\"${TOKEN_MINT}\")")
  ATA=$(echo "$ATA" | awk -F'"' '{print $2}')
done
echo "The dest address: $SOL_RECEIVER and the token address: $TOKEN_MINT aossicated account is: $ATA"

sleep 15
echo "mock: redeem from solana to customs... "
# first, burn token
CUSTOMS_RECEIVER="D58qMHmDAoEaviG8s9VmGwRhcw2z1apJHt6RnPtgxdVj"
OWNER=~/.config/solana/boern.json
BURN_AMOUNT=111111
echo spl-token burn $ATA $BURN_AMOUNT  --with-memo $CUSTOMS_RECEIVER  --owner $OWNER
# echo $(spl-token burn $ATA $BURN_AMOUNT  --with-memo $CUSTOMS_RECEIVER  --owner $OWNER)
SIGNAURE=$(spl-token burn $ATA $BURN_AMOUNT  --with-memo $CUSTOMS_RECEIVER  --owner $OWNER)
SIGNAURE=$(echo "$SIGNAURE" | awk '/Signature:/ {line=$2} END {print line}')
echo "burn signature: $SIGNAURE"

sleep 10

# secord,generate ticket
dfx canister call solana_route generate_ticket "(record {
        signature=\"$SIGNAURE\";
        action = variant { Redeem };
        token_id = \"${TOKEN_ID}\";
        target_chain_id =  \"${BITCOIN_CHAIN_ID}\";
        sender =  \"${SOL_RECEIVER}\";
        receiver =  \"${CUSTOMS_RECEIVER}\";
        amount = $BURN_AMOUNT:nat64;
        memo = null;
        })"

dfx canister call omnity_hub query_tickets "(opt \"${BITCOIN_CHAIN_ID}\",0:nat64,5:nat64)"

# update token
# TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•NICE"
# TOKEN_NAME="HOPE•YOU•GET•DDDDDD"
# TOKEN_SYMBOL="DDDDDD"
# DECIMALS=5
# ICON="https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/image.png"
# dfx canister call omnity_hub validate_proposal "( vec {variant { UpdateToken = record { 
#         token_id = \"${TOKEN_ID}\"; 
#         name = \"${TOKEN_NAME}\";
#         issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
#         symbol = \"${TOKEN_SYMBOL}\"; 
#         decimals = ${DECIMALS};
#         icon = opt \"${ICON}\"; 
#         metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
#         dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})"
# dfx canister call omnity_hub execute_proposal "( vec {variant { UpdateToken = record { 
#         token_id = \"${TOKEN_ID}\"; 
#         name = \"${TOKEN_NAME}\";
#         issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
#         symbol = \"${TOKEN_SYMBOL}\"; 
#         decimals = ${DECIMALS};
#         icon = opt \"${ICON}\"; 
#         metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
#         dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})"
# dfx canister call omnity_hub query_directives "(opt \"${SOL_CHAIN_ID}\",opt variant {UpdateToken},0:nat64,5:nat64)"

# sleep 30

# dfx canister call solana_route get_token_list '()' 


sleep 300

# cannel schedule
dfx canister call solana_route cancel_schedule '()' 

# manual operation 

# TOKEN_NAME="HOPE•YOU•GET•RICH"
# TOKEN_SYMBOL="RICH"
# DECIMALS=2
# ICON="https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json"

# dfx canister call solana_route create_mint "(record {
#          token_id=\"${TOKEN_ID}\";
#         name=\"${TOKEN_NAME}\";
#         symbol=\"${TOKEN_SYMBOL}\";
#         decimals=${DECIMALS}:nat8;
#         uri=\"${ICON}\";
# })"

#dfx canister call solana_route get_or_create_aossicated_account '("3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia","Gi6BuNTXVgjhXzCjHg5m8Xz2jDQAyDDBi9F77p23ekYi")' 

# TX_ID=28b47548-55dc-4e89-b41d-76bc0247828f
# dfx canister call solana_route mint_to "(\"${TX_ID}\",
#         \"${ATA}\",
#         888888:nat64,
#         \"${TOKEN_MINT}\")"

# upgrade canister
# echo "upgrade solana route ..."
#dfx canister install --mode upgrade --argument '(variant { Upgrade = null })'  --upgrade-unchanged --yes solana_route



