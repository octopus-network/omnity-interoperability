
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
  "mnemonic": "chimney zone resource coast vibrant prevent immense under void fever treat enlist",
  "passphrase": ""
}

$ rm -rf ~/.local/share/ord/regtest/index.redb
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "addresses": [
    "bcrt1pyudjvmh9etjmzvwrj2cru73ezczx46zc2k09dmlwk63xdl0xn7qqqvrkdt"
  ]
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1pyudjvmh9etjmzvwrj2cru73ezczx46zc2k09dmlwk63xdl0xn7qqqvrkdt

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
  "commit": "05764d8e9c7ee0b3c3b856c503c88966c25e82ec2a1ca09f8a48377f4d4f123c",
  "commit_psbt": null,
  "inscriptions": [
    {
      "destination": "bcrt1pz9x4et39sphlawapsjrtpqfwpx857hdpp0t5cg4dvtuejf4etvwqs7plyx",
      "id": "e9e20190441ad8b3159746ed396014947aa613f4d116e8382ca68362c7de1f7ai0",
      "location": "e9e20190441ad8b3159746ed396014947aa613f4d116e8382ca68362c7de1f7a:0:0"
    }
  ],
  "parent": null,
  "reveal": "e9e20190441ad8b3159746ed396014947aa613f4d116e8382ca68362c7de1f7a",
  "reveal_psbt": null,
  "rune": {
    "destination": "bcrt1pyvs3ehahscsa63flsnkp8058npx9uxppgeheas7rukppqnlncurqwjyt53",
    "location": "e9e20190441ad8b3159746ed396014947aa613f4d116e8382ca68362c7de1f7a:1",
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
("bcrt1qngm7ucr7pjpa9fx2gaa0kyj28ljem9cq49askw")

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1qngm7ucr7pjpa9fx2gaa0kyj28ljem9cq49askw 7FIRST•RUNE•TOKEN
{
  "txid": "b30dd5b17d0393a7c4a1e37baa6ce1ef385ca6ab8b537bf4fed9fe37ab151bdf",
  "psbt": "cHNidP8BAOICAAAAAnof3sdig6YsOOgW0fQTpnqUFGA57UaXFbPYGkSQAeLpAQAAAAD/////PBJPTX83SIqfoBwq7IJewmaJyAPFVrjDs+B+nI5NdgUBAAAAAP3///8EAAAAAAAAAAAIal0FAGwBRgIQJwAAAAAAACJRICsgBYjTHUn6Pvpsz4dwX9VYHs8zS+PGxo+3EcWzCvy9ECcAAAAAAAAWABSaN+5gfgyD0qTKR3r7Eko/5Z2XAFJ6BSoBAAAAIlEgdgKt2XfHDd3BfHEcZqaifAC3JsPi3CcUD5bpikYMa+YAAAAAAAEBKxAnAAAAAAAAIlEgIyEc37eGId1FP4TsE76HmExeGCFGb57Dw+WCEE/zxwYBCEIBQEVsHfG23IWjvvXthibOK9HWYvZOqSYLnL0k7Y+rXbfn0d77QAmPkYOzzXLCN7q56XUkjrCI2SRR0rbO6YOSVqwAAQErZqIFKgEAAAAiUSDBQ2HVl/EhbptNG1CjLRbc2yKeenfALe28/hFyi+I2ZQEIQgFARws1VXIQ2cIdGSJWvfrmi8za5/Uam1ufJHx2h7B2G8wq1wRfW7vQNWUvtBTEKfe2vabI1CcwzqxFI2pxhdUdcAAAAQUgJelzn95vpPkr+CNOuScqjDy0XCtp4xD3vBeJIhDUu9chByXpc5/eb6T5K/gjTrknKow8tFwraeMQ97wXiSIQ1LvXGQDEvgd/VgAAgAEAAIAAAACAAQAAAAQAAAAAAAEFIJIIO4thI3YXRMVQpIDM0zQjAl+wWiqKxG3wtc1fIQ1WIQeSCDuLYSN2F0TFUKSAzNM0IwJfsFoqisRt8LXNXyENVhkAxL4Hf1YAAIABAACAAAAAgAEAAAAFAAAAAA==",
  "outgoing": "7 FIRST•RUNE•TOKEN",
  "fee": 260
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 6 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "eICP"; receiver = "glfzm-3xumc-23ch4-znudm-hs76m-ffyre-k7yxu-ct342-2emb7-lm4wa-3qe"; rune_id = "108:1"; amount = 70; txid = "b30dd5b17d0393a7c4a1e37baa6ce1ef385ca6ab8b537bf4fed9fe37ab151bdf"})'

$ dfx canister call bitcoin_customs get_pending_gen_ticket_requests '(record {max_count = 3; start_txid = null})'
(
  vec {
    record {
      received_at = 1_711_541_850_234_752_593 : nat64;
      token_id = "Bitcoin-runes-FIRST•RUNE•TOKEN";
      txid = blob "\df\1b\15\ab\37\fe\d9\fe\f4\7b\53\8b\ab\a6\5c\38\ef\e1\6c\aa\7b\e3\a1\c4\a7\93\03\7d\b1\d5\0d\b3";
      target_chain_id = "eICP";
      address = "bcrt1qngm7ucr7pjpa9fx2gaa0kyj28ljem9cq49askw";
      amount = 70 : nat;
      receiver = "glfzm-3xumc-23ch4-znudm-hs76m-ffyre-k7yxu-ct342-2emb7-lm4wa-3qe";
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
          token = "Bitcoin-runes-FIRST•RUNE•TOKEN";
          action = variant { Transfer };
          dst_chain = "eICP";
          memo = null;
          ticket_id = "b30dd5b17d0393a7c4a1e37baa6ce1ef385ca6ab8b537bf4fed9fe37ab151bdf";
          sender = null;
          ticket_time = 1_711_542_455_514_000_757 : nat64;
          src_chain = "Bitcoin";
          amount = "70";
          receiver = "glfzm-3xumc-23ch4-znudm-hs76m-ffyre-k7yxu-ct342-2emb7-lm4wa-3qe";
        };
      };
    }
  },
)



$ dfx canister call omnity_hub send_ticket '(record { ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21"; ticket_time = 1707291817947 : nat64; token = "Bitcoin-runes-FIRST•RUNE•TOKEN"; amount = "10"; src_chain = "eICP"; dst_chain = "Bitcoin"; action = variant { Redeem }; sender = opt "glfzm-3xumc-23ch4-znudm-hs76m-ffyre-k7yxu-ct342-2emb7-lm4wa-3qe"; receiver = "bcrt1pyudjvmh9etjmzvwrj2cru73ezczx46zc2k09dmlwk63xdl0xn7qqqvrkdt"; memo = null;})'
$ dfx canister call omnity_hub query_tickets '(opt "Bitcoin", 0, 10)'
(
  variant {
    Ok = vec {
      record {
        0 : nat64;
        record {
          token = "Bitcoin-runes-FIRST•RUNE•TOKEN";
          action = variant { Redeem };
          dst_chain = "Bitcoin";
          memo = null;
          ticket_id = "f8aee1cc-db7a-40ea-80c2-4cf5e6c84c21";
          sender = opt "glfzm-3xumc-23ch4-znudm-hs76m-ffyre-k7yxu-ct342-2emb7-lm4wa-3qe";
          ticket_time = 1_707_291_817_947 : nat64;
          src_chain = "eICP";
          amount = "10";
          receiver = "bcrt1pyudjvmh9etjmzvwrj2cru73ezczx46zc2k09dmlwk63xdl0xn7qqqvrkdt";
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
  "cardinal": 74999969362,
  "ordinal": 10000,
  "runes": {
    "FIRSTRUNETOKEN": 9940
  },
  "runic": 10546,
  "total": 74999989908
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
