#!/bin/bash

BITCOIN_CHAIN_ID="Bitcoin"
BITCOIN_CID="be2us-64aaa-aaaaa-qaabq-cai"

SOL_CHAIN_ID="eSolana"
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
# TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH202409242036"
# TOKEN_NAME="HOPE•YOU•GET•RICH202409242036"
# TOKEN_SYMBOL="RICH202409242036"
# DECIMALS=2
# TOKEN_URI="https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json"

# TOKEN_ID="Bitcoin-runes-202410211549"
# export TOKEN_ID
# TOKEN_NAME="HOPE•YOU•GET•NICE"
# TOKEN_SYMBOL="NICE"
# DECIMALS=0
# TOKEN_URI="https://arweave.net/MIvxbV_yLcsDwH-ks3BLNhz2xU8MZm2DvKPystDuA0g"
# TOKEN_URI="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/x.png"
# TOKEN_URI="https://arweave.net/DLXvyVzx01VKiNkLqTeSRTI4d7Mn_77U_DZjXQCRVhE"
PROTO="Bitcoin-runes"
TOKEN_NAME="RUNES•X•BITCOIN"
TIMESTAMP=$(date +"%Y%m%d%H%M")
TOKEN_ID="${PROTO}-${TOKEN_NAME}${TIMESTAMP}"
export TOKEN_ID
# TOKEN_SYMBOL=$(echo "$TOKEN_NAME" | grep -oE 'NICE[0-9]+')
TOKEN_SYMBOL="X"
DECIMALS=0
TOKEN_URI="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/x_meta.json"
# TOKEN_URI=https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/x.png
# TOKEN_URI="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/x_uri.json"
# https://xpwdk-zyaaa-aaaar-qajaa-cai.raw.icp0.io/token_meta?id=Bitcoin-runes-RUNES%E2%80%A2X%E2%80%A2BITCOIN202410220902
dfx canister call omnity_hub validate_proposal "( vec {variant { AddToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${TOKEN_URI}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})"
dfx canister call omnity_hub execute_proposal "( vec {variant { AddToken = record { 
        token_id = \"${TOKEN_ID}\"; 
        name = \"${TOKEN_NAME}\";
        issue_chain = \"${BITCOIN_CHAIN_ID}\"; 
        symbol = \"${TOKEN_SYMBOL}\"; 
        decimals = ${DECIMALS};
        icon = opt \"${TOKEN_URI}\"; 
        metadata =  vec{ record {\"rune_id\"; \"107:1\"}}; 
        dst_chains = vec {\"${BITCOIN_CHAIN_ID}\";\"${SOL_CHAIN_ID}\";}}}})"
dfx canister call omnity_hub query_directives "(opt \"${SOL_CHAIN_ID}\",opt variant {AddToken},0:nat64,5:nat64)"

# update fee
dfx canister call omnity_hub update_fee "vec {variant { UpdateTargetChainFactor = 
        record { target_chain_id=\"${BITCOIN_CHAIN_ID}\"; 
                 target_chain_factor=5000 : nat}}; 
                 variant { UpdateFeeTokenFactor = record { fee_token=\"${SOL_FEE}\"; 
                                                 fee_token_factor=2876 : nat}}}"

dfx canister call omnity_hub query_directives "(opt \"${SOL_CHAIN_ID}\",null,0:nat64,12:nat64)"

# req airdrop
solana airdrop 2
MASTER_KEY=$(solana address)
echo "current solana cli default address: $MASTER_KEY and balance: $(solana balance $MASTER_KEY)"
# get signer and init it
# KEYTYPE="variant { Native }"
KEYTYPE="variant { ChainKey }"
dfx canister call solana_route update_key_type "($KEYTYPE)" 
dfx canister call solana_route query_key_type "($KEYTYPE)" 
SIGNER=$(dfx canister call solana_route signer "($KEYTYPE)" --candid ./assets/solana_route.did)
SIGNER=$(echo "$SIGNER" | awk -F'"' '{print $2}')
echo "current SIGNER: $SIGNER"
# transfer SOL to init signer
AMOUNT=0.2
echo "transfer SOL to $SIGNER from $MASTER_KEY"
solana transfer $SIGNER $AMOUNT --with-memo init_account --allow-unfunded-recipient
echo "$SIGNER balance: $(solana balance $SIGNER)"

echo "Init done!"


# start schedule 
echo start_schedule  
dfx canister call solana_route start_schedule '(null)' 

# wait for query directives or tickets from hub to solana route
sleep 90

echo "check sync directive from hub "
dfx canister call solana_route get_chain_list '()' 
dfx canister call solana_route get_token_list '()' 
dfx canister call solana_route get_redeem_fee '("Bitcoin")' 
echo

# A-B tansfer/redeem
echo "mock: transfer from Bitcoin to Solana ..."
echo 
TID="28b47548-55dc-4e89-b41d-76bc0247828f"
AMOUNT=22222222
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

sleep 60

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


echo "mock: redeem from solana to customs... "
# first collect fee
# get fee account
FEE_ACCOUNT=$(dfx canister call solana_route get_fee_account '()')
FEE_ACCOUNT=$(echo "$FEE_ACCOUNT" | awk -F'"' '{print $2}')
echo "fee account: $FEE_ACCOUNT"
# get fee amount
FEE_AMOUNT=$(dfx canister call solana_route get_redeem_fee "(\"${BITCOIN_CHAIN_ID}\")")
FEE_AMOUNT=$(echo "$FEE_AMOUNT" | grep -oE '[0-9_]+ ' | sed 's/_//g' | awk '{printf "%.9f\n", $1 / 1000000000}')
FEE_AMOUNT=$(echo "$FEE_AMOUNT * 10^9" | bc | awk '{printf "%.0f", $0}')
echo "fee amount: $FEE_AMOUNT lamports"
# collect fee
# WALLET_ADDRESS=$(solana address)
# echo "collect fee to $FEE_ACCOUNT from $WALLET_ADDRESS"
# solana transfer $FEE_ACCOUNT $FEE_AMOUNT 

# second, burn token
CUSTOMS_RECEIVER="D58qMHmDAoEaviG8s9VmGwRhcw2z1apJHt6RnPtgxdVj"
# WALLET_ADDRESS=~/.config/solana/boern.json
BURN_AMOUNT=11111111
# echo spl-token burn $ATA $BURN_AMOUNT  --with-memo $CUSTOMS_RECEIVER  --owner $WALLET_ADDRESS
# # echo $(spl-token burn $ATA $BURN_AMOUNT  --with-memo $CUSTOMS_RECEIVER  --owner $OWNER)
# SIGNAURE=$(spl-token burn $ATA $BURN_AMOUNT  --with-memo $CUSTOMS_RECEIVER  --owner $WALLET_ADDRESS)
# SIGNAURE=$(echo "$SIGNAURE" | awk '/Signature:/ {line=$2} END {print line}')
# echo "burn signature: $SIGNAURE"

SOLANA_RPC_URL="https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ"
KEYPAIR=$(bat -p ~/.config/solana/boern.json)
echo "redeem tx vars:"
echo "rpc_url: $SOLANA_RPC_URL"
echo "keypair: $KEYPAIR"
echo "transfer from_account: $SOL_RECEIVER"
echo "fee_account: $FEE_ACCOUNT"
echo "fee_amount $FEE_AMOUNT"
echo "token_mint: $TOKEN_MINT"
echo "burn_account: $ATA"
echo "owner_account: $SOL_RECEIVER"
echo "burn_amount: $BURN_AMOUNT"
echo "memo_msg: $CUSTOMS_RECEIVER"

# python ./scripts/redeem_tx.py \
#   --rpc_url $SOLANA_RPC_URL \
#   --keypair $KEYPAIR \
#   --from_account $SOL_RECEIVER \
#   --fee_account $FEE_ACCOUNT \
#   --fee_amount $FEE_AMOUNT \
#   --token_mint $TOKEN_MINT \
#   --burn_account $ATA \
#   --owner_account $SOL_RECEIVER \
#   --burn_amount $BURN_AMOUNT \
#   --memo_msg $CUSTOMS_RECEIVER | \
#   echo "Transaction Output: $(cat)"

sleep 30
# check minto token result
# TOKEN_BALANCE=$(spl-token balance $TOKEN_MINT --owner $SOL_RECEIVER | tr -d ' \n')
# echo "token balance: $TOKEN_BALANCE"

# if [ "$AMOUNT" -eq "$TOKEN_BALANCE" ]; then
#   echo "AMOUNT and TOKEN_BALANCE are equal."
# else
#   echo "AMOUNT and TOKEN_BALANCE are not equal."
# fi

TOKEN_BALANCE=$(spl-token balance "$TOKEN_MINT" --owner "$SOL_RECEIVER" | tr -d ' \n')
while [ -z "$TOKEN_BALANCE" ]; do
  echo "Waiting for TOKEN_BALANCE to have a value..."
  sleep 5  
  TOKEN_BALANCE=$(spl-token balance "$TOKEN_MINT" --owner "$SOL_RECEIVER" | tr -d ' \n')
done
echo "TOKEN_BALANCE is : $TOKEN_BALANCE"

# execute redeem tx
# python ./scripts/redeem_tx.py \
#   --rpc_url $SOLANA_RPC_URL \
#   --keypair $KEYPAIR \
#   --from_account $SOL_RECEIVER \
#   --fee_account $FEE_ACCOUNT \
#   --fee_amount $FEE_AMOUNT \
#   --token_mint $TOKEN_MINT \
#   --burn_account $ATA \
#   --owner_account $SOL_RECEIVER \
#   --burn_amount $BURN_AMOUNT \
#   --memo_msg $CUSTOMS_RECEIVER 

# SIGNAURE=$(python ./scripts/redeem_tx.py \
#   --rpc_url $SOLANA_RPC_URL \
#   --keypair $KEYPAIR \
#   --from_account $SOL_RECEIVER \
#   --fee_account $FEE_ACCOUNT \
#   --fee_amount $FEE_AMOUNT \
#   --token_mint $TOKEN_MINT \
#   --burn_account $ATA \
#   --owner_account $SOL_RECEIVER \
#   --burn_amount $BURN_AMOUNT \
#   --memo_msg $CUSTOMS_RECEIVER )

SIGNATURE=$(python ./scripts/redeem_tx.py \
  --rpc_url $SOLANA_RPC_URL \
  --keypair $KEYPAIR \
  --from_account $SOL_RECEIVER \
  --fee_account $FEE_ACCOUNT \
  --fee_amount $FEE_AMOUNT \
  --token_mint $TOKEN_MINT \
  --burn_account $ATA \
  --owner_account $SOL_RECEIVER \
  --burn_amount $BURN_AMOUNT \
  --memo_msg $CUSTOMS_RECEIVER  \
| tail -n 1 )

echo "redeem tx signature: $SIGNATURE"

sleep 15

# finally,generate ticket and send to hub
dfx canister call solana_route generate_ticket "(record {
        signature=\"$SIGNATURE\";
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
# TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH66"
# TOKEN_NAME="HOPE•YOU•GET•URICH66"
# TOKEN_SYMBOL="URICH66"
# DECIMALS=2
# ICON="https://github.com/ordinals/ord/assets/14307069/f1307be5-84fb-4b58-81d0-6521196a2406"
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

# sleep 50
# dfx canister call solana_route get_token_list '()' 


sleep 120

# cannel schedule
dfx canister call solana_route stop_schedule '(null)' 

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


# update muti rpc config
# rpc1=https://solana-devnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ
# rpc2=https://rpc.ankr.com/solana_devnet/670ae11cd641591e7ca8b21e7b7ff75954269e96f9d9f14735380127be1012b3
# rpc3=https://nd-471-475-490.p2pify.com/6de0b91c609fb3bd459e043801aa6aa4
# rpc4=https://solana-mainnet.g.alchemy.com/v2/ClRAj3-CPTvcl7CljBv-fdtwhVK-XWYQ

# dfx canister call solana_route update_multi_rpc "(record {
#     rpc_list = vec {\"${rpc1}\";\"${rpc2}\";\"${rpc3}\"};\
#     minimum_response_count = 2;
# })"

# dfx canister call solana_route multi_rpc_config '()'

# valid_tx_from_multi_rpc
# sig=4e1gA4YvTt95DYY5kdwSWpGr2oiMqRX2nk4XenF1aiJSz69cbLBMeTfV6HG4jG7jHtdcHwwjGCSw5zepgpC8n5g7
# dfx canister call solana_route valid_tx_from_multi_rpc "(\"${sig}\")"

# update provider rpc 
# dfx canister call ic-solana-provider update_rpc "(\"${rpc1}\")"

# query tx detail
# dfx canister call solana_route get_transaction "(\"${sig}\",null)"
# dfx canister call solana_route debug '(true)' --ic
# target_sig=2u8tyX1Eyod9potrhqTWqin1fiTZGMrbFU6sHBu3hkys8h8eZM2Hrh83kWnSjL2qydf6ZjGdSoYH6s3PjqJsozqn
# test_account=5nevxTLuX5TZsJg4Wf9ws6B1hecyFaHmjc5PNEJVrGiL
# limit=6
# dfx canister call solana_route search_signature_from_address "(\"$target_sig\",\"$test_account\",opt $limit,null)"
# dfx canister call solana_route start_schedule '(opt vec {
#         variant { GetDirectives };
#         variant { GetTickets }
#         })' 
# dfx canister call solana_route stop_schedule '(opt vec {
#         variant { GetDirectives };
#         variant { GetTickets }
#         })' 

# dfx canister call solana_route start_schedule '(null)' 
# dfx canister call solana_route stop_schedule '(opt vec {
#         variant { CreateMint };
#         variant { UpdateToken };
#         variant { CreateATA };
#         variant { MintToken };
#         })' 
# dfx canister call solana_route stop_schedule '(null)' 