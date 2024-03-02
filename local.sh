
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ dfx stop
$ dfx start --clean
$ dfx deploy omnity_hub --mode reinstall -y
dfx deploy bitcoin_customs --mode reinstall -y --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 12; max_time_in_queue_nanos = 600_000_000_000 } })' # 20 mins for testnet
$ dfx canister call bitcoin_customs get_btc_address '(record {target_chain_id = "cosmoshub"; receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"})'
("bcrt1qgcjur89fe4xehrpdl5cy757jf2n0jarkxyzrqs")
$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1qgcjur89fe4xehrpdl5cy757jf2n0jarkxyzrqs


# https://github.com/lesterli/ord/blob/docs/runes/docs/src/guides/runes.md
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet create
{
  "mnemonic": "pulp tongue search unlock hover alien cable dial you target canoe exchange",
  "passphrase": ""
}

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet receive
{
  "address": "bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww"
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes --index-transactions server --http --http-port 23456 --address 0.0.0.0
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 etch --divisibility 0 --fee-rate 1 --rune FIRST.RUNE.TOKEN --supply 21000000 --symbol $
{
  "rune": "FIRST•RUNE•TOKEN",
  "transaction": "c829a7b5f594b0cc9acc16ca6b5c0879d5ca30bff45dde2da8becb301cef9048"
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 747499990000,
  "ordinal": 0,
  "runes": {
    "FIRSTRUNETOKEN": 21000000
  },
  "runic": 10000,
  "total": 747500000000
}
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-user ic-btc-integration --bitcoin-rpc-pass QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1qgcjur89fe4xehrpdl5cy757jf2n0jarkxyzrqs 7FIRST•RUNE•TOKEN
{
  "txid": "ff8323408864fe8d7c46078edfe72a829a94064eed08f9728a759f27172df6f6",
  "psbt": "cHNidP8BAO0CAAAAAkiQ7xwwy76oLd5d9L8wytV5CFxryhbMmsywlPW1pynIAQAAAAD/////SJDvHDDLvqgt3l30vzDK1XkIXGvKFsyazLCU9bWnKcgCAAAAAP3///8EAAAAAAAAAAATaglSVU5FX1RFU1QHAIvS/wEHAhAnAAAAAAAAIlEgNPVzAi06yNBcv1J/jaoVV0xBIJeCXmRFUocYvUOM6AoQJwAAAAAAABYAFEYlwZypzU2bjC39ME9T0kqm+XR2iSyBSgAAAAAiUSDgxs0MmcU+gpdfP5Hav/QbBYMSur7I4jwYgefaKxPz0gAAAAAAAQErECcAAAAAAAAiUSByJ2Ek5AU83dR8cDmptVhuQfadlUciLnz3iJImHZxvxgEIQgFAy0skiGYfYHPVmEubJJGYGfSQDIbemJrudMcDBh0Hx0QDxdkHZF6oAciAabNPu/obMGYV1T3ZPAkPV/sZbPoyUgABASuoVIFKAAAAACJRIACLjIe+RhbBR6VbYoQC1UMc68fxmON2qwW0YR6JIV1eAQhCAUCBS0WZdiLiZMNzedm/wYQ99RkowS/SbDtr9I4AZJDEXw+aXyH89ctdp8pA1hng1EjyUjwTfBizlRY10N6qY+4DAAABBSDLCSZ4ccaGWkXODwO1FpCwWtrWgG1rx/e4SLllAl7IlSEHywkmeHHGhlpFzg8DtRaQsFra1oBta8f3uEi5ZQJeyJUZAPI8NSBWAACAAQAAgAAAAIABAAAABAAAAAAAAQUgtZVKOoRGM4GKSR93Qjg8ma/6ym313YH0+CYLQLph2YUhB7WVSjqERjOBikkfd0I4PJmv+spt9d2B9PgmC0C6YdmFGQDyPDUgVgAAgAEAAIAAAACAAQAAAAUAAAAA",
  "outgoing": "7 FIRST•RUNE•TOKEN",
  "fee": 271
}
$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
http://192.168.1.105:23456/tx/ff8323408864fe8d7c46078edfe72a829a94064eed08f9728a759f27172df6f6
$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "cosmoshub"; receiver = "cosmos1kwf682z5rxj38jsemljvdh67ykswns77j3euur"; rune_id = "123"; amount = 7; txid = "53cff04a46e97740ac678d110ceccfa30e7f63715766d28ebd378770e9b3652d"})'
