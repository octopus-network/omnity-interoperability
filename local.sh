
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ dfx stop
$ dfx start --clean
$ dfx deploy omnity_hub
$ dfx identity --identity default get-principal
o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe
$ dfx deploy bitcoin_customs --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 1; max_time_in_queue_nanos = 600_000_000_000; runes_oracle_principal = principal "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe" } })' # 20 mins for testnet

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
  "mnemonic": "purpose convince install street vocal garden blast design stumble siege position sort",
  "passphrase": ""
}

$ rm -rf ~/.local/share/ord/regtest/index.redb
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "address": "bcrt1p2j485ny26ywyzp2n62rac5mgs8xjjry3cv57et9uafatvv33wjzq5rhq05"
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1p2j485ny26ywyzp2n62rac5mgs8xjjry3cv57et9uafatvv33wjzq5rhq05

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 5000000000,
  "ordinal": 0,
  "runes": {},
  "runic": 0,
  "total": 5000000000
}
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 etch --rune FIRST•RUNE•TOKEN --divisibility 1 --fee-rate 1 --supply 1000 --symbol ¢
{
  "rune": "FIRST•RUNE•TOKEN",
  "transaction": "e57fc5f5e45e266da5cc80462dc07c5d1b41518b7b1eeeb1e20d2cf1c90ec12e"
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 9999989799,
  "ordinal": 0,
  "runes": {
    "FIRSTRUNETOKEN": 10000
  },
  "runic": 10000,
  "total": 9999999799
}

$ dfx canister call bitcoin_customs get_btc_address '(record {target_chain_id = "cosmoshub"; receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"})'
("bcrt1q9jvz3tkk0nptsx8tw8chvjz03h77fvf8dy66z2")

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1q9jvz3tkk0nptsx8tw8chvjz03h77fvf8dy66z2

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1q9jvz3tkk0nptsx8tw8chvjz03h77fvf8dy66z2 7FIRST•RUNE•TOKEN
{
  "txid": "c6e42f634765f8cedd772bbf1f00a3716a42c35c5b4fff7af78f05485ba49da4",
  "psbt": "cHNidP8BAO0CAAAAAi7BDsnxLA3ise4ee4tRQRtdfMAtRoDMpW0mXuT1xX/lAQAAAAD/////LsEOyfEsDeKx7h57i1FBG118wC1GgMylbSZe5PXFf+UCAAAAAP3///8EAAAAAAAAAAATaglSVU5FX1RFU1QHAIKW/wFGAhAnAAAAAAAAIlEgyqyXRbIrGCOdLo5iNFIg9MyOJPBT9O0IG/mhBcxF894QJwAAAAAAABYAFCyYKK7WfMK4GOtx8XZIT4395LEnCKIFKgEAAAAiUSBidxiEVosIupXWYW60zVYrkwD0Fzgux2iBYg1ra/yL+AAAAAAAAQErECcAAAAAAAAiUSA6A9QWwjOxC7DfGOKchUaK7kFxdqIhd11u0G5fP9zvxgEIQgFAoRvwcQ88f2a8+rk0zg0avE+g+FFms+tkgYgKABnNwwiSXUcMD8mCDycfeIXU3k3CJ5oV9tZJk098ZqaI3eudpgABASsnygUqAQAAACJRIDFLKIaHIdWHdQ5LngRe9SVwjhcY+F/jiD0N89AnQciUAQhCAUCMRDKQ7hlEe4dzpQto0hKTNjoYh0HUW/J/IO7dda5HWuYWqy94R7HjwDqITz4LBdwevw0YyhSvWrMGjDdvohYFAAABBSDYSgcX8vD+L6ILCZPqGtHQWPCJioHuQrtFcn82P5L1pSEH2EoHF/Lw/i+iCwmT6hrR0FjwiYqB7kK7RXJ/Nj+S9aUZANpgCkhWAACAAQAAgAAAAIABAAAAAgAAAAAAAQUgLK6r9WyreCLzLkl/NvwmAB0UxEl03xOniFRE9anJxlchByyuq/Vsq3gi8y5Jfzb8JgAdFMRJdN8Tp4hURPWpycZXGQDaYApIVgAAgAEAAIAAAACAAQAAAAMAAAAA",
  "outgoing": "7 FIRST•RUNE•TOKEN",
  "fee": 271
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
http://192.168.1.105:23456/rune/FIRST%E2%80%A2RUNE%E2%80%A2TOKEN
rune_id: 102:1
$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "cosmoshub"; receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"; rune_id = "102:1"; amount = 70; txid = "c6e42f634765f8cedd772bbf1f00a3716a42c35c5b4fff7af78f05485ba49da4"})'
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

$ dfx canister call omnity_hub validate_proposal '(variant { AddChain = record { chain_state=variant { Active };chain_id = "BTC"; chain_type=variant { SettlementChain };}})'
$ dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_id = "BTC"; chain_type=variant { SettlementChain };}})'

$ dfx canister call omnity_hub validate_proposal '(variant { AddChain = record { chain_state=variant { Active };chain_id = "cosmoshub"; chain_type=variant { ExecutionChain };}})'
$ dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_id = "cosmoshub"; chain_type=variant { ExecutionChain };}})'

$ dfx canister call omnity_hub validate_proposal '(variant { AddToken = record { decimals = 1 : nat8; icon = opt "rune"; token_id = "102:1"; issue_chain = "BTC"; symbol = "FIRST•RUNE•TOKEN";}})'
$ dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 1 : nat8; icon = opt "rune"; token_id = "102:1"; issue_chain = "BTC"; symbol = "FIRST•RUNE•TOKEN";}})'
$ dfx canister call omnity_hub query_directives '("BTC",opt variant {AddToken=null},0:nat64,5:nat64)' 
(
  variant {
    Ok = vec {
      record {
        0 : nat64;
        variant {
          AddToken = record {
            decimals = 1 : nat8;
            token_id = "102:1";
            icon = opt "rune";
            issue_chain = "BTC";
            symbol = "FIRST•RUNE•TOKEN";
          }
        };
      };
    }
  },
)


$ dfx canister call omnity_hub set_whitelist '(principal "be2us-64aaa-aaaaa-qaabq-cai", true)'
$ RUST_LOG=info ./target/debug/runes_oracle
$ dfx canister call omnity_hub query_tickets '("cosmoshub", 0, 10)'
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
