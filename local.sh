
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
$ git checkout dev
$ sudo docker run --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=mysecretpassword -v ~/dev/data:/var/lib/postgresql/data -d postgres:12
$ sudo docker run -it --rm --network host postgres:12 psql -h 127.0.0.1 -U postgres
postgres=# CREATE DATABASE runescan ENCODING = 'UTF8';
$ sudo docker exec -i postgres psql -U postgres -d runescan < deploy/runescan.sql
$ export DATABASE_URL=postgres://postgres:mysecretpassword@127.0.0.1:5432/runescan
$ cargo build
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet create
{
  "mnemonic": "reward equip add inner cash vivid certain table juice smile thing ride",
  "passphrase": ""
}

$ rm -rf ~/.local/share/ord/regtest/index.redb
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "addresses": [
    "bcrt1ppxr2qnx3kcaccwwa0smruzfr6w9qpa58yx3fke4hacx4mj44w0tq4zmnae"
  ]
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1ppxr2qnx3kcaccwwa0smruzfr6w9qpa58yx3fke4hacx4mj44w0tq4zmnae

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
Waiting for rune commitment fe235de077f0c3150580e55bc9e6a31951e26fc5204ffe27dd8f4ecf0cb8fbb6 to mature…

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 6 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

{
  "commit": "fe235de077f0c3150580e55bc9e6a31951e26fc5204ffe27dd8f4ecf0cb8fbb6",
  "commit_psbt": null,
  "inscriptions": [
    {
      "destination": "bcrt1pzgpzufzyfslt7s9uygqklamq9t5s3363dru7mytpvnry8msmhugqhtjchh",
      "id": "9a4cca8706a3b7e7960dada3d9a9597f82a2bf31a9a8916bc4634a615f389b55i0",
      "location": "9a4cca8706a3b7e7960dada3d9a9597f82a2bf31a9a8916bc4634a615f389b55:0:0"
    }
  ],
  "parent": null,
  "reveal": "9a4cca8706a3b7e7960dada3d9a9597f82a2bf31a9a8916bc4634a615f389b55",
  "reveal_broadcast": true,
  "reveal_psbt": null,
  "rune": {
    "destination": "bcrt1pgyplwd4sea9kee9vdz9gc95evswxxmuwfpv67g64h53g5jvyt39supadnx",
    "location": "9a4cca8706a3b7e7960dada3d9a9597f82a2bf31a9a8916bc4634a615f389b55:1",
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
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}; fee_token="BTC"}}}})'
$ dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"eICP"}; fee_token= "BTC"}}}})'

# Note: replace the canister id to ICP route canister id and constract address
$ dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="br5f7-7uaaa-aaaaa-qaaca-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}; fee_token="ICP"}}}})'
$ dfx canister call omnity_hub execute_proposal  '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="br5f7-7uaaa-aaaaa-qaaca-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}; fee_token="ICP"}}}})'

$ dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 2 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; settlement_chain = "Bitcoin"; symbol = "UNCOMMON•GOODS"; metadata = opt vec{ record {"rune_id"; "108:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'
$ dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 2 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; settlement_chain = "Bitcoin"; symbol = "UNCOMMON•GOODS"; metadata = opt vec{ record {"rune_id"; "108:1"}}; dst_chains = vec {"Bitcoin";"eICP";}}}})'

# update fee
dfx canister call omnity_hub update_fee 'vec {variant {ChainFactor = record {chain_id="Bitcoin"; chain_factor=1000 : nat}}; variant {TokenFactor = record {dst_chain_id="Bitcoin"; fee_token="ICP"; fee_token_factor=60000000000 : nat}}}'
# query update fee directive
dfx canister call omnity_hub query_directives '(opt "ICP",opt variant {UpdateFee=opt "ICP"},0:nat64,5:nat64)' 


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
            decimals = 2 : nat8;
            token_id = "Bitcoin-runes-UNCOMMON•GOODS";
            metadata = opt vec { record { "rune_id"; "108:1" } };
            icon = opt "rune.logo.url";
            issue_chain = "Bitcoin";
            symbol = "UNCOMMON•GOODS";
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
("bcrt1qp5ezzetuwc4jtzjfc9w2t47n7yvhgl4xz842pf")

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1qp5ezzetuwc4jtzjfc9w2t47n7yvhgl4xz842pf 7:UNCOMMON•GOODS
{
  "txid": "dee5e4547f26242cc4ddeae9dca68708d766efd5f7065fbf3b19c7d39a35b442",
  "psbt": "cHNidP8BAOMCAAAAAlWbOF9hSmPEa5GoqTG/ooJ/WanZo60Nlue3owaHykyaAQAAAAD/////tvu4DM9Oj90n/k8gxW/iURmj5slb5YAFFcPwd+BdI/4BAAAAAP3///8EAAAAAAAAAAAJal0GAGwBvAUCECcAAAAAAAAiUSC11hpe+hmZMfgS5CLxbp+jICMDZ2DaSzzIeWUTNPxAChAnAAAAAAAAFgAUDTIhZXx2KyWKScFcpdfT8Rl0fqZAegUqAQAAACJRIPU3h5vCzipp4OzIkGLnWbctyQkiXPMK00O9yfcB12+FAAAAAAABASsQJwAAAAAAACJRIEED9zawz0ts5KxoiowWmWQcY2+OSFmvI1W9IopJhFxLAQhCAUAuziQMPwv3uo47P0B4meZyqYs2b3aWN5q+oAAUgySNYpSLnAHXFGZdPnbxkwi8GCdBpA1ZghckH5QtElwZFfsKAAEBK1WiBSoBAAAAIlEg2u6EwK+2RKcbeRhXQGnVHp4A8oWXJ+0NoiHS706vWEwBCEIBQHXKZ3PPSYR9u0iq0G+J6F91IJFoc0bTa7ti4utvCW3Exh0o5ApdXUX1Y6Zmk1udiTCytJCejYJgF2tkqUCf/4sAAAEFICMUNzx2r3Sm6Bi1LNrWf8MJ8kwpqpkDrpqI88tRLDwjIQcjFDc8dq90pugYtSza1n/DCfJMKaqZA66aiPPLUSw8IxkArnKnfFYAAIABAACAAAAAgAEAAAAEAAAAAAABBSAbUBc8MlCZrml2T7gGJOMv1PpmZ3NoLZstHBgzITpkQCEHG1AXPDJQma5pdk+4BiTjL9T6ZmdzaC2bLRwYMyE6ZEAZAK5yp3xWAACAAQAAgAAAAIABAAAABQAAAAA=",
  "outgoing": "7:UNCOMMON•GOODS",
  "fee": 261
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 6 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "eICP"; receiver = "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe"; rune_id = "108:1"; amount = 700; txid = "dee5e4547f26242cc4ddeae9dca68708d766efd5f7065fbf3b19c7d39a35b442"})'

$ dfx canister call bitcoin_customs get_pending_gen_ticket_requests '(record {max_count = 3; start_txid = null})'
(
  vec {
    record {
      received_at = 1_712_063_290_323_408_781 : nat64;
      token_id = "Bitcoin-runes-UNCOMMON•GOODS";
      txid = blob "\42\b4\35\9a\d3\c7\19\3b\bf\5f\06\f7\d5\ef\66\d7\08\87\a6\dc\e9\ea\dd\c4\2c\24\26\7f\54\e4\e5\de";
      target_chain_id = "eICP";
      address = "bcrt1qp5ezzetuwc4jtzjfc9w2t47n7yvhgl4xz842pf";
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
        1 : nat64;
        record {
          token = "Bitcoin-runes-UNCOMMON•GOODS";
          action = variant { Transfer };
          dst_chain = "eICP";
          memo = null;
          ticket_id = "dee5e4547f26242cc4ddeae9dca68708d766efd5f7065fbf3b19c7d39a35b442";
          sender = null;
          ticket_time = 1_712_064_888_559_826_494 : nat64;
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


$ dfx canister call icp_route generate_ticket '(record {target_chain_id = "Bitcoin"; receiver = "bcrt1ppxr2qnx3kcaccwwa0smruzfr6w9qpa58yx3fke4hacx4mj44w0tq4zmnae"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; amount = 100})'

$ dfx canister call omnity_hub query_tickets '(opt "Bitcoin", 0, 10)'
(
  variant {
    Ok = vec {
      record {
        2 : nat64;
        record {
          token = "Bitcoin-runes-UNCOMMON•GOODS";
          action = variant { Redeem };
          dst_chain = "Bitcoin";
          memo = null;
          ticket_id = "2";
          sender = null;
          ticket_time = 1_712_066_075_159_097_473 : nat64;
          src_chain = "eICP";
          amount = "100";
          receiver = "bcrt1ppxr2qnx3kcaccwwa0smruzfr6w9qpa58yx3fke4hacx4mj44w0tq4zmnae";
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