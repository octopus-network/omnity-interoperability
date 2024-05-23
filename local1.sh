
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ cd omnity
$ dfx stop
$ dfx start --clean
$ cargo clean

# https://internetcomputer.org/docs/current/developer-docs/defi/icp-tokens/ledger-local-setup
dfx identity new minter
dfx identity use minter
export MINTER_ACCOUNT_ID=$(dfx ledger account-id)
dfx identity use default
export DEFAULT_ACCOUNT_ID=$(dfx ledger account-id)
dfx deploy --specified-id ryjl3-tyaaa-aaaaa-aaaba-cai icp_ledger_canister --argument "
  (variant {
    Init = record {
      minting_account = \"$MINTER_ACCOUNT_ID\";
      initial_values = vec {
        record {
          \"$DEFAULT_ACCOUNT_ID\";
          record {
            e8s = 10_000_000_000 : nat64;
          };
        };
      };
      send_whitelist = vec {};
      transfer_fee = opt record {
        e8s = 10_000 : nat64;
      };
      token_symbol = opt \"LICP\";
      token_name = opt \"Local ICP\";
    }
  })
"
dfx ledger balance $DEFAULT_ACCOUNT_ID
100.00000000 ICP

$ dfx identity --identity default get-principal
oqqew-3kok2-4ca2v-uwf4q-bykqb-yghly-kwet3-a5vqf-cu4ug-ztg4o-sqe
 dfx deploy omnity_hub --argument '(variant { Init = record { admin = principal "oqqew-3kok2-4ca2v-uwf4q-bykqb-yghly-kwet3-a5vqf-cu4ug-ztg4o-sqe"} })'
 dfx deploy bitcoin_customs --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 1; max_time_in_queue_nanos = 1_000_000_000; runes_oracle_principal = principal "oqqew-3kok2-4ca2v-uwf4q-bykqb-yghly-kwet3-a5vqf-cu4ug-ztg4o-sqe"; chain_id = "Bitcoin"; chain_state = variant { Active } } })'
# dfx deploy icp_route --argument '(variant { Init = record { hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; chain_id = "eICP"; chain_state = variant { Active } } })'
#deploy evm_rpc
dfx deploy evm_rpc --argument '(record { nodesInSubnet = 28 })'
#deploy cdk route
dfx deploy evm_route --argument '(record { fee_token_id = "BTC" network = variant { local }; omnity_port_contract = "0x765F2c1F334E6479Be5D5F8f2E12128612f47CE3"; scan_start_height = 200000; evm_rpc_canister_addr = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai";  evm_chain_id = 11155111; admin = principal "oqqew-3kok2-4ca2v-uwf4q-bykqb-yghly-kwet3-a5vqf-cu4ug-ztg4o-sqe"; hub_principal = principal "be2us-64aaa-aaaaa-qaabq-cai"; chain_id = "cdk_sepolia"; rpc_url = "https://rpc-sepolia.rockx.com";})'

# https://github.com/lesterli/ord/blob/docs/runes/docs/src/guides/runes.md
$ git clone https://github.com/octopus-network/ord.git
$ git checkout dev
$ sudo docker run --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=mysecretpassword -v ~/dev/data:/var/lib/postgresql/data -d postgres:12
$ sudo docker run -it --rm --network host postgres:12 psql -h 127.0.0.1 -U postgres
postgres=# CREATE DATABASE runescan ENCODING = 'UTF8';
$ sudo docker exec -i postgres psql -U postgres -d runescan < deploy/runescan.sql
$ export DATABASE_URL=postgres://postgres:mysecretpassword@127.0.0.1:5432/runescan
$ cargo build
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet create
{
  "mnemonic": "cluster measure flag drastic govern permit voice about argue enable announce major",
  "passphrase": ""
}

$ rm -rf ~/.local/share/ord/regtest/index.redb
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "addresses": [
    "bcrt1pcc84ph5u5f2nvaq67dc2rq42lrkqfmlrvyce46q6j8d3shltp50qjnujg8"
  ]
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1pcc84ph5u5f2nvaq67dc2rq42lrkqfmlrvyce46q6j8d3shltp50qjnujg8

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 5000000000,
  "ordinal": 0,
  "runes": {},
  "runic": 0,
  "total": 5000000000
}

$ cat /tmp/batch.yaml
mode: separate-outputs
parent: null
postage: null
reinscribe: false
etching:
  rune: UNCOMMON•GOODS
  divisibility: 2
  premine: 1000000.00
  supply: 1000000.00
  symbol: $
  turbo: true

inscriptions:
- file: /tmp/batch.yaml

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 batch --fee-rate 1 --batch /tmp/batch.yaml
Waiting for rune UNCOMMONGOODS commitment 138eeaa503808b174becbf6f5c346c0c4dca98406e07b8d85431d8ce7b8496f4 to mature…

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 5 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
Maturing in...[0s]   [████████████████████████████████████████] 6/6
{
  "commit": "138eeaa503808b174becbf6f5c346c0c4dca98406e07b8d85431d8ce7b8496f4",
  "commit_psbt": null,
  "inscriptions": [
    {
      "destination": "bcrt1p4l7lfcncfe8egn6hrh57kppchzksvt2muhnyxkqgzrv5209u8jjqzlewyd",
      "id": "5bdea4a4dafef3eb00dd36c6f4d90ab4c47b306b353583d52184fac8090bf0afi0",
      "location": "5bdea4a4dafef3eb00dd36c6f4d90ab4c47b306b353583d52184fac8090bf0af:0:0"
    }
  ],
  "parent": null,
  "reveal": "5bdea4a4dafef3eb00dd36c6f4d90ab4c47b306b353583d52184fac8090bf0af",
  "reveal_broadcast": true,
  "reveal_psbt": null,
  "rune": {
    "destination": "bcrt1p8wdtay82az6wkdlx9gpl48e5qrre6a6563zsudaknzksda2uctwsydhwqr",
    "location": "5bdea4a4dafef3eb00dd36c6f4d90ab4c47b306b353583d52184fac8090bf0af:1",
    "rune": "UNCOMMON•GOODS"
  },
  "total_fees": 432
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 34999979568,
  "ordinal": 10000,
  "runes": {
    "UNCOMMON•GOODS": "1000000"
  },
  "runic": 10000,
  "total": 34999999568
}

http://192.168.0.111:23456/rune/UNCOMMON%E2%80%A2GOODS
rune_id: 107:1

# sub hub topic
$ dfx canister call omnity_hub sub_directives '(opt "Bitcoin", vec {variant {AddChain};variant {AddToken};variant {UpdateFee};variant {ToggleChainState}})'
$ dfx canister call omnity_hub sub_directives '(opt "eICP", vec {variant {AddChain};variant {AddToken};variant {UpdateFee};variant {ToggleChainState}})'

# Note: replace the canister id to Bitcoin customs canister id
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}; fee_token=null}}})'
$ dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}; fee_token=null}}})'

# Note: replace the canister id to ICP route canister id and constract address
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="br5f7-7uaaa-aaaaa-qaaca-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}; fee_token=opt "LICP"}}})'
$ dfx canister call omnity_hub execute_proposal  '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="br5f7-7uaaa-aaaaa-qaaca-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}; fee_token=opt "LICP"}}})'

$ dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 2 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; name = "UNCOMMON•GOODS";issue_chain = "Bitcoin"; symbol = "UNCOMMON•GOODS"; metadata =  vec{ record {"rune_id"; "107:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'
$ dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 2 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; name = "UNCOMMON•GOODS";issue_chain = "Bitcoin"; symbol = "UNCOMMON•GOODS"; metadata =  vec{ record {"rune_id"; "107:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'

# update fee
$ dfx canister call omnity_hub update_fee 'vec {variant { UpdateTargetChainFactor = record {target_chain_id="Bitcoin"; target_chain_factor=10000 : nat}}; variant { UpdateFeeTokenFactor = record { fee_token="LICP"; fee_token_factor=1 : nat}}}'

$ dfx canister call icp_route get_redeem_fee '("Bitcoin")'
(opt (20_000 : nat64))
# query update fee directive
$ dfx canister call omnity_hub query_directives '(opt "eICP",null,0:nat64,5:nat64)'
$ dfx canister call omnity_hub query_directives '(opt "Bitcoin",null,0:nat64,5:nat64)'

$ dfx canister call icp_route get_token_list
(
  vec {
    record {
      decimals = 2 : nat8;
      token_id = "Bitcoin-runes-UNCOMMON•GOODS";
      icon = opt "rune.logo.url";
      rune_id = opt "107:1";
      symbol = "UNCOMMON•GOODS";
    };
  },
)

$ dfx canister call icp_route get_token_ledger '("Bitcoin-runes-UNCOMMON•GOODS")'
(opt principal "bw4dl-smaaa-aaaaa-qaacq-cai")

# https://internetcomputer.org/docs/current/tutorials/developer-journey/level-4/4.2-icrc-tokens
$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_symbol '()'
("UNCOMMON•GOODS")

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_metadata '()'
(
  vec {
    record { "icrc1:logo"; variant { Text = "rune.logo.url" } };
    record { "icrc1:decimals"; variant { Nat = 2 : nat } };
    record { "icrc1:name"; variant { Text = "UNCOMMON•GOODS" } };
    record { "icrc1:symbol"; variant { Text = "UNCOMMON•GOODS" } };
    record { "icrc1:fee"; variant { Nat = 10_000 : nat } };
    record { "icrc1:max_memo_length"; variant { Nat = 32 : nat } };
  },
)

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_balance_of "(record {owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\"; })"
(0 : nat)

$ dfx canister call bitcoin_customs get_btc_address '(record {target_chain_id = "eICP"; receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"})'
("bcrt1qxy0yjfmty99e3p3rqgt9rrrmv6kqe4u9v4zcca")

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1qxy0yjfmty99e3p3rqgt9rrrmv6kqe4u9v4zcca 70000:UNCOMMON•GOODS
{
  "txid": "7780097b00e974f146c27cba5fdae16f52f6be5053c4c8bad9fccc72466a118b",
  "psbt": "cHNidP8BAOUCAAAAAq/wCwnI+oQh1YM1NWswe8S0Ctn0xjbdAOvz/tqkpN5bAQAAAAD/////9JaEe87YMVTYuAduQJjKTQxsNFxvv+xLF4uAA6XqjhMBAAAAAP3///8EAAAAAAAAAAALal0IAGsBwJ+rAwIQJwAAAAAAACJRIPjxy0vYeiBrPFpcRAkW7SwnI+Uce8PpJmL6kzEGPNxRECcAAAAAAAAWABQxHkknayFLmIYjAhZRjHtmrAzXhRl6BSoBAAAAIlEgag7Dl0p/O5MBJT3I4hDYXAu/U8FsE3FBAdEGo8ZWhN4AAAAAAAEBKxAnAAAAAAAAIlEgO5q+kOrotOs35ioD+p80AMedd1TURQ43tpitBvVcwt0BCEIBQGouraqlmJrpPTWbybEyxaCwVxcrM24S6aAFOmlrOq+6Jzj8b4tmnsHwafBSkaBSTOrosl5brjJgkTrtuhpghlwAAQErMKIFKgEAAAAiUSCOno4/5leCJIjjJGj4Zngmd5N1e/jXQdX4ldeNzb6kiwEIQgFAk0zBwwoOthLr1ETKlCplpiOaFEzkv/cnAmu9ft4FHpRy2sYSlCb3HxYyvvtvZnEw8vhAWhT7F2MlImv4IIjdygAAAQUgRJlZBkXAnlMzGRkgzMPUVCSEml2RxFswGRhm7ZqFUCUhB0SZWQZFwJ5TMxkZIMzD1FQkhJpdkcRbMBkYZu2ahVAlGQCr/H6pVgAAgAEAAIAAAACAAQAAAAQAAAAAAAEFIDoOPAnznFwRmbEpa0/eYjMZFeyYbkR1A7fl3GoWdOkhIQc6DjwJ85xcEZmxKWtP3mIzGRXsmG5EdQO35dxqFnTpIRkAq/x+qVYAAIABAACAAAAAgAEAAAAFAAAAAA==",
  "outgoing": "70000:UNCOMMON•GOODS",
  "fee": 263
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "eICP"; receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"; rune_id = "107:1"; amount = 7000000; txid = "7780097b00e974f146c27cba5fdae16f52f6be5053c4c8bad9fccc72466a118b"})'

$ dfx canister call bitcoin_customs get_pending_gen_ticket_requests '(record {max_count = 3; start_txid = null})'
(
  vec {
    record {
      received_at = 1_715_887_108_893_675_145 : nat64;
      token_id = "Bitcoin-runes-UNCOMMON•GOODS";
      txid = blob "\8b\11\6a\46\72\cc\fc\d9\ba\c8\c4\53\50\be\f6\52\6f\e1\da\5f\ba\7c\c2\46\f1\74\e9\00\7b\09\80\77";
      target_chain_id = "eICP";
      address = "bcrt1qxy0yjfmty99e3p3rqgt9rrrmv6kqe4u9v4zcca";
      amount = 7_000_000 : nat;
      receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe";
      rune_id = record { tx = 1 : nat32; block = 107 : nat64 };
    };
  },
)

$ cargo build -p runes_oracle

export INDEXER_URL=http://localhost:23456
export PEM_PATH=/home/julian/.config/dfx/identity/default/identity.pem
export IC_GATEWAY=http://localhost:4943
export CUSTOMS_CANISTER_ID=be2us-64aaa-aaaaa-qaabq-cai
$ RUST_LOG=info ./target/debug/runes_oracle
$ dfx canister call omnity_hub query_tickets '(opt "eICP", 0, 10)'
(
  variant {
    Ok = vec {
      record {
        0 : nat64;
        record {
          token = "Bitcoin-runes-UNCOMMON•GOODS";
          action = variant { Transfer };
          dst_chain = "eICP";
          memo = null;
          ticket_id = "7780097b00e974f146c27cba5fdae16f52f6be5053c4c8bad9fccc72466a118b";
          sender = null;
          ticket_time = 1_715_887_353_183_737_735 : nat64;
          ticket_type = variant { Normal };
          src_chain = "Bitcoin";
          amount = "7000000";
          receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe";
        };
      };
    }
  },
)

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_balance_of "(record {owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\"; })"
(7_000_000 : nat)


$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc2_approve "(record { amount = 20000; spender = record{owner = principal \"br5f7-7uaaa-aaaaa-qaaca-cai\";} })"
(variant { Ok = 1 : nat })
$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc2_allowance "(record { account = record{owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\";}; spender = record{owner = principal \"br5f7-7uaaa-aaaaa-qaaca-cai\";} })"
(record { allowance = 20_000 : nat; expires_at = null })

$ dfx canister call icp_route get_fee_account '(null)'
(
  blob "\00\3b\7d\df\13\af\eb\a2\16\bb\7d\13\eb\d9\63\ca\58\a1\be\af\0a\07\ce\78\5c\e8\35\1c\ea\c3\74\c5",
)

$ dfx ledger transfer 003b7ddf13afeba216bb7d13ebd963ca58a1beaf0a07ce785ce8351ceac374c5 --memo 1 --amount 2
Transfer sent at block height 1
$ dfx ledger balance 003b7ddf13afeba216bb7d13ebd963ca58a1beaf0a07ce785ce8351ceac374c5
2.00000000 ICP

$ dfx canister call icp_route generate_ticket '(record {target_chain_id = "Bitcoin"; receiver = "bcrt1p38mc9erwfmkmssvs5w55fknq8x3wkq972j72ue5mv8hy35pfc4pssanhuf"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; amount = 20000})'
(variant { Ok = record { ticket_id = "bw4dl-smaaa-aaaaa-qaacq-cai_2" } })

$ dfx canister call omnity_hub query_tickets '(opt "Bitcoin", 0, 10)'
(
  variant {
    Ok = vec {
      record {
        0 : nat64;
        record {
          token = "Bitcoin-runes-UNCOMMON•GOODS";
          action = variant { Redeem };
          dst_chain = "Bitcoin";
          memo = null;
          ticket_id = "bw4dl-smaaa-aaaaa-qaacq-cai_2";
          sender = null;
          ticket_time = 1_715_887_496_186_488_321 : nat64;
          ticket_type = variant { Normal };
          src_chain = "eICP";
          amount = "20000";
          receiver = "bcrt1p38mc9erwfmkmssvs5w55fknq8x3wkq972j72ue5mv8hy35pfc4pssanhuf";
        };
      };
    }
  },
)

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
# $ dfx canister call bitcoin_customs update_btc_utxos
$ dfx canister call bitcoin_customs get_events '(record {start = 0; length = 100})'
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 44999969305,
  "ordinal": 10000,
  "runes": {
    "UNCOMMON•GOODS": "930200"
  },
  "runic": 10546,
  "total": 44999989851
}

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_balance_of "(record {owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\"; })"
(6_970_000 : nat)

# -------------------redeem token script--------------------------------
# deploy the hub_mock canister instead of omnity_hub
# when deploying customs, set the hub_principal parameter to the canister id of mock_hub
$ dfx deploy hub_mock --mode reinstall -y

# Before starting redeem, you need to complete the customs -> execution crosschain process to ensure that there is a runes balance in customs.

# deposit BTC as the fee of redeem tx
$ dfx canister call bitcoin_customs get_main_btc_address 'BTC'
("bcrt1q72ycas7f7h0wfv8egqh6vfzhurlaeket33l4qa")

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf -rpcwallet=test1 sendtoaddress 'bcrt1q72ycas7f7h0wfv8egqh6vfzhurlaeket33l4qa' 1
$ bitcoin-cli -conf=$(pwd)/bitcoin.conf -rpcwallet=test1 -generate 1
# wait 10 seconds, make sure there is utxo in the returned result
$ dfx canister call bitcoin_customs update_btc_utxos

$ dfx canister call hub_mock push_ticket '(record {ticket_id='xxx'...})'
# wait 10 seconds, query the event to confirm that the transaction has been sent
$ dfx canister call bitcoin_customs get_events '(record {start = 0; length = 100})'