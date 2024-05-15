
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ cd omnity
$ dfx stop
$ dfx start --clean
$ cargo clean

# https://internetcomputer.org/docs/current/developer-docs/defi/icp-tokens/ledger-local-setup
$ dfx identity new minter
$ dfx identity use minter
$ export MINTER_ACCOUNT_ID=$(dfx ledger account-id)
$ dfx identity use default
$ export DEFAULT_ACCOUNT_ID=$(dfx ledger account-id)
$ dfx deploy --specified-id ryjl3-tyaaa-aaaaa-aaaba-cai icp_ledger_canister --argument "
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
$ dfx ledger balance $DEFAULT_ACCOUNT_ID
100.00000000 ICP

$ dfx identity --identity default get-principal
o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe
$ dfx deploy omnity_hub --argument '(variant { Init = record { admin = principal "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"} })'
$ dfx deploy bitcoin_customs --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 1; max_time_in_queue_nanos = 1_000_000_000; runes_oracle_principal = principal "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"; chain_id = "Bitcoin"; chain_state = variant { Active } } })'
$ dfx deploy icp_route --argument '(variant { Init = record { hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; chain_id = "eICP"; chain_state = variant { Active } } })'

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
  "mnemonic": "save mutual foil conduct quick know outer journey duty crumble funny naive",
  "passphrase": ""
}

$ rm -rf ~/.local/share/ord/regtest/index.redb
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "addresses": [
    "bcrt1pnjnu8ncexusk6q4kzh8yccdh7yylgj0atw2cmx6d3a8j36hkgqzs7pxwhz"
  ]
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1pnjnu8ncexusk6q4kzh8yccdh7yylgj0atw2cmx6d3a8j36hkgqzs7pxwhz

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
Waiting for rune UNCOMMONGOODS commitment a6d183ac9fb7ad386977b0240cc354d72f87e195b220784d780465b3b8bceaf3 to mature…

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 6 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
Maturing in...[0s]   [████████████████████████████████████████] 6/6
{ "commit": "a6d183ac9fb7ad386977b0240cc354d72f87e195b220784d780465b3b8bceaf3",
  "commit_psbt": null,
  "inscriptions": [
    {
      "destination": "bcrt1pt9857un27rvl9j58k05g6094t2p6zt938vxu5z3un69u9h3qpywsxazcaz",
      "id": "5da9f5c4e4c257ef909ee83e7b3386192a4dfeee80ea7bc747205cb446bc4116i0",
      "location": "5da9f5c4e4c257ef909ee83e7b3386192a4dfeee80ea7bc747205cb446bc4116:0:0"
    }
  ],
  "parent": null,
  "reveal": "5da9f5c4e4c257ef909ee83e7b3386192a4dfeee80ea7bc747205cb446bc4116",
  "reveal_broadcast": true,
  "reveal_psbt": null,
  "rune": {
    "destination": "bcrt1pddhv0xuwe5u25mannvsqradu23j2q8hd0zg99l60k65v8sww0qusrawlu2",
    "location": "5da9f5c4e4c257ef909ee83e7b3386192a4dfeee80ea7bc747205cb446bc4116:1",
    "rune": "UNCOMMON•GOODS"
  },
  "total_fees": 490
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 34999979510,
  "ordinal": 10000,
  "runes": {
    "UNCOMMON•GOODS": "1000000"
  },
  "runic": 10000,
  "total": 34999999510
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
("bcrt1q8gv7wyj9pzvjr0jx3wu5rv4njnuwfpzaxpuddc")

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1q8gv7wyj9pzvjr0jx3wu5rv4njnuwfpzaxpuddc 7:UNCOMMON•GOODS
{
  "txid": "24791f9f6c48275f185c3462ff9e287f69e05e7d1333390abaf7de888d24d271",
  "psbt": "cHNidP8BAOMCAAAAAhZBvEa0XCBHx3vqgO7+TSoZhjN7PuiekO9XwuTE9aldAQAAAAD/////8+q8uLNlBHhNeCCyleGHL9dUwwwksHdpOK23n6yD0aYBAAAAAP3///8EAAAAAAAAAAAJal0GAGsBvAUCECcAAAAAAAAiUSCLMghwbnY4YveRioG65Y6SAdxkfKPvu0TtNaO9l7IBlBAnAAAAAAAAFgAUOhnnEkUImSG+Rou5QbKzlPjkhF3heQUqAQAAACJRIDJtwBTiilUaT8mT3LOezP895qab741fM5mnL8MhBc5PAAAAAAABASsQJwAAAAAAACJRIGtux5uOzTiqb7ObIAH1vFRkoB7teJBS/0+2qMPBzng5AQhCAUD1L2FFwARIqHRqZ6v3Qtk7YkXwuDW26Q1Dk6AqLFua8pJV5UAsBNavYl9G1FUELmcfptJGPOF+4vT1FKSG9riFAAEBK/ahBSoBAAAAIlEgXATpZGCt7tcs6xKHs2s9tuTlp5UMh5glkYr23g+AJaYBCEIBQDNpbiOz217AOxGfETXfnC6LXM9xjVS5mLgITH8rVFaXg4n4jX/8TlvH8/ZFLV3pha8UTUuFpmB83OYEQ0zRRZwAAAEFIOhUmzCRUCi+WSEwLVefgDzTtYsreuswyYi7Q7OuHlO8IQfoVJswkVAovlkhMC1Xn4A807WLK3rrMMmIu0Ozrh5TvBkAKD/YglYAAIABAACAAAAAgAEAAAAEAAAAAAABBSAppZKzK3NiY/MXMu7mjP2PVC9Ituj7Zlmwv5bYwwEUxSEHKaWSsytzYmPzFzLu5oz9j1QvSLbo+2ZZsL+W2MMBFMUZACg/2IJWAACAAQAAgAAAAIABAAAABQAAAAA=",
  "outgoing": "7:UNCOMMON•GOODS",
  "fee": 261
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "eICP"; receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"; rune_id = "107:1"; amount = 700; txid = "24791f9f6c48275f185c3462ff9e287f69e05e7d1333390abaf7de888d24d271"})'

$ dfx canister call bitcoin_customs get_pending_gen_ticket_requests '(record {max_count = 3; start_txid = null})'
(
  vec {
    record {
      received_at = 1_712_656_596_433_950_685 : nat64;
      token_id = "Bitcoin-runes-UNCOMMON•GOODS";
      txid = blob "\ab\0a\85\9e\cd\84\1d\79\55\c5\03\b3\35\0b\8f\e3\b5\6d\f0\21\73\3a\b1\43\ff\06\83\3f\05\71\c3\3a";
      target_chain_id = "eICP";
      address = "bcrt1qrunh3ypertlpw5ufa0j5l3pq8tj0hppu7urzg4";
      amount = 700 : nat;
      receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe";
      rune_id = record { tx = 1 : nat32; block = 108 : nat32 };
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
          ticket_id = "24791f9f6c48275f185c3462ff9e287f69e05e7d1333390abaf7de888d24d271";
          sender = null;
          ticket_time = 1_715_789_849_224_042_721 : nat64;
          ticket_type = variant { Normal };
          src_chain = "Bitcoin";
          amount = "700";
          receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe";
        };
      };
    }
  },
)

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_balance_of "(record {owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\"; })"
(700 : nat)


$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc2_approve "(record { amount = 20000; spender = record{owner = principal \"br5f7-7uaaa-aaaaa-qaaca-cai\";} })"
(variant { Ok = 1 : nat })
$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc2_allowance "(record { account = record{owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\";}; spender = record{owner = principal \"br5f7-7uaaa-aaaaa-qaaca-cai\";} })"
(record { allowance = 100 : nat; expires_at = null })

$ dfx canister call icp_route get_fee_account '(null)'
(
  blob "\00\3b\7d\df\13\af\eb\a2\16\bb\7d\13\eb\d9\63\ca\58\a1\be\af\0a\07\ce\78\5c\e8\35\1c\ea\c3\74\c5",
)

$ dfx ledger transfer 003b7ddf13afeba216bb7d13ebd963ca58a1beaf0a07ce785ce8351ceac374c5 --memo 1 --amount 2
Transfer sent at block height 1
$ dfx ledger balance 003b7ddf13afeba216bb7d13ebd963ca58a1beaf0a07ce785ce8351ceac374c5
2.00000000 ICP

$ dfx canister call icp_route generate_ticket '(record {target_chain_id = "Bitcoin"; receiver = "bcrt1pnjnu8ncexusk6q4kzh8yccdh7yylgj0atw2cmx6d3a8j36hkgqzs7pxwhz"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; amount = 20000})'

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
          ticket_id = "bw4dl-smaaa-aaaaa-qaacq-cai_5";
          sender = null;
          ticket_time = 1_715_791_725_252_746_366 : nat64;
          ticket_type = variant { Normal };
          src_chain = "eICP";
          amount = "20000";
          receiver = "bcrt1pnjnu8ncexusk6q4kzh8yccdh7yylgj0atw2cmx6d3a8j36hkgqzs7pxwhz";
        };
      };
    }
  },
)

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
$ dfx canister call bitcoin_customs update_btc_utxos
$ dfx canister call bitcoin_customs get_events '(record {start = 0; length = 100})'
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 74999969344,
  "ordinal": 10000,
  "runes": {
    "UNCOMMON•GOODS": 99400
  },
  "runic": 10546,
  "total": 74999989890
}

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