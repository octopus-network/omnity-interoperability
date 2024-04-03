
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ dfx stop
$ dfx start --clean
$ cd omnity
$ cargo clean
$ dfx deploy omnity_hub
$ dfx identity --identity default get-principal
o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe
$ dfx deploy bitcoin_customs --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 1; max_time_in_queue_nanos = 1_000_000_000; runes_oracle_principal = principal "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"; chain_id = "Bitcoin" } })' # 20 mins for testnet/prod
$ dfx deploy icp_route --argument '(variant { Init = record { hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; chain_id = "eICP" } })'

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
  "mnemonic": "lock such entire bean screen inside push mystery copy mask quiz lend",
  "passphrase": ""
}

$ rm -rf ~/.local/share/ord/regtest/index.redb
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "addresses": [
    "bcrt1pjr98ggycsnrkl555lg7z37pfyxwl62eseyms4qdpt8eer46nwkgslhamtu"
  ]
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1pjr98ggycsnrkl555lg7z37pfyxwl62eseyms4qdpt8eer46nwkgslhamtu

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
  premine: '100000000'
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
  "commit": "50f03ac8512c67b6e5ab8ec732e39aeababe496683aa623344610e568557d025",
  "commit_psbt": null,
  "inscriptions": [
    {
      "destination": "bcrt1pz7cnc33wgrjuy9jh030dc3mjhctlju9c6wwqghx9aakcze5axh4q47xhnp",
      "id": "f83a8f1d5e2403c600978544c71c2c4a44c0ed7681f88881accc6c5de8a5b46ci0",
      "location": "f83a8f1d5e2403c600978544c71c2c4a44c0ed7681f88881accc6c5de8a5b46c:0:0"
    }
  ],
  "parent": null,
  "reveal": "f83a8f1d5e2403c600978544c71c2c4a44c0ed7681f88881accc6c5de8a5b46c",
  "reveal_psbt": null,
  "rune": {
    "destination": "bcrt1pxj4l2sve0cpnggrax7pct0tgvqz6lznl8d2f7ervspm22u6xsdaq6m6gww",
    "location": "f83a8f1d5e2403c600978544c71c2c4a44c0ed7681f88881accc6c5de8a5b46c:1",
    "rune": "FIRST•RUNE•TOKEN"
  },
  "total_fees": 381
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 39999979619,
  "ordinal": 10000,
  "runes": {
    "FIRSTRUNETOKEN": 1000000000
  },
  "runic": 10000,
  "total": 39999999619
}

http://192.168.1.105:23456/rune/FIRST%E2%80%A2RUNE%E2%80%A2TOKEN
rune_id: 108:1

# Note: replace the canister id to Bitcoin customs canister id
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}}}})'
$ dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}}}})'

# Note: replace the canister id to ICP route canister id and constract address
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="br5f7-7uaaa-aaaaa-qaaca-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}}}})'
$ dfx canister call omnity_hub execute_proposal  '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="br5f7-7uaaa-aaaaa-qaaca-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}}}})'

$ dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 1 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-FIRST•RUNE•TOKEN"; settlement_chain = "Bitcoin"; symbol = "FIRST•RUNE•TOKEN"; metadata = opt vec{ record {"rune_id"; "108:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'
$ dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 1 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-FIRST•RUNE•TOKEN"; settlement_chain = "Bitcoin"; symbol = "FIRST•RUNE•TOKEN"; metadata = opt vec{ record {"rune_id"; "108:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'

$ dfx canister call omnity_hub query_directives '(opt "Bitcoin",null,0:nat64,5:nat64)'
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

$ dfx canister call icp_route get_token_list
(
  vec {
    record {
      decimals = 1 : nat8;
      token_id = "Bitcoin-runes-FIRST•RUNE•TOKEN";
      metadata = opt vec { record { "rune_id"; "108:1" } };
      icon = opt "rune.logo.url";
      issue_chain = "Bitcoin";
      symbol = "FIRST•RUNE•TOKEN";
    };
  },
)

$ dfx canister call icp_route get_token_ledger '("Bitcoin-runes-FIRST•RUNE•TOKEN")'
(opt principal "bw4dl-smaaa-aaaaa-qaacq-cai")

# https://internetcomputer.org/docs/current/tutorials/developer-journey/level-4/4.2-icrc-tokens
$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_symbol '()'
("FIRST•RUNE•TOKEN")

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_balance_of "(record {owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\"; })"
(0 : nat)

$ dfx canister call bitcoin_customs get_btc_address '(record {target_chain_id = "eICP"; receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"})'
("bcrt1qhhnhv9azfxz8vm8csn3h8cz8yk9e2klrgvecc3")

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1qhhnhv9azfxz8vm8csn3h8cz8yk9e2klrgvecc3 100100FIRST•RUNE•TOKEN
{
  "txid": "3b979f01963051eb83624653f32a221990c30258a5b0e423c42f6d9cd9b5b9e6",
  "psbt": "cHNidP8BAOQCAAAAAmy0pehdbMysgYj4gXbtwERKLBzHRIWXAMYDJF4djzr4AQAAAAD/////JdBXhVYOYUQzYqqDZkm+uuqa4zLHjqvltmcsUcg68FABAAAAAP3///8EAAAAAAAAAAAKal0HAGwBqIw9AhAnAAAAAAAAIlEg1Uxia8bgne23npSjjcrcobnEbuEKpf3kyuRqc9UkaN0QJwAAAAAAABYAFL3ndheiSYR2bPiE43PgRyWLlVvjTXoFKgEAAAAiUSCYf3q4ECTW4LiXfil8ZIUbXZSyAdLYZsC+yhno9agZZAAAAAAAAQErECcAAAAAAAAiUSA0q/VBmX4DNCB9N4OFvWhgBa+KfztUn2RsgHalc0aDegEIQgFA3lMSgIRyPCBb+qGHDTqkN/h34nesK58JUVqMwffr39oiIb0LyhKEIcurnSj0nmG1BbyluG8qrm3SJwTqcP9yugABAStjogUqAQAAACJRIKTb7rcZy3VpINnYgfUcGEY3jWsJ3I4gqSol5iUyq5JTAQhCAUC96cgEMhoPx92yRrJsh4eH1esT10r33bAA80iErIP0WEXDHaCD7mNAo4brTHn7VebPgZXzjf1VFTIJnOHMB0qGAAABBSAHaEp/6mKNyUMzeNK6eWvaBbCk/0F03E73HJF5yCpaYCEHB2hKf+pijclDM3jSunlr2gWwpP9BdNxO9xyRecgqWmAZANg9g1pWAACAAQAAgAAAAIABAAAABAAAAAAAAQUgdzFTuRjJv46NZ1nJ8/75+Ct265XykpxUALc5aVMPIBMhB3cxU7kYyb+OjWdZyfP++fgrduuV8pKcVAC3OWlTDyATGQDYPYNaVgAAgAEAAIAAAACAAQAAAAUAAAAA",
  "outgoing": "100100 FIRST•RUNE•TOKEN",
  "fee": 262
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 6 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "eICP"; receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"; rune_id = "108:1"; amount = 1001000; txid = "3b979f01963051eb83624653f32a221990c30258a5b0e423c42f6d9cd9b5b9e6"})'

$ dfx canister call bitcoin_customs get_pending_gen_ticket_requests '(record {max_count = 3; start_txid = null})'
(
  vec {
    record {
      received_at = 1_711_769_987_045_738_209 : nat64;
      token_id = "Bitcoin-runes-FIRST•RUNE•TOKEN";
      txid = blob "\e6\b9\b5\d9\9c\6d\2f\c4\23\e4\b0\a5\58\02\c3\90\19\22\2a\f3\53\46\62\83\eb\51\30\96\01\9f\97\3b";
      target_chain_id = "eICP";
      address = "bcrt1qhhnhv9azfxz8vm8csn3h8cz8yk9e2klrgvecc3";
      amount = 1_001_000 : nat;
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
        1 : nat64;
        record {
          token = "Bitcoin-runes-FIRST•RUNE•TOKEN";
          action = variant { Transfer };
          dst_chain = "eICP";
          memo = null;
          ticket_id = "3b979f01963051eb83624653f32a221990c30258a5b0e423c42f6d9cd9b5b9e6";
          sender = null;
          ticket_time = 1_711_770_165_743_813_416 : nat64;
          src_chain = "Bitcoin";
          amount = "1001000";
          receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe";
        };
      };
    }
  },
)

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_balance_of "(record {owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\"; })"
(1_001_000 : nat)


$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc2_approve "(record { amount = 10010; spender = record{owner = principal \"br5f7-7uaaa-aaaaa-qaaca-cai\";} })"
(variant { Ok = 1 : nat })
$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc2_allowance "(record { account = record{owner = principal \"o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe\";}; spender = record{owner = principal \"br5f7-7uaaa-aaaaa-qaaca-cai\";} })"
(record { allowance = 10_010 : nat; expires_at = null })


$ dfx canister call icp_route generate_ticket '(record {target_chain_id = "Bitcoin"; receiver = "bcrt1pjr98ggycsnrkl555lg7z37pfyxwl62eseyms4qdpt8eer46nwkgslhamtu"; token_id = "Bitcoin-runes-FIRST•RUNE•TOKEN"; amount = 10010})'

$ dfx canister call omnity_hub query_tickets '(opt "Bitcoin", 0, 10)'
(
  variant {
    Ok = vec {
      record {
        2 : nat64;
        record {
          token = "Bitcoin-runes-FIRST•RUNE•TOKEN";
          action = variant { Redeem };
          dst_chain = "Bitcoin";
          memo = null;
          ticket_id = "3";
          sender = null;
          ticket_time = 1_711_770_906_257_507_303 : nat64;
          src_chain = "eICP";
          amount = "10010";
          receiver = "bcrt1pjr98ggycsnrkl555lg7z37pfyxwl62eseyms4qdpt8eer46nwkgslhamtu";
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
  "cardinal": 79999969357,
  "ordinal": 10000,
  "runes": {
    "FIRSTRUNETOKEN": 999002405
  },
  "runic": 10546,
  "total": 79999989903
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