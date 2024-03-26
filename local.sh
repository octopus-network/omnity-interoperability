
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ dfx stop
$ dfx start --clean
$ dfx deploy omnity_hub
$ dfx identity --identity default get-principal
o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe
$ dfx deploy bitcoin_customs --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 1; max_time_in_queue_nanos = 1_000_000_000; runes_oracle_principal = principal "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"; chain_id = "Bitcoin" } })' # 20 mins for testnet/prod

# https://github.com/lesterli/ord/blob/docs/runes/docs/src/guides/runes.md
$ git clone https://github.com/octopus-network/ord.git
$ git checkout runescan
$ sudo docker run --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=mysecretpassword -v ~/dev/data:/var/lib/postgresql/data -d postgres:12
$ sudo docker run -it --rm --network host postgres:12 psql -h 127.0.0.1 -U postgres
postgres=# CREATE DATABASE runescan ENCODING = 'UTF8';
$ sudo docker exec -i postgres psql -U postgres -d runescan < deploy/runescan.sql
$ export DATABASE_URL=postgres://postgres:mysecretpassword@127.0.0.1:5432/runescan
$ cargo build
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet create
{
  "mnemonic": "wool evoke deliver detail zebra little found until genius unlock large fix",
  "passphrase": ""
}

$ rm -rf ~/.local/share/ord/regtest/index.redb
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "addresses": [
    "bcrt1pt0wl5fmf704r2qtlfa4znzcu7hp5m5dzj3qqyhdq7asqgspywdfqe3ae20"
  ]
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1pt0wl5fmf704r2qtlfa4znzcu7hp5m5dzj3qqyhdq7asqgspywdfqe3ae20

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 5000000000,
  "ordinal": 0,
  "runes": {},
  "runic": 0,
  "total": 5000000000
}

$ cat /tmp/batch.yaml
inscriptions:
- delegate: null
  destination: null
  file: /tmp/inscription.txt
  metadata: null
  metaprotocol: null
  satpoint: null
mode: separate-outputs
parent: null
postage: null
reinscribe: false
etch:
  divisibility: 1
  mint: null
  premine: '1000'
  rune: FIRST•RUNE•TOKEN
  symbol: '¢'
sat: null
satpoint: null

$ cat /tmp/inscription.txt
FOO

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 inscribe --fee-rate 1 --batch /tmp/batch.yaml
Waiting for rune commitment to mature…

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 6 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

{
  "commit": "fab3af59aa2445b71301b8da2da003136befeceb772a21b3c59ab06324743908",
  "commit_psbt": null,
  "inscriptions": [
    {
      "destination": "bcrt1pvy9z73v5k8mavyv2pu88358kx7a6qagjjhpuklpv0aw6phax8fdq2n6ljv",
      "id": "63758ef37d14ad9d15a5e8b75219698b04d3e8b524d1e209811bdc061f167b12i0",
      "location": "63758ef37d14ad9d15a5e8b75219698b04d3e8b524d1e209811bdc061f167b12:0:0"
    }
  ],
  "parent": null,
  "reveal": "63758ef37d14ad9d15a5e8b75219698b04d3e8b524d1e209811bdc061f167b12",
  "reveal_psbt": null,
  "rune": {
    "destination": "bcrt1pl4wd2qcw3h4dndycmr5fv6a833exkw2h00s840t8wsk4jzly0ppq5wv6n4",
    "location": "63758ef37d14ad9d15a5e8b75219698b04d3e8b524d1e209811bdc061f167b12:1",
    "rune": "FIRST•RUNE•TOKEN"
  },
  "total_fees": 378
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 39999979622,
  "ordinal": 10000,
  "runes": {
    "FIRSTRUNETOKEN": 10000
  },
  "runic": 10000,
  "total": 39999999622
}

http://192.168.1.105:23456/rune/FIRST%E2%80%A2RUNE%E2%80%A2TOKEN
rune_id: 108:1

# Note: replace the canister id to Bitcoin customs canister id
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}}}})'
$ dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}}}})'

# Note: replace the canister id to ICP route canister id and constract address
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="rahyp-xyaaa-aaaag-qcwha-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}}}})'
$ dfx canister call omnity_hub execute_proposal  '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="rahyp-xyaaa-aaaag-qcwha-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}}}})'

$ dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 1 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-FIRST•RUNE•TOKEN"; settlement_chain = "Bitcoin"; symbol = "FIRST•RUNE•TOKEN"; metadata = opt vec{ record {"rune_id"; "108:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'
$ dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 1 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-FIRST•RUNE•TOKEN"; settlement_chain = "Bitcoin"; symbol = "FIRST•RUNE•TOKEN"; metadata = opt vec{ record {"rune_id"; "108:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'

$ dfx canister call omnity_hub query_dires '(opt "Bitcoin",null,0:nat64,5:nat64)'
(
  variant {
    Ok = vec {
      record {
        0 : nat64;
        variant {
          AddChain = record {
            chain_id = "eICP";
            chain_state = variant { Active };
            chain_type = variant { ExecutionChain };
            contract_address = null;
          }
        };
      };
      record {
        1 : nat64;
        variant {
          AddToken = record {
            decimals = 1 : nat8;
            token_id = "Bitcoin-runes-FIRST•RUNE•TOKEN";
            metadata = opt vec { record { "rune_id"; "108:1" } };
            icon = opt "rune.logo.url";
            issue_chain = "Bitcoin";
            symbol = "FIRST•RUNE•TOKEN";
          }
        };
      };
    }
  },
)

$ dfx canister call bitcoin_customs get_btc_address '(record {target_chain_id = "eICP"; receiver = "glfzm-3xumc-23ch4-znudm-hs76m-ffyre-k7yxu-ct342-2emb7-lm4wa-3qe"})'
("bcrt1q38whd92q8ln57hxhvj0jvfev4yhsg50z7nrz8r")

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1q38whd92q8ln57hxhvj0jvfev4yhsg50z7nrz8r 7FIRST•RUNE•TOKEN
{
  "txid": "c3a4206ca076ed144e865c55f1df21572670bce1d615ae88ad6b2128a66697f0",
  "psbt": "cHNidP8BAOICAAAAAhJ7Fh8G3BuBCeLRJLXo0wSLaRlSt+ilFZ2tFH3zjnVjAQAAAAD/////CDl0JGOwmsWzISp36+zvaxMDoC3auAETt0Ukqlmvs/oBAAAAAP3///8EAAAAAAAAAAAIal0FAGwBRgIQJwAAAAAAACJRIIl0ljfJH0MSVeFpMh8Y4c6rOpCZEIZ9GKVauDGR2rldECcAAAAAAAAWABSJ3XaVQD/nT1zXZJ8mJyypLwRR4lJ6BSoBAAAAIlEgzzPA65tqgCfljcAybm31ugAlOaZZL1YwKm9Bf0g8MzMAAAAAAAEBKxAnAAAAAAAAIlEg/VzVAw6N6tm0mNjolmunjHJrOVd74Hq9Z3QtWQvkeEIBCEIBQCEug1LB2Ea2dvIOjivRyrugengSqqpUQ5pnABdZqgY4lXfegLGORZXn5wmgvKpmoVvyvBX09OjD7cDEem0j3NEAAQErZqIFKgEAAAAiUSDHmuLZdITRLXiyDLWBXNAf1isUsa2HSzACrl19jkFKiQEIQgFAV3WBUaFiSPsbm5b/B0R3c3wk6oPT+B+IdaQk9Y99puI8uvLwoOVb1tdySYVOxl4iVfTq9BU6NHlBi8TDU90EiAAAAQUg3nDgjZmwsRkoSmAaG4BnCuChSRJ4qHK3y4ei9qCiem0hB95w4I2ZsLEZKEpgGhuAZwrgoUkSeKhyt8uHovagonptGQB2TOC1VgAAgAEAAIAAAACAAQAAAAQAAAAAAAEFIJzRpjfknyAdc+83jO9EZ1JsVNbRrvC5KkP3iEqjZktyIQec0aY35J8gHXPvN4zvRGdSbFTW0a7wuSpD94hKo2ZLchkAdkzgtVYAAIABAACAAAAAgAEAAAAFAAAAAA==",
  "outgoing": "7 FIRST•RUNE•TOKEN",
  "fee": 260
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww



$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "eICP"; receiver = "bcrt1q38whd92q8ln57hxhvj0jvfev4yhsg50z7nrz8r"; rune_id = "108:1"; amount = 70; txid = "c3a4206ca076ed144e865c55f1df21572670bce1d615ae88ad6b2128a66697f0"})'
(variant { Err = variant { NoNewUtxos } })

$ dfx canister call bitcoin_customs update_btc_utxos
$ dfx canister call bitcoin_customs get_pending_gen_ticket_requests
(
  vec {
    record {
      received_at = 1_710_158_505_600_484_656 : nat64;
      txid = blob "\a4\9d\a4\5b\48\05\8f\f7\7a\ff\4f\5b\5c\c3\42\6a\71\a3\00\1f\bf\2b\77\dd\ce\f8\65\47\63\2f\e4\c6";
      target_chain_id = "cosmoshub";
      address = "bcrt1q9jvz3tkk0nptsx8tw8chvjz03h77fvf8dy66z2";
      amount = 70 : nat;
      receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur";
      rune_id = record { height = 102 : nat32; index = 1 : nat16 };
    };
  },
)


#$ dfx canister call omnity_hub set_whitelist '(principal "be2us-64aaa-aaaaa-qaabq-cai", true)'

export INDEXER_URL=http://localhost:23456
export PEM_PATH=/home/julian/.config/dfx/identity/default/identity.pem
export IC_GATEWAY=http://localhost:4943
export CUSTOMS_CANISTER_ID=be2us-64aaa-aaaaa-qaabq-cai
$ RUST_LOG=info ./target/debug/runes_oracle
$ dfx canister call omnity_hub query_tickets '(opt "cosmoshub", 0, 10)'
(
  variant {
    Ok = vec {
      record {
        0 : nat64;
        record {
          token = "102:1";
          action = variant { Transfer };
          dst_chain = "cosmoshub";
          memo = null;
          ticket_id = "c6e42f634765f8cedd772bbf1f00a3716a42c35c5b4fff7af78f05485ba49da4";
          sender = "";
          ticket_time = 1_710_159_448_141_081_850 : nat64;
          src_chain = "BTC";
          amount = "70";
          receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur";
        };
      };
    }
  },
)





$ dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21"; ticket_time = 1707291817947 : nat64; token = "102:1"; amount = "10"; src_chain = "cosmoshub"; dst_chain = "Bitcoin"; action = variant { Redeem }; sender = opt "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"; receiver = "bcrt1q72ycas7f7h0wfv8egqh6vfzhurlaeket33l4qa"; memo = null;})'
$ dfx canister call omnity_hub query_tickets '(opt "Bitcoin", 0, 10)'
(
  variant {
    Ok = vec {
      record {
        0 : nat64;
        record {
          token = "102:1";
          action = variant { Redeem };
          dst_chain = "BTC";
          memo = null;
          ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21";
          sender = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur";
          ticket_time = 1_707_291_817_947 : nat64;
          src_chain = "cosmoshub";
          amount = "10";
          receiver = "bcrt1q72ycas7f7h0wfv8egqh6vfzhurlaeket33l4qa";
        };
      };
    }
  },
)
$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
$ dfx canister call bitcoin_customs get_events '(record {start = 0; length = 100})'

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
