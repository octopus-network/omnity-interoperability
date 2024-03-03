
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ dfx stop
$ dfx start --clean
$ dfx deploy omnity_hub --mode reinstall -y
$ dfx deploy bitcoin_customs --mode reinstall -y --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 1; max_time_in_queue_nanos = 600_000_000_000 } })' # 20 mins for testnet

# https://github.com/lesterli/ord/blob/docs/runes/docs/src/guides/runes.md
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet create
{
  "mnemonic": "little jaguar mix coral wool violin wink hip author stable elbow grit",
  "passphrase": ""
}

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet receive
{
  "address": "bcrt1ppna28kx85s7mca9djugzka3hns0heex5wny5zmjcmwfaav8azpjs03q85p"
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1ppna28kx85s7mca9djugzka3hns0heex5wny5zmjcmwfaav8azpjs03q85p

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes --index-transactions server --http --http-port 23456 --address 0.0.0.0
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 etch --divisibility 0 --fee-rate 1 --rune FIRSTRUNETOKEN --supply 21000000 --symbol $
{
  "rune": "FIRSTRUNETOKEN",
  "transaction": "52fa48b5d2c6862d3276cd58a4bd4fe53bfac0117c96b726c992ced7f037b686"
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 9999989805,
  "ordinal": 0,
  "runes": {
    "FIRSTRUNETOKEN": 21000000
  },
  "runic": 10000,
  "total": 9999999805
}
$ dfx canister call bitcoin_customs get_btc_address '(record {target_chain_id = "cosmoshub"; receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"})'
("bcrt1qxvl4n26ktl7qy3ttvqh7fr33qvgdwjuf59s2k6")
$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1qxvl4n26ktl7qy3ttvqh7fr33qvgdwjuf59s2k6
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1qxvl4n26ktl7qy3ttvqh7fr33qvgdwjuf59s2k6 7FIRSTRUNETOKEN
{
  "txid": "2da1447674d97922dbe2e1d14cdbdb07b42675e97eb810fb90d3d776cf0ace46",
  "psbt": "cHNidP8BAO0CAAAAAoa2N/DXzpLJJreWfBHA+jvlT72kWM12Mi2GxtK1SPpSAQAAAAD/////hrY38NfOkskmt5Z8EcD6O+VPvaRYzXYyLYbG0rVI+lICAAAAAP3///8EAAAAAAAAAAATaglSVU5FX1RFU1QHAIKW/wEHAhAnAAAAAAAAIlEg6HNX5Mrk1uj/V98tP+ila4JZDTy+7ZbP6ORG83NFAvoQJwAAAAAAABYAFDM/WatWX/wCRWtgL+SOMQMQ10uJDqIFKgEAAAAiUSBOHBWsntVh8UKPo9uKn0TBahuVWLoBA+iB+tXzx7OZzwAAAAAAAQErECcAAAAAAAAiUSCY4S7y3+ysjSW2/rgS4RF9SpmsicKhS0siOFhRI9oK3wEIQgFAfOpe7mIuwznjKzWhJ1Ezl4u85P0x7FK9Dc5yqmfGVB1XNyNRLY560IV2Jdfg705SDJMNe2883snEGOVFPHAQmQABASstygUqAQAAACJRIBmlSgJpw+eDaeAi2R90sCOIlnEgdG7DnRFvRuCq2QkLAQhCAUA8nvKeAL4rDp7Mhathr/pNvzVeM3VjP7Z9ilEPG6sep5K99mYNQjdOCgYKBSDci9U7b5JiPYGbLSqM+Hjr2yOcAAABBSCDDsEeI4W/FElO3bKkN6Zf4zSIj4ArvHeBtqXMnyJIQSEHgw7BHiOFvxRJTt2ypDemX+M0iI+AK7x3gbalzJ8iSEEZAN9vTNJWAACAAQAAgAAAAIABAAAAAgAAAAAAAQUgpROdi2vyhFCiUBlzhtxn22PBM+6hl+5GE1fcfnYC4vEhB6UTnYtr8oRQolAZc4bcZ9tjwTPuoZfuRhNX3H52AuLxGQDfb0zSVgAAgAEAAIAAAACAAQAAAAMAAAAA",
  "outgoing": "7 FIRSTRUNETOKEN",
  "fee": 271
}
$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
# http://192.168.1.105:23456/api/tx/2da1447674d97922dbe2e1d14cdbdb07b42675e97eb810fb90d3d776cf0ace46
$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "cosmoshub"; receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"; rune_id = 6684673; amount = 7; txid = "2da1447674d97922dbe2e1d14cdbdb07b42675e97eb810fb90d3d776cf0ace46"})'
$ dfx canister call bitcoin_customs get_pending_gen_ticket_requests
(
  vec {
    record {
      received_at = 1_709_450_514_381_646_755 : nat64;
      txid = blob "\46\ce\0a\cf\76\d7\d3\90\fb\10\b8\7e\e9\75\26\b4\07\db\db\4c\d1\e1\e2\db\22\79\d9\74\76\44\a1\2d";
      target_chain_id = "cosmoshub";
      address = "bcrt1qxvl4n26ktl7qy3ttvqh7fr33qvgdwjuf59s2k6";
      amount = 7 : nat;
      receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur";
      rune_id = 6_684_673 : nat;
    };
  },
)
$ dfx canister call omnity_hub build_directive '(variant { AddChain = record { chain_state=variant { Active };chain_name = "BTC"; chain_type=variant { SettlementChain };}})'
$ dfx canister call omnity_hub set_whitelist '(principal "be2us-64aaa-aaaaa-qaabq-cai", true)'
$ ./target/debug/runes_oracle
$ dfx canister call omnity_hub query_tickets '("cosmoshub", 0, 10)'
