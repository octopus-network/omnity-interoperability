
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ dfx stop
$ dfx start --clean
$ cd omnity
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

$ dfx deploy omnity_hub --argument '(variant { Init = record { admin = principal "cu4zh-2c4it-54irp-xgtxc-gajvr-h6gle-c5n7r-hwpeg-spkye-z4ta7-iae"} })'
$ dfx identity --identity default get-principal
o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe
$ dfx deploy bitcoin_customs --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 1; max_time_in_queue_nanos = 1_000_000_000; runes_oracle_principal = principal "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"; chain_id = "Bitcoin" } })'
$ dfx deploy icp_route --argument '(variant { Init = record { hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; chain_id = "eICP" } })'

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
  "mnemonic": "whisper canvas boss remove report ivory pill satoshi direct choose museum spread",
  "passphrase": ""
}

$ rm -rf ~/.local/share/ord/regtest/index.redb
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "addresses": [
    "bcrt1psgwf8p9p45xeycm49jlkq5dqcmlc4y2xmwgyyla0ccz67rjaa5wqkkw7ts"
  ]
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1psgwf8p9p45xeycm49jlkq5dqcmlc4y2xmwgyyla0ccz67rjaa5wqkkw7ts

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
  premine: 1000.00
  supply: 10000.00
  symbol: $
  terms:
    amount: 100.00
    cap: 90
    height:
      start: 840000
      end: 850000
    offset:
      start: 1000
      end: 9000

inscriptions:
- file: /tmp/inscription.txt
  delegate: null
  destination: null
  metadata: null

$ cat /tmp/inscription.txt
FOO

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 batch --fee-rate 1 --batch /tmp/batch.yaml
Waiting for rune commitment 33853b36b89697209a45a6d961ee6b3abab7f1a148d92f38c52e24d15d61016a to mature…

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 6 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

{
  "commit": "33853b36b89697209a45a6d961ee6b3abab7f1a148d92f38c52e24d15d61016a",
  "commit_psbt": null,
  "inscriptions": [
    {
      "destination": "bcrt1pey0f6dxwvjpu2z7ndxt8wat98rt5ke9ys3hcms9dqpc7s4wnyckqfaalvx",
      "id": "a355fb48f0d3ef781d8fc63fdd04b72e9847a3d41095cb02008175843b5f32c8i0",
      "location": "a355fb48f0d3ef781d8fc63fdd04b72e9847a3d41095cb02008175843b5f32c8:0:0"
    }
  ],
  "parent": null,
  "reveal": "a355fb48f0d3ef781d8fc63fdd04b72e9847a3d41095cb02008175843b5f32c8",
  "reveal_broadcast": true,
  "reveal_psbt": null,
  "rune": {
    "destination": "bcrt1p0mvu63whr40pmwcfxg84pxmhrrddc2whljdqy5u0v228zgmsv9yspznwz5",
    "location": "a355fb48f0d3ef781d8fc63fdd04b72e9847a3d41095cb02008175843b5f32c8:1",
    "rune": "UNCOMMON•GOODS"
  },
  "total_fees": 395
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 39999979605,
  "ordinal": 10000,
  "runes": {
    "UNCOMMON•GOODS": 100000
  },
  "runic": 10000,
  "total": 39999999605
}

http://192.168.1.105:23456/rune/UNCOMMON%E2%80%A2GOODS
rune_id: 108:1

# Note: replace the canister id to Bitcoin customs canister id
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}; fee_token=null}}})'
$ dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}; fee_token=null}}})'

# Note: replace the canister id to ICP route canister id and constract address
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="br5f7-7uaaa-aaaaa-qaaca-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}; fee_token=opt "LICP"}}})'
$ dfx canister call omnity_hub execute_proposal  '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="br5f7-7uaaa-aaaaa-qaaca-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}; fee_token=opt "LICP"}}})'

$ dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 2 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; name = "UNCOMMON•GOODS";issue_chain = "Bitcoin"; symbol = "UNCOMMON•GOODS"; metadata =  vec{ record {"rune_id"; "108:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'
$ dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 2 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; name = "UNCOMMON•GOODS";issue_chain = "Bitcoin"; symbol = "UNCOMMON•GOODS"; metadata =  vec{ record {"rune_id"; "108:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'

# update fee
$ dfx canister call omnity_hub update_fee 'vec {variant { UpdateTargetChainFactor = record {target_chain_id="Bitcoin"; target_chain_factor=10000 : nat}}; variant { UpdateFeeTokenFactor = record { fee_token="LICP"; fee_token_factor=1 : nat}}}'

$ dfx canister call icp_route get_redeem_fee '("Bitcoin")'
(opt (20_000 : nat64))
# query update fee directive
$ dfx canister call omnity_hub query_directives '(opt "eICP",opt variant {UpdateFee=opt "LICP"},0:nat64,5:nat64)'
$ dfx canister call omnity_hub query_directives '(opt "Bitcoin",null,0:nat64,5:nat64)'

$ dfx canister call icp_route get_token_list
(
  vec {
    record {
      decimals = 2 : nat8;
      token_id = "Bitcoin-runes-UNCOMMON•GOODS";
      metadata = opt vec { record { "rune_id"; "108:1" } };
      icon = opt "rune.logo.url";
      issue_chain = "Bitcoin";
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
    record { "icrc1:decimals"; variant { Nat = 2 : nat } };
    record { "icrc1:name"; variant { Text = "Bitcoin-runes-UNCOMMON•GOODS" } };
    record { "icrc1:symbol"; variant { Text = "UNCOMMON•GOODS" } };
    record { "icrc1:fee"; variant { Nat = 0 : nat } };
    record { "icrc1:max_memo_length"; variant { Nat = 32 : nat } };
  },
)

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_balance_of "(record {owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\"; })"
(0 : nat)

$ dfx canister call bitcoin_customs get_btc_address '(record {target_chain_id = "eICP"; receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"})'
("bcrt1qrunh3ypertlpw5ufa0j5l3pq8tj0hppu7urzg4")

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1qrunh3ypertlpw5ufa0j5l3pq8tj0hppu7urzg4 7:UNCOMMON•GOODS
{
  "txid": "3ac371053f8306ff43b13a7321f06db5e38f0b35b303c555791d84cd9e850aab",
  "psbt": "cHNidP8BAOMCAAAAAsgyXzuEdYEAAsuVENSjR5gutwTdP8aPHXjv0/BI+1WjAQAAAAD/////agFhXdEkLsU4L9lIofG3ujpr7mHZpkWaIJeWuDY7hTMBAAAAAP3///8EAAAAAAAAAAAJal0GAGwBvAUCECcAAAAAAAAiUSC8a0sWT7OqycWNpT4SLjgh0/n+rcScRAsEU9r9vlTflBAnAAAAAAAAFgAUHyd4kDka/hdTievlT8QgOuT7hDxAegUqAQAAACJRICjLEB/kuNka/iCa9Sf2E+baJ0FY5Lc/huVtJ4HehpRJAAAAAAABASsQJwAAAAAAACJRIH7ZzUXXHV4duwkyD1Cbdxja3CnX/JoCU49ilHEjcGFJAQhCAUDkWz5AST6xDe4/Voqyc5fYvtcwMK6L7uLVLSjjoUjMBcAfjqyRWa1r6PqWp1t8kWfnpNZF9gOiAhLm/xtmfDEFAAEBK1WiBSoBAAAAIlEgYKNmBiJZ2DagIaAdhfPPHz5De2GlYChnPcYyFkpzJ2oBCEIBQMwvIGhvzvOiZ0ugkyuPzfnCdu+JS9wrdCJdGNiszTviUFAGH/XlAhG6bPViYIpsTTbgSbVX23DjR8eemKepP6QAAAEFIPtTnY+2hehuR48FdPDpNdSg5f0JVfPMNmphHG2+82QeIQf7U52PtoXobkePBXTw6TXUoOX9CVXzzDZqYRxtvvNkHhkAeY20XVYAAIABAACAAAAAgAEAAAAEAAAAAAABBSCNf63lqs6BZ6cv5LSFoaznH5eXrU6WO/3C+rfQ8njA2iEHjX+t5arOgWenL+S0haGs5x+Xl61Oljv9wvq30PJ4wNoZAHmNtF1WAACAAQAAgAAAAIABAAAABQAAAAA=",
  "outgoing": "7:UNCOMMON•GOODS",
  "fee": 261
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 6 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "eICP"; receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"; rune_id = "108:1"; amount = 700; txid = "3ac371053f8306ff43b13a7321f06db5e38f0b35b303c555791d84cd9e850aab"})'

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
          ticket_id = "3ac371053f8306ff43b13a7321f06db5e38f0b35b303c555791d84cd9e850aab";
          sender = null;
          ticket_time = 1_712_656_790_557_746_044 : nat64;
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


$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc2_approve "(record { amount = 100; spender = record{owner = principal \"br5f7-7uaaa-aaaaa-qaaca-cai\";} })"
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

$ dfx canister call icp_route generate_ticket '(record {target_chain_id = "Bitcoin"; receiver = "bcrt1psgwf8p9p45xeycm49jlkq5dqcmlc4y2xmwgyyla0ccz67rjaa5wqkkw7ts"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; amount = 100})'

$ dfx canister call omnity_hub query_tickets '(opt "Bitcoin", 0, 10)'
(
  variant {
    Ok = vec {
      record {
        1 : nat64;
        record {
          token = "Bitcoin-runes-UNCOMMON•GOODS";
          action = variant { Redeem };
          dst_chain = "Bitcoin";
          memo = null;
          ticket_id = "2";
          sender = null;
          ticket_time = 1_712_656_910_883_921_827 : nat64;
          src_chain = "eICP";
          amount = "100";
          receiver = "bcrt1psgwf8p9p45xeycm49jlkq5dqcmlc4y2xmwgyyla0ccz67rjaa5wqkkw7ts";
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