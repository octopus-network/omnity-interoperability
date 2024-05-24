
# https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/local-development#setting-up-a-local-bitcoin-network
$ bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444
$ cd omnity
git checkout feature/rename-evm
$ dfx stop
$ dfx start --clean
$ cargo clean

# https://internetcomputer.org/docs/current/developer-docs/defi/icp-tokens/ledger-local-setup
dfx identity new minter
dfx identity use minter
export MINTER_ACCOUNT_ID=$(dfx ledger account-id)
dfx identity use default
export DEFAULT_ACCOUNT_ID=$(dfx ledger account-id)
dfx deploy --specified-id ryjl3-tyaaa-aaaaa-aaaba-cai icp_ledger_canister --argument "
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
dfx ledger balance $DEFAULT_ACCOUNT_ID
100.00000000 ICP


$ dfx identity --identity default get-principal
oqqew-3kok2-4ca2v-uwf4q-bykqb-yghly-kwet3-a5vqf-cu4ug-ztg4o-sqe
 dfx deploy omnity_hub --argument '(variant { Init = record { admin = principal "oqqew-3kok2-4ca2v-uwf4q-bykqb-yghly-kwet3-a5vqf-cu4ug-ztg4o-sqe"} })'
 dfx deploy bitcoin_customs --argument '(variant { Init = record { mode = variant { GeneralAvailability }; btc_network = variant { Regtest }; hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; ecdsa_key_name = "dfx_test_key"; min_confirmations = opt 1; max_time_in_queue_nanos = 1_000_000_000; runes_oracle_principal = principal "oqqew-3kok2-4ca2v-uwf4q-bykqb-yghly-kwet3-a5vqf-cu4ug-ztg4o-sqe"; chain_id = "Bitcoin"; chain_state = variant { Active } } })'
# dfx deploy icp_route --argument '(variant { Init = record { hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; chain_id = "eICP"; chain_state = variant { Active } } })'
#deploy evm_rpc
dfx deploy evm_rpc --argument '(record { nodesInSubnet = 28 })'
#deploy cdk route
cd route/evm
cargo build --release --target wasm32-unknown-unknown --package evm_route
candid-extractor target/wasm32-unknown-unknown/release/evm_route.wasm > evm_route.did
cd ../..
#scan_start_height 为扫链起始高度， 部署时先到bevm 测试网浏览器查询一下。 https://scan-testnet.bevm.io
dfx deploy evm_route --argument '(record { fee_token_id = "bevmBTC"; network = variant { local }; scan_start_height = 1164700; evm_rpc_canister_addr = principal "br5f7-7uaaa-aaaaa-qaaca-cai";  evm_chain_id = 11503; admin = principal "oqqew-3kok2-4ca2v-uwf4q-bykqb-yghly-kwet3-a5vqf-cu4ug-ztg4o-sqe"; hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; chain_id = "bevm"; rpc_url = "https://testnet.bevm.io";})'

#generate chainkey
dfx canister call evm_route init_chain_pubkey
("038e56b24f8fa09d006cbcb4cacdd2fa35764f01a007f5f1b234e139545f9e3557")

#get chainkey ecdsa address
dfx canister call evm_route pubkey_and_evm_addr
(
  "0x038e56b24f8fa09d006cbcb4cacdd2fa35764f01a007f5f1b234e139545f9e3557",
  "0x30dbCA3314c49e4c59A9815CF87DDC7E5205C327",  #the address is chainkey addr
)

//使用钱包在bevm-testnet转点手续费到0x30dbCA3314c49e4c59A9815CF87DDC7E5205C327 0.001个BTC足够了

# deploy port contract
git clone git@github.com:octopus-network/omnity-port-solidity.git
cd omnity-port-solidity
git checkout feature/yh-fix
npx hardhat compile //需要安装hardhat相关组件

cat hardhat.config.ts
```
  import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "@nomicfoundation/hardhat-ignition-ethers";
import { vars } from "hardhat/config";
const DEPLOY_PRI_KEY = vars.get("DEPLOY_PRI_KEY");
const config: HardhatUserConfig = {
  solidity: "0.8.19",
  networks: {
    sepolia: {
      url: `https://rpc-sepolia.rockx.com`,
      accounts: [DEPLOY_PRI_KEY],
    },
    bevm_testnet: {
      url: `https://testnet.bevm.io`,
      accounts: [DEPLOY_PRI_KEY],
    }
  }
};
export default config;

```

#set DEPLOY_PRI_KEY in you vars
npx hardhat vars set DEPLOY_PRI_KEY
enter value:  your evm account private key #这个地址用于部署port 合约， 可以是任意地址， 需要手续费可以找叶欢要

cat ignition/modules/omnity_port.js
```
const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");

const ProtModule = buildModule("PortModule", (m) => {
  //param: routes chainkey address
  const port = m.contract("OmnityPortContract",[ "0x30dbCA3314c49e4c59A9815CF87DDC7E5205C327"]); //chainkey addr
  return { port };
});

module.exports = ProtModule;

```
rm -rf ./ignition/deployments
npx hardhat ignition deploy ./ignition/modules/omnity_port.js --network bevm_testnet
```
    ✔ Confirm deploy to network bevm_testnet (11503)? … yes
    Hardhat Ignition 🚀

    Deploying [ PortModule ]

    Batch #1
      Executed PortModule#OmnityPortContract

    [ PortModule ] successfully deployed 🚀

    Deployed Addresses

    PortModule#OmnityPortContract - 0x641a149324E52600E09C30Bf8904e1636D689436
```

#set port address into route
dfx canister call evm_route set_omnity_port_contract_addr '("0x641a149324E52600E09C30Bf8904e1636D689436")'


# https://github.com/lesterli/ord/blob/docs/runes/docs/src/guides/runes.md
$ git clone https://github.com/octopus-network/ord.git
$ git checkout dev
 docker run --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=mysecretpassword -v ~/dev/data:/var/lib/postgresql/data -d postgres:12
  docker run -it --rm --network host postgres:12 psql -h 127.0.0.1 -U postgres
postgres=# CREATE DATABASE runescan ENCODING = 'UTF8';
$ sudo docker exec -i postgres psql -U postgres -d runescan < deploy/runescan.sql
$ export DATABASE_URL=postgres://postgres:mysecretpassword@127.0.0.1:5432/runescan
$ cargo build
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet create
{
  "mnemonic": "area century math finger flip salad top ankle lucky decline army earth",
  "passphrase": ""
}

$ rm -rf ~/.local/share/ord/regtest/index.redb
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes server --http --http-port 23456 --address 0.0.0.0

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 receive
{
  "addresses": [
    "bcrt1pdxuujrka3njx08ee0pncvk3eer5zrfs328p7tjpnlh2357k47qxqg07rqv"
  ]
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 101 bcrt1pdxuujrka3njx08ee0pncvk3eer5zrfs328p7tjpnlh2357k47qxqg07rqv

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
Waiting for rune UNCOMMONGOODS commitment 138eeaa503808b174becbf6f5c346c0c4dca98406e07b8d85431d8ce7b8496f4 to mature…

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 5 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
Maturing in...[0s]   [████████████████████████████████████████] 6/6
{
  "commit": "138eeaa503808b174becbf6f5c346c0c4dca98406e07b8d85431d8ce7b8496f4",
  "commit_psbt": null,
  "inscriptions": [
    {
      "destination": "bcrt1p4l7lfcncfe8egn6hrh57kppchzksvt2muhnyxkqgzrv5209u8jjqzlewyd",
      "id": "5bdea4a4dafef3eb00dd36c6f4d90ab4c47b306b353583d52184fac8090bf0afi0",
      "location": "5bdea4a4dafef3eb00dd36c6f4d90ab4c47b306b353583d52184fac8090bf0af:0:0"
    }
  ],
  "parent": null,
  "reveal": "5bdea4a4dafef3eb00dd36c6f4d90ab4c47b306b353583d52184fac8090bf0af",
  "reveal_broadcast": true,
  "reveal_psbt": null,
  "rune": {
    "destination": "bcrt1p8wdtay82az6wkdlx9gpl48e5qrre6a6563zsudaknzksda2uctwsydhwqr",
    "location": "5bdea4a4dafef3eb00dd36c6f4d90ab4c47b306b353583d52184fac8090bf0af:1",
    "rune": "UNCOMMON•GOODS"
  },
  "total_fees": 432
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 34999979568,
  "ordinal": 10000,
  "runes": {
    "UNCOMMON•GOODS": "1000000"
  },
  "runic": 10000,
  "total": 34999999568
}

http://192.168.0.111:23456/rune/UNCOMMON%E2%80%A2GOODS
rune_id: 107:1

# sub hub topic
$ dfx canister call omnity_hub sub_directives '(opt "Bitcoin", vec {variant {AddChain};variant {AddToken};variant {UpdateFee};variant {ToggleChainState}})'
$ dfx canister call omnity_hub sub_directives '(opt "bevm", vec {variant {AddChain};variant {AddToken};variant {UpdateFee};variant {ToggleChainState}})'

dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"bevm"}; fee_token=null}}})'
dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "Bitcoin"; chain_type=variant { SettlementChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai"; contract_address=null;counterparties=opt vec {"bevm"}; fee_token=null}}})'

dfx canister call omnity_hub validate_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "bevm"; chain_type=variant { ExecutionChain };canister_id="bw4dl-smaaa-aaaaa-qaacq-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}; fee_token=opt "bevmBTC"}}})'
dfx canister call omnity_hub execute_proposal  '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "bevm"; chain_type=variant { ExecutionChain };canister_id="bw4dl-smaaa-aaaaa-qaacq-cai";  contract_address=null; counterparties= opt vec {"Bitcoin"}; fee_token=opt "bevmBTC"}}})'

dfx canister call omnity_hub validate_proposal '( vec {variant { AddToken = record { decimals = 2 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; name = "UNCOMMON•GOODS";issue_chain = "Bitcoin"; symbol = "UNCOMMON•GOODS"; metadata =  vec{ record {"rune_id"; "107:1"}}; dst_chains = vec {"Bitcoin";"bevm";}}}})'
dfx canister call omnity_hub execute_proposal  '( vec {variant { AddToken = record { decimals = 2 : nat8; icon = opt "rune.logo.url"; token_id = "Bitcoin-runes-UNCOMMON•GOODS"; name = "UNCOMMON•GOODS";issue_chain = "Bitcoin"; symbol = "UNCOMMON•GOODS"; metadata =  vec{ record {"rune_id"; "107:1"}}; dst_chains = vec {"Bitcoin";"bevm";}}}})'

# update fee
$ dfx canister call omnity_hub update_fee 'vec {variant { UpdateTargetChainFactor = record {target_chain_id="Bitcoin"; target_chain_factor=10000 : nat}}; variant { UpdateFeeTokenFactor = record { fee_token="bevmBTC"; fee_token_factor=1 : nat}}}'


# query update fee directive
$ dfx canister call omnity_hub query_directives '(opt "bevm",null,0:nat64,5:nat64)'
$ dfx canister call omnity_hub query_directives '(opt "Bitcoin",null,0:nat64,5:nat64)'


#generate btc addr
$ dfx canister call bitcoin_customs get_btc_address '(record {target_chain_id = "bevm"; receiver = "0x544F52f459a42E098775118e0A1880f1FA3eb9a9"})'
("bcrt1qz0j9drvpc3p60jh2suxtj4dx9xjkg3wecqq6d3")

#send runes to bevm
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= --index-runes wallet --server-url http://127.0.0.1:23456 send --fee-rate 1 bcrt1qz0j9drvpc3p60jh2suxtj4dx9xjkg3wecqq6d3 70000:UNCOMMON•GOODS
{
  "txid": "78a76d3e4565856b812412d4946a771cd585ce81d77ae39f749e910e26478105",
  "psbt": "cHNidP8BAOYCAAAAAvD56A9YdFfkX6be9pZ9AyjZvyGc/yUnJSxpSnFTpl63AQAAAAD/////j/BtwSQ7R5wNEJiYVBTL8ifHXvtOK/+vvHf/jZr9eSgBAAAAAP3///8EAAAAAAAAAAAMal0JANABAcCfqwMCECcAAAAAAAAiUSAtxtxbiM8iKjXIxmRReKVkqOr+qqZGR5Zrbsl93uo8KRAnAAAAAAAAFgAUE+RWjYHEQ6fK6ocMuVWmKaVkRdkYegUqAQAAACJRINhcPg05OWP3/y7OyjWo7n5HmMqH1X+4wO3SiCyp+tfrAAAAAAABASsQJwAAAAAAACJRIPbqNbJGzxfRObSFcYTmwRNZ+ikl7MmX4SmfV4iXuVSfAQhCAUDDTeVy5TMoNYLXPm14fHX3BXSCMB++6Ig+Y14X+mZwj05X7pQhBYCIsolyFfEsKbWw1SM8a28mnXDtn8RBk2rCAAEBKzCiBSoBAAAAIlEgNqfTxl5ZprNF+cDN+lKgh8uGoMVgRS5Teo/2vuQnZQkBCEIBQEUtxYdc1BgB4coPhkHtU58yHnpONP21nukdiP00Ei+Kt7hqKn1Ne31Pozl9b4n/DJr1AysaGkEQIhCiR6d0fgAAAAEFIO+D57xQZPPs+8BAIW7udtrqJe4HQIptomhYabwCm7CcIQfvg+e8UGTz7PvAQCFu7nba6iXuB0CKbaJoWGm8ApuwnBkA8u/XbVYAAIABAACAAAAAgAEAAAAEAAAAAAABBSDkVPGNx3AjP2MUfOj6toC6MIRZAwzWs6cV0CLc9cxeFyEH5FTxjcdwIz9jFHzo+raAujCEWQMM1rOnFdAi3PXMXhcZAPLv121WAACAAQAAgAAAAIABAAAABQAAAAA=",
  "outgoing": "70000:UNCOMMON•GOODS",
  "fee": 264
}

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww

$ dfx canister call bitcoin_customs generate_ticket '(record {target_chain_id = "bevm"; receiver = "0x544F52f459a42E098775118e0A1880f1FA3eb9a9"; rune_id = "107:1"; amount = 7000000; txid = "78a76d3e4565856b812412d4946a771cd585ce81d77ae39f749e910e26478105"})'

$ dfx canister call bitcoin_customs get_pending_gen_ticket_requests '(record {max_count = 3; start_txid = null})'
(
  vec {
    record {
      received_at = 1_716_535_291_403_752_000 : nat64;
      token_id = "Bitcoin-runes-UNCOMMON•GOODS";
      txid = blob "\05\81\47\26\0e\91\9e\74\9f\e3\7a\d7\81\ce\85\d5\1c\77\6a\94\d4\12\24\81\6b\85\65\45\3e\6d\a7\78";
      target_chain_id = "bevm";
      address = "bcrt1qz0j9drvpc3p60jh2suxtj4dx9xjkg3wecqq6d3";
      amount = 7_000_000 : nat;
      receiver = "0x544F52f459a42E098775118e0A1880f1FA3eb9a9";
      rune_id = record { tx = 1 : nat32; block = 208 : nat64 };
    };
  },
)

$ cargo build -p runes_oracle

export INDEXER_URL=http://localhost:23456
export PEM_PATH=/Users/yehuan/.config/dfx/identity/default/identity.pem
export IC_GATEWAY=http://localhost:4943
export CUSTOMS_CANISTER_ID=be2us-64aaa-aaaaa-qaabq-cai
$ RUST_LOG=info ./target/debug/runes_oracle
$ dfx canister call omnity_hub query_tickets '(opt "bevm", 0, 10)'
(
  variant {
    Ok = vec {
      record {
        0 : nat64;
        record {
          token = "Bitcoin-runes-UNCOMMON•GOODS";
          action = variant { Transfer };
          dst_chain = "bevm";
          memo = null;
          ticket_id = "78a76d3e4565856b812412d4946a771cd585ce81d77ae39f749e910e26478105";
          sender = null;
          ticket_time = 1_716_535_493_222_171_000 : nat64;
          ticket_type = variant { Normal };
          src_chain = "Bitcoin";
          amount = "7000000";
          receiver = "0x544F52f459a42E098775118e0A1880f1FA3eb9a9";
        };
      };
    }
  },
)

#稍等片刻 去bevm 查看账户余额

#redeem，直接调用合约 port 合约 redeemToken,
参数
 tokenId: Bitcoin-runes-UNCOMMON•GOODS
 receiver: bcrt1pdxuujrka3njx08ee0pncvk3eer5zrfs328p7tjpnlh2357k47qxqg07rqv
 amount: 20000
 fee: 10000

#成功之后稍等预计1分钟， 查看ticket

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
          ticket_id = "85129f909497dee273bb9cc5f6a852db26e06b733b720aa79e0cd5e941720ef1-1";
          sender = null;
          ticket_time = 1_168_132 : nat64;
          ticket_type = variant { Normal };
          src_chain = "bevm";
          amount = "20000";
          receiver = "bcrt1pdxuujrka3njx08ee0pncvk3eer5zrfs328p7tjpnlh2357k47qxqg07rqv";
        };
      };
    }
  },
)

$ bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 bcrt1p0lj28skrcfnanufwdmll75338gk75rzh3ejkv9dvy3e0cdrsuh5qwq8pww
# $ dfx canister call bitcoin_customs update_btc_utxos
$ dfx canister call bitcoin_customs get_events '(record {start = 0; length = 100})'
$ ./target/debug/ord -r --bitcoin-data-dir ~/dev/bitcoin/data --bitcoin-rpc-username ic-btc-integration --bitcoin-rpc-password QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= wallet --server-url http://127.0.0.1:23456 balance
et --server-url http://127.0.0.1:23456 balance
{
  "cardinal": 44999969304,
  "ordinal": 10000,
  "runes": {
    "UNCOMMON•GOODS": "930200"
  },
  "runic": 10546,
  "total": 44999989850
}
