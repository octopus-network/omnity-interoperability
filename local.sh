
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ dfx stop
$ dfx start --clean
$ dfx deploy omnity_hub --mode reinstall -y
$ dfx identity --identity default get-principal
o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe
$ dfx deploy bitcoin_customs --mode reinstall -y --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 1; max_time_in_queue_nanos = 600_000_000_000; runes_oracle_principal = principal "o3dmw-dhvlv-7rh3g-eput4-g2pxm-linuy-4yh7a-n2pd4-7lhgk-4c4aq-bqe" } })' # 20 mins for testnet

# https://github.com/lesterli/ord/blob/docs/runes/docs/src/guides/runes.md
$ git clone https://github.com/octopus-network/ord.git
$ git checkout runescan
$ sudo docker run --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=mysecretpassword -v ~/dev/data:/var/lib/postgresql/data -d postgres:12
$ sudo docker run -it --rm --network host postgres:12 psql -h 127.0.0.1 -U postgres
postgres=# CREATE DATABASE runescan ENCODING = 'UTF8';
$ sudo docker exec -i postgres psql -U postgres -d runescan < deploy/runescan.sql
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet create
{
  "mnemonic": "utility organ bamboo cause venture tackle reunion else mass wing clump ill",
  "passphrase": ""
}

$ export DATABASE_URL=postgres://postgres:mysecretpassword@127.0.0.1:5432/runescan
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "address": "bcrt1pcaplepck2cu4a457xn2wep2k26hzasyv5nuusd8xf6mrwvnt0fuqg8md8c"
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1pcaplepck2cu4a457xn2wep2k26hzasyv5nuusd8xf6mrwvnt0fuqg8md8c

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 5000000000,
  "ordinal": 0,
  "runes": {},
  "runic": 0,
  "total": 5000000000
}
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 etch --divisibility 0 --fee-rate 1 --rune_id FIRST•RUNE•TOKEN --supply 21000000 --symbol $
{
  "rune": "FIRST•RUNE•TOKEN",
  "transaction": "6027783e5693af7feba3797a2568a3d484a498edb2499ad9455d0cfb088d5bdc"
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 9999989800,
  "ordinal": 0,
  "runes": {
    "FIRSTRUNETOKEN": 21000000
  },
  "runic": 10000,
  "total": 9999999800
}

$ dfx canister call bitcoin_customs get_btc_address '(record {target_chain_id = "cosmoshub"; receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"})'
("bcrt1qnwc03kekz4zexmtd69fffy6ap6pl3x4xwagdqf")
$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1qnwc03kekz4zexmtd69fffy6ap6pl3x4xwagdqf

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1qnwc03kekz4zexmtd69fffy6ap6pl3x4xwagdqf 7FIRST•RUNE•TOKEN
{
  "txid": "4058fd4afb991a89d09dff3d79abe6abecfdb2f90ae81314e357ca2a5b052a4a",
  "psbt": "cHNidP8BAO0CAAAAAtxbjQj7DF1F2ZpJsu2YpITUo2glenmj63+vk1Y+eCdgAQAAAAD/////3FuNCPsMXUXZmkmy7ZikhNSjaCV6eaPrf6+TVj54J2ACAAAAAP3///8EAAAAAAAAAAATaglSVU5FX1RFU1QHAIKW/wEHAhAnAAAAAAAAIlEgSDdo9VBWyHqvwks0zGlxavcaCHf/ZQtQ3O87BvzRI6EQJwAAAAAAABYAFJuw+Ns2FUWTbW3RUpSTXQ6D+JqmCaIFKgEAAAAiUSB2vRW+y2NnA+jIRfKVUkKps9y66q7QzGiWUlWr551dCAAAAAAAAQErECcAAAAAAAAiUSAY5IWZlU5/3gu7CK6fWY6IXZ/6jYAxvjArdxGBqtPogwEIQgFAs+4X48D66XGDpVUa7wsaQgDfQubACDpiwwCpEg/4ueJ/cng4P4DSUKkcJSw45gSQS2alrPbkzHAecsniQOBz1gABASsoygUqAQAAACJRII4DO6tGUYCd+a/gfY0A0C8V4+riU1pmZDv0nqqyhyLJAQhCAUAfDfVccye8RvtoYE9+D7zV1gRTSN+hJVIz1P1aeYoT9JF1VgLGi7NR47v+yHw+EeImUhEjkxKaA1SYbMD21XFtAAABBSAZ5AeHu7EUPOKclBd3DBhhXFFmqPBE/RTmZoWW3JXK+CEHGeQHh7uxFDzinJQXdwwYYVxRZqjwRP0U5maFltyVyvgZADGU4OBWAACAAQAAgAAAAIABAAAAAgAAAAAAAQUg0LE8CglocrrEDjE22qsPi05LHWeXb03MWgU31jS7Nr0hB9CxPAoJaHK6xA4xNtqrD4tOSx1nl29NzFoFN9Y0uza9GQAxlODgVgAAgAEAAIAAAACAAQAAAAMAAAAA",
  "outgoing": "7 FIRST•RUNE•TOKEN",
  "fee": 271
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
http://192.168.1.105:23456/rune/FIRST%E2%80%A2RUNE%E2%80%A2TOKEN
rune_id: 102:1
$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "cosmoshub"; receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"; rune_id = record {height = 102; index = 1;}; amount = 7; txid = "4058fd4afb991a89d09dff3d79abe6abecfdb2f90ae81314e357ca2a5b052a4a"})'
$ dfx canister call bitcoin_customs get_pending_gen_ticket_requests
(
  vec {
    record {
      received_at = 1_709_996_895_574_557_125 : nat64;
      txid = blob "\4a\2a\05\5b\2a\ca\57\e3\14\13\e8\0a\f9\b2\fd\ec\ab\e6\ab\79\3d\ff\9d\d0\89\1a\99\fb\4a\fd\58\40";
      target_chain_id = "cosmoshub";
      address = "bcrt1qnwc03kekz4zexmtd69fffy6ap6pl3x4xwagdqf";
      amount = 7 : nat;
      receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur";
      rune_id = "102:1";
    };
  },
)



dfx canister call omnity_hub validate_proposal '(variant { AddChain = record { chain_state=variant { Active };chain_id = "BTC"; chain_type=variant { SettlementChain };}})'
dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_id = "BTC"; chain_type=variant { SettlementChain };}})'
dfx canister call omnity_hub validate_proposal '(variant { AddToken = record { decimals = 0 : nat8; icon = opt "rune"; token_id = "102:1"; issue_chain = "BTC"; symbol = "FIRST•RUNE•TOKEN";}})'
dfx canister call omnity_hub build_directive '(variant { AddToken = record { decimals = 0 : nat8; icon = opt "rune"; token_id = "102:1"; issue_chain = "BTC"; symbol = "FIRST•RUNE•TOKEN";}})'
dfx canister call omnity_hub query_directive_with_topic '("Ethereum",opt variant {AddToken=null},0:nat64,5:nat64)' 


$ dfx canister call omnity_hub set_whitelist '(principal "be2us-64aaa-aaaaa-qaabq-cai", true)'
$ ./target/debug/runes_oracle
$ dfx canister call omnity_hub query_tickets '("cosmoshub", 0, 10)'

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
