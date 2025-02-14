#!/bin/bash

export DFX_WARNING="-mainnet_plaintext_identity"
# config network
NETWORK=local

# check route config 
dfx canister call sui_route get_route_config '()' --network $NETWORK

# NETWORK=http://localhost:12345/
# NETWORK=ic
# get sui_route_address and init it
# KEYTYPE="variant { Native }"
KEYTYPE="variant { ChainKey }"
# dfx canister call sui_route sui_route_address "($KEYTYPE)" --network $NETWORK 
sui_route_address=$(dfx canister call sui_route sui_route_address "($KEYTYPE)" --network $NETWORK)
sui_route_address=$(echo "$sui_route_address" | awk -F'"' '{print $2}' | tr -d '[:space:]')
echo "sui_route_address: $sui_route_address"
# requrie faucet
sui client faucet --address $sui_route_address

# transfer SUI to init signer
# 1  MIST = 0.000_000_001 SUI.
# 1 SUI =1_000_000_000 MIST
# AMOUNT=500000000
coin_amount=800000000
sui client ptb \
  --assign recipient @$sui_route_address \
  --assign coin_amount $coin_amount \
  --split-coins gas [coin_amount] \
  --assign coins \
  --transfer-objects [coins.0] recipient \
  --gas-budget 50000000 \
  --dry-run

gas_obj_id=0x98f3fddb83a23866c7d2c3ffed636e77a18bdff8dea50a719efa3233a28c8a96
active_address=$(sui client active-address)
echo "transfer SUI to $sui_route_address from $active_address"
sui client transfer-sui --to $sui_route_address --sui-coin-object-id $gas_obj_id --gas-budget 5000000
# check balance
echo sui route address balance: 
sui client gas $sui_route_address

# test: query info from sui chain via sui route
dfx canister call sui_route get_gas_price '()' --network $NETWORK

# owner=${sui_route_address}
# owner=0x365eb9f54539cf07332773f756a392d5af507b3b8990f84e52ee6f6b6b57534b
coin_type="0x2::sui::SUI"
dfx canister call sui_route get_balance "(\"${sui_route_address}\",opt \"${coin_type}\")" --network $NETWORK

address=0xaf9306cac62396be300b175046140c392eed876bd8ac0efac6301cea286fa272
struct_type="0x2::coin::Coin<0x2::sui::SUI>"
dfx canister call sui_route get_owner_objects "(\"${address}\",opt \"${struct_type}\")" --network $NETWORK

obj_id="0x62f219823a358961015fbe6e712b571aca62442092e4ab6a0b409bbb20697fb8"
dfx canister call sui_route get_object "(\"${obj_id}\")" --network $NETWORK

coin_type="0x2::sui::SUI"
dfx canister call sui_route get_coins "(\"${sui_route_address}\",opt \"${coin_type}\")" --network $NETWORK

# get_transaction_block
digest=8Qffae1qP1ssr8LiX3pZ9TUVCrkhuMTStappV5JPcJYY
dfx canister call sui_route get_transaction_block "(\"${digest}\")" --network $NETWORK

# get events
digest=8Qffae1qP1ssr8LiX3pZ9TUVCrkhuMTStappV5JPcJYY
dfx canister call sui_route get_events "(\"${digest}\")" --network $NETWORK


# tansfer sui from sui route to recipient
echo sui route address balance: 
sui client gas $sui_route_address
recipient=0xaf9306cac62396be300b175046140c392eed876bd8ac0efac6301cea286fa272
# sui client objects $recipient
amount=50000000
digest=$(dfx canister call sui_route transfer_sui "(\"$recipient\",$amount)" --network $NETWORK)
digest=$(echo "$digest" | awk -F'"' '{print $2}')
echo "$digest"
dfx canister call sui_route get_transaction_block "(\"${digest}\")" --network $NETWORK

# transfer object to recipent
recipient=0x021e364dfa89ce87cbfbbae322ebd730c0737ff10a41d4a3b295f1b386031c51
obj_id=0xb55f302a44034dd7b6e1bcac542a434f234f67daea34773f66af31af10044656
dfx canister call sui_route transfer_object "(\"$recipient\",\"$obj_id\")" --network $NETWORK

################################################################
# Note: publish sui port contracts,includes action and tokens
# transfer coin treasury cap/coin metadata/ port_owner_cap/ from publisher to sui route address 
# ref to sui port README
################################################################

sui client objects $sui_route_address

# update sui port action info
port_owner_cap=0x4a990b885d5834e72442fa49ab13d004c3d518904caeef0cc88f7ebd0398ae10
ticket_table=0xff08353287be3005ca5b31f288c8592f0b613d9f37156f7255eda8e395f54286
action_package=0x26c5ce2c1ed70b877723bbebc13c9c10984bc113b4095492cc081a41c78dddf4
action_module=action
action_upgrade=0x14e7da34ae3f68e6f087c96ea9dc4e22dad49ffff2a4c8f3b38ebf032a9152c5
# update sui port action info to sui route
dfx canister call sui_route update_sui_port_action "(
    record {
       package = \"$action_package\";
       module = \"$action_module\";
       functions = vec { \"mint\";
                         \"mint_with_ticket\";
                         \"collect_fee\";
                         \"burn_coin\";
                         \"redeem\";
                         \"create_ticket_table\";
                         \"minted_ticket\";
                         \"remove_ticket\";
                         \"drop_ticket_table\";
                         };
      port_owner_cap = \"$port_owner_cap\";
      ticket_table = \"$ticket_table\";
      upgrade_cap = \"$action_upgrade\";
    }
)" --network $NETWORK

dfx canister call sui_route sui_port_action '()' --network $NETWORK



# update sui port token info for ICP
token_id=Bitcoin-runes-DOG•GO•TO•THE•MOON
coin_package=0xe27ec5044f815be78ba062515d3139cd1181028ca3013fa19bab7567539cca21
coin_module=dog
coin_treasury_cap=0xc82532fc14d6db37b7953208a3581675fcf67ee462cd3fd5cbea23fef23929b7
coin_metadata=0x525d6dd219e75f5d7f780c6492ce89c8153500a7cffdeb4e33c58b0beccaec75
type_tag=0xe27ec5044f815be78ba062515d3139cd1181028ca3013fa19bab7567539cca21::dog::DOG
coin_upgrade=0x9af72cd1faa3a287df423b8d1726e3f82b2c694ed1f75130d73ce2d64816b95e

dfx canister call sui_route update_sui_token "(
    \"$token_id\",
    record {
       package = \"$coin_package\";
       module = \"$coin_module\";
       treasury_cap = \"$coin_treasury_cap\";
       metadata = \"$coin_metadata\";
       type_tag = \"$type_tag\";
       functions = vec {};
       upgrade_cap = \"$coin_upgrade\";
    }
)" --network $NETWORK

dfx canister call sui_route sui_token "(\"$token_id\")" --network $NETWORK


# update sui port token info for RICH.OT
token_id="Bitcoin-runes-HOPE•YOU•GET•RICH"
coin_package=0xa071e8021b690d58dbc1112eaba6f9361ee9deda527775aefaf896686996fd8c
coin_module=rich
coin_treasury_cap="0x6c36691b6b50759073d49586b9ab8abe131af4d83e525ca868a7b1355957e389"
coin_metadata="0x92486543cbf10231ac47257ec392c7d6d5fb7c866e1ddeb7e199f70f95034d9f"
type_tag=0xa071e8021b690d58dbc1112eaba6f9361ee9deda527775aefaf896686996fd8c::rich::RICH
token_upgrade=0xf410a4a2e55c8ce83c0c3ea0582c9ab2e869f3b75661b81e3e34d12ff4222c5f

dfx canister call sui_route update_sui_token "(
    \"$token_id\",
    record {
       package = \"$coin_package\";
       module = \"$coin_module\";
       treasury_cap = \"$coin_treasury_cap\";
       metadata = \"$coin_metadata\";
       type_tag = \"$type_tag\";
       functions = vec {};
       upgrade_cap = \"$token_upgrade\";
    }
)" --network $NETWORK

dfx canister call sui_route sui_token "(\"$token_id\")" --network $NETWORK


# mint token to recipient
token_id="Bitcoin-runes-HOPE•YOU•GET•RICH"
timestamp=$(date +"%Y%m%d%H%M")
ticket_id=${token_id}-$timestamp
echo ticket_id: $ticket_id
# recipient=0xaf9306cac62396be300b175046140c392eed876bd8ac0efac6301cea286fa272
# recipient=$(sui client active-address)
recipient=$sui_route_address
echo recipient: $recipient
amount=10000
echo mint amount: $amount

dfx canister call sui_route mint_to_with_ticket "(
    \"$ticket_id\",
    \"$token_id\",
    \"$recipient\",
    $amount:nat64
)" --network $NETWORK 

digest="4JCVazuKaeeGhVKjCfVrPf2b23RXsEV35nvu5cSTZ53F"
dfx canister call sui_route get_events "(\"${digest}\")" --network $NETWORK

# burn token via sui route
# first split and transfer the burned coin to sui route 
# obj_id=0xb2c28ea3fcedf0949530c6ab5b525ec72a8f997dc8ffa0a17fac46278de26478
# sui client object $obj_id
# to=$sui_route_address
# sui client transfer --to $to --object-id $obj_id
sui client objects $sui_route_address
obj_id=0xc844a514ad21e4c12ac22b914650c38b6238040d643f089d435e9a6330faf28f
sui client object $obj_id
# execute burn token
dfx canister call sui_route burn_token "(
    \"$token_id\",
    \"$obj_id\",
)" --network $NETWORK 


# update fee
TARGET_CHAIN_ID=sICP
TARGET_CHAIN_FACTOR=2000
# SUI_CHAIN_ID="eSui"
FEE_TOKEN_FACTOR=10000
SUI_FEE="SUI"

dfx canister call sui_route update_redeem_fee "(variant { UpdateTargetChainFactor =
        record { target_chain_id=\"${TARGET_CHAIN_ID}\"; 
                 target_chain_factor=$TARGET_CHAIN_FACTOR : nat}})" --network $NETWORK
dfx canister call sui_route update_redeem_fee "(variant { UpdateFeeTokenFactor = 
        record { fee_token=\"${SUI_FEE}\"; 
                fee_token_factor=$FEE_TOKEN_FACTOR : nat}})" --network $NETWORK

dfx canister call sui_route get_redeem_fee "(\"${TARGET_CHAIN_ID}\")" --network $NETWORK

fee_account=$sui_route_address
fee_amount=50000000
echo "fee account: $fee_account"
echo "fee amount: $fee_amount"

# call collet_fee
func=collect_fee
sui client ptb \
  --assign fee_amount $fee_amount \
  --assign recipient @$fee_account \
  --split-coins gas [fee_amount] \
  --assign fee_coins \
  --move-call $package::$module::$func fee_coins.0 recipient \
  --gas-budget 100000000 \
  --dry-run \
  --preview

# call redeem
target_chain_id=Bitcoin
target_chain_id=$(printf '%s' "$target_chain_id" | od -An -v -tuC -w1 | awk '{$1=$1;print}' | tr '\n' ',' | sed 's/,$//')
target_chain_id="[${target_chain_id}]"
echo "target_chain_id bytes: $target_chain_id"
token_id="Bitcoin-runes-APPLE•PIE"
token_id=$(printf '%s' "$token_id" | od -An -v -tuC -w1 | awk '{$1=$1;print}' | tr '\n' ',' | sed 's/,$//')
token_id="[${token_id}]"
echo "token_id bytes: $token_id"
burn_token_obj=0x145756516a5795b00bdebd531f81b42823ea89b0a281bea0b3544ff7b5159f4d
echo "burn token object id: $burn_token_obj"
receiver=bc1qmh0chcr9f73a3ynt90k0w8qsqlydr4a6espnj6
receiver=$(printf '%s' "$receiver" | od -An -v -tuC -w1 | awk '{$1=$1;print}' | tr '\n' ',' | sed 's/,$//')
receiver="[${receiver}]"
echo "recevier bytes: $receiver"
memo="This ticket is redeemed from Sui to Bitcoin"
memo=$(printf '%s' "$memo" | od -An -v -tuC -w1 | awk '{$1=$1;print}' | tr '\n' ',' | sed 's/,$//')
memo="[${memo}]"
echo "memo bytes: $memo"
route_address=$sui_route_address
echo "sui route address:$route_address"
redeem_amount=50000000
echo "redeem amount: $redeem_amount"


# call redeem via sui client ptb cli
sui client ptb \
  --assign fee_amount $fee_amount \
  --assign recipient @$fee_account \
  --split-coins gas [fee_amount] \
  --assign fee_coins \
  --move-call $package::$module::collect_fee fee_coins.0 recipient \
  --make-move-vec "<u8>" $target_chain_id \
  --assign target_chain_id_bytes \
  --move-call std::string::utf8 target_chain_id_bytes \
  --assign target_chain_id \
  --make-move-vec "<u8>" $token_id \
  --assign token_id_bytes \
  --move-call std::string::utf8 token_id_bytes \
  --assign token_id \
  --make-move-vec "<u8>" $receiver \
  --assign receiver_bytes \
  --move-call std::string::utf8 receiver_bytes \
  --assign receiver \
  --make-move-vec "<u8>" $memo \
  --assign memo_bytes \
  --move-call std::string::utf8 memo_bytes \
  --assign memo_str \
  --move-call std::option::some "<std::string::String>" memo_str \
  --assign memo \
  --split-coins @$burn_token_obj [$redeem_amount] \
  --assign burn_token \
  --assign route_address @$route_address \
  --move-call $package::$module::redeem target_chain_id token_id burn_token receiver memo route_address \
  --gas-budget 100000000 \
  --dry-run
  

dfx canister call sui_route get_chain_list '()' --network $NETWORK
dfx canister call sui_route get_token_list '()' --network $NETWORK
# get events
digest=7NozueMkxV5VvLTDapBi8uynvoy6GwG9MUKvKMR7HRqj
dfx canister call sui_route get_events "(\"${digest}\")" --network $NETWORK
dfx canister call sui_route valid_tx_from_multi_rpc "(\"${digest}\")" --network $NETWORK

# update coin meta
token_id="Bitcoin-runes-APPLE•PIE"
# update symbole
symbol=PIE
dfx canister call sui_route update_token_meta "(
    \"$token_id\",
    variant {Symbol=\"$symbol\"})"
# update name
name=APPLE•PIE
dfx canister call sui_route update_token_meta "(
    \"$token_id\",
    variant {Name=\"$name\"})"
# update icon
icon=https://arweave.net/tTTr14osgHDC2jBcvIM5FHi1H8kuUmQh4Tlknr5pG7U
dfx canister call sui_route update_token_meta "(
    \"$token_id\",
    variant {Icon=\"$icon\"})"

# update icon
desc="The Apple Pie is a protocol based on bitcoin runes"
dfx canister call sui_route update_token_meta "(
    \"$token_id\",
    variant {Description=\"$desc\"})"

# upgrade sui port
upgrade_cap_id=0x8fea3b52c72aa54461fc877bbd68a38923403f6c65ad62fe4ec713bb3aaf1c8b
sui client upgrade \
  --upgrade-capability $upgrade_cap_id \
  --gas-budget 100000000 \
  --dry-run 


# update sui token info with upgrade info
package="new package id"
mint_record=0x05bbb8c4fa16c63578c733bc64f616f34e3c2f05ae10f058fa83f67bea02d621
dfx canister call sui_route update_sui_token "(
    \"$token_id\",
    record {
       package = \"$package\";
       module = \"$module\";
       treasury_cap = \"$treasury_cap\";
       metadata = \"$metadata\";
       type_tag = \"$type_tag\";
       functions = vec { \"mint_to\";
                         \"collect_fee\";
                         \"redeem\";
                         \"create_mint_record\";
                         \"clear_mint_record\";
                         \"minted_ticket\"};
       mint_record_obj = \"$mint_record\";
       port_owner_cap = \"$port_owner_cap\";
    }
)" --network $NETWORK


# split coins
split_amount=555555
sui client ptb \
  --move-call sui::tx_context::sender \
  --assign sender \
  --assign split_amount $split_amount \
  --split-coins gas [split_amount] \
  --assign coins \
  --transfer-objects [coins.0] sender \
  --gas-budget 50000000 \
  --dry-run

# merge coins
# if a address only has two cions, can`t merge the last two coins
base_coin=0x98f3fddb83a23866c7d2c3ffed636e77a18bdff8dea50a719efa3233a28c8a96
coin_1=0x66c1e9987bf136ebc3ec70e6d512b19411d5ea0c1bf5393b16791ff83d06c0d9
coin_2=0xd75774d03c2ea25e7d4c04b841d1f9692878d54028ab3b4e7635acb63244d48a
sui client ptb \
  --assign base_coin @$base_coin \
  --assign coin_1 @$coin_1 \
  --assign coin_2 @$coin_2 \
  --merge-coins base_coin [coin_1,coin_2] \
  --gas-coin @$coin_1 \
  --gas-budget 5000000 \
  --dry-run

base_coin=0x87f4445aa9029000e4a700bbaa51a576c6f51c9087a1222c8d323d567b5a89d1
merged_coin=0x66c1e9987bf136ebc3ec70e6d512b19411d5ea0c1bf5393b16791ff83d06c0d9
fee_coin=0x98f3fddb83a23866c7d2c3ffed636e77a18bdff8dea50a719efa3233a28c8a96
sui client ptb \
  --assign base_coin @$base_coin \
  --assign merged_coin @$merged_coin \
  --merge-coins base_coin [merged_coin] \
  --gas-budget 5000000 \
  --gas-coin @$fee_coin \
  --dry-run


obj_id="0x800782cd065c567a29d0b1bbb5c47f0589ad04256516dc365612ee0f704c4a4e"
dfx canister call sui_route check_object_exists "(\"${sui_route_address}\",\"${obj_id}\")" --network $NETWORK


dfx canister call sui_route get_gas_budget '()' --network $NETWORK
gas_budget=10000000
dfx canister call sui_route update_gas_budget "(${gas_budget})" --network $NETWORK


chain_id=Bitcoin
dfx canister call sui_route get_redeem_fee "(\"${chain_id}\")" --network $NETWORK

# recipient=$(sui client active-address)
recipient=0x021e364dfa89ce87cbfbbae322ebd730c0737ff10a41d4a3b295f1b386031c51
echo recipient: $recipient
ticket_table=0xd83d2eaea0516749038aae2579ef5dfb98f58a98924f8f88035a8a9d264e4b8d
port_owner_cap=0x62f219823a358961015fbe6e712b571aca62442092e4ab6a0b409bbb20697fb8
treasure_cap=0x26215cfe5b19502eb01c934ef9805d5c9cd0117f156d467413cd17c637c42737
metadata=0x53463426bb2c1b2202a82db19b99d64b42177db9eb6e7bc15f6389284b8616a9

# echo obj_id: $obj_id
dfx canister call sui_route transfer_objects "(\"${recipient}\",
    vec {\"${ticket_table}\";\"${port_owner_cap}\";\"${treasure_cap}\";\"${metadata}\"})" --network $NETWORK

base_coin=0xce75a61cb01535e7c6078c719c6feb60b5702d51671aaf5fa1f551e2101048e3
echo base_coin: $base_coin
coin_1=0xa2fd733151227f423f90d44219768a1e12a03bbadec2bc9d19b69072f95cb060
coin_2=0xf17ff49c117ae5ad6870a29641aa3d4369dcdf704c4931a580365b58759afa2d
dfx canister call sui_route merge_coin "(\"${base_coin}\",
    vec {\"${coin_1}\";\"${coin_2}\"})" --network $NETWORK


base_coin=0x98f3fddb83a23866c7d2c3ffed636e77a18bdff8dea50a719efa3233a28c8a96
echo base_coin: $base_coin
coin_1=0x3072cd99319a26c9e0bac00813b0681ff1fe795b3e2e7b9a00b9334c5af89533
dfx canister call sui_route merge_coin "(\"${base_coin}\",
    vec {\"${coin_1}\"})" --network $NETWORK

token_id="Bitcoin-runes-APPLE•PIE"
echo token_id: $token_id
dfx canister call sui_route create_ticket_table "(
    \"${sui_route_address}\")" --network $NETWORK

token_id="Bitcoin-runes-APPLE•PIE"
mint_record_id=0xb79fd7f37c6184b8d280694194140d78037f83689a855a9629082832ac0aaa30
echo token_id: $token_id
dfx canister call sui_route drop_ticket_table "(
    \"${mint_record_id}\")" --network $NETWORK

coin_id=0xb5375ddb657cb7c629545e6ed9e695d9356cff92fa88014223c27a748845cbc8
echo coin_id: $coin_id
amount=88888888
dfx canister call sui_route split_coin "(
    \"${coin_id}\",$amount,\"${sui_route_address}\")" --network $NETWORK

# split gas coin, SUI
coin_id=0xce75a61cb01535e7c6078c719c6feb60b5702d51671aaf5fa1f551e2101048e3
echo coin_id: $coin_id
amount=22222222
dfx canister call sui_route split_coin "(
    \"${coin_id}\",$amount,\"${sui_route_address}\")" --network $NETWORK

coin_type="0x2::sui::SUI"
threshold=5000000
dfx canister call sui_route fetch_coin "(
    \"${sui_route_address}\",
    opt \"${coin_type}\",
    $threshold:nat64)" --network $NETWORK


TOKEN_ID="Bitcoin-runes-HOPE•YOU•GET•RICH"
TOKEN_NAME="HOPE•YOU•GET•RICH"
TOKEN_SYMBOL="RICH.OT"
DECIMALS=2
RUNE_ID="840000:846"
ICON="https://raw.githubusercontent.com/octopus-network/omnity-token-imgs/main/metadata/rich_ot_meta.json"

dfx canister call sui_route add_token "(record {
        token_id=\"${TOKEN_ID}\";
        name=\"${TOKEN_NAME}\";
        symbol=\"${TOKEN_SYMBOL}\";
        decimals=${DECIMALS}:nat8;
        icon=opt \"${ICON}\";
        metadata = vec{ record {\"rune_id\" ; \"840000:846\"}};
})" --network $NETWORK

dfx canister call sui_route get_token "(\"${TOKEN_ID}\")" --network $NETWORK

# check route config 
dfx canister call sui_route get_route_config '()' --network $NETWORK

dfx canister call sui_route multi_rpc_config '()' --network $NETWORK
rpc1="https://fullnode.testnet.sui.io:443"
rpc2="https://fullnode.testnet.sui.io:443"
rpc3="https://fullnode.testnet.sui.io:443"
dfx canister call sui_route update_multi_rpc "(record { 
    rpc_list = vec {\"${rpc1}\";
                     \"${rpc2}\";
                     \"${rpc3}\";};\
    minimum_response_count = 2:nat32;})" --network $NETWORK
dfx canister call sui_route multi_rpc_config '()' --network $NETWORK

dfx canister call sui_route start_schedule '(null)' --network $NETWORK
dfx canister call sui_route active_tasks '()' --network $NETWORK
dfx canister call sui_route stop_schedule '(null)' --network $NETWORK
dfx canister call sui_route seqs '()' --network $NETWORK

dfx canister call sui_route forward '()' --network $NETWORK
forward="https://fullnode.testnet.sui.io:443"
forward=https://sui.nownodes.io
dfx canister call sui_route update_forward "(\"${forward}\")" --network $NETWORK

http_url="https://solana-rpc-proxy-398338012986.us-central1.run.app"
ws_url="wss://solana-rpc-proxy-398338012986.us-central1.run.app"
dfx canister call sui_route rpc_provider '()' --network $NETWORK

dfx canister call sui_route update_rpc_provider "(variant {Custom=record {
    \"${http_url}\";\"${ws_url}\"}})" --network $NETWORK

dfx canister call sui_route rpc_provider '()' --network $NETWORK



# mint dog token to recipient
token_id=Bitcoin-runes-DOG•GO•TO•THE•MOON
timestamp=$(date +"%Y%m%d%H%M")
ticket_id=${token_id}-$timestamp
echo ticket_id: $ticket_id
# recipient=0xaf9306cac62396be300b175046140c392eed876bd8ac0efac6301cea286fa272
# recipient=$(sui client active-address)
recipient=$sui_route_address
echo recipient: $recipient
amount=800000
echo mint amount: $amount

dfx canister call sui_route mint_to_with_ticket "(
    \"$ticket_id\",
    \"$token_id\",
    \"$recipient\",
    $amount:nat64
)" --network $NETWORK 

digest="4JCVazuKaeeGhVKjCfVrPf2b23RXsEV35nvu5cSTZ53F"
dfx canister call sui_route get_events "(\"${digest}\")" --network $NETWORK
