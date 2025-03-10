# Omnity Interoperability

Omnity is an omni-chain interoperability protocol built by Omnity Network on the Internet Computer (IC) specially designed to fit the modular blockchain landscape. It is implemented by a set of smart contracts deployed on IC.

## High-level Design

<img width="400" height="200" alt="Omnity" src="./img/omnity.png">

* E : settlement chains. currently we have bitcoin(supports both runes and brc20) and icp as the settlement chains.
* S : execution chains. currently we have the icp, bevm, bitlayer, b² network, x layer, merlin, bob, rootstock, bitfinity, ailayer, core, ethereum, osmosis and solana as the execution chains.
* Ticket: a transaction message.
* Hub: a canister (smart contract) on icp that handles chain and token registration and ticket (transaction) execution, and it also lists settlement chains and execution chains.
* Customs: the customs is where the assets are listed, each custom represents a settlement chain. the customs generates transfering tickets.
* Route: each route represents a execution chain. the routes generates redeeming tickets.

### Logical Architecture For Bitcoin Assets
<img width="400" height="200" alt="BTC" src="./img/btc.png">

* Gate represents Customs in this image.
* Spoke represents Routes in this image.
* [Ord Inderxer Canister](https://github.com/octopus-network/ord-canister):  a solution for fetching the runes information detail. the bitcoin header api will help the ord canister remove its trust assumption on rpc services.
* Bitcoin Canister: a native bitcoin integration on icp, the gateway where the bitcoin address can fetch its status like balance and make transactions through the provided apis.

## Current Supported Chains

* [Bitcoin](https://bitcoin.org/en/)
* [ICP](https://internetcomputer.org/)
* [Bevm](https://www.bevm.io/)
* [Bitlayer](https://www.bitlayer.org/)
* [B² Network](https://www.bsquared.network/)
* [X Layer](https://www.okx.com/xlayer)
* [Merlin](https://merlinchain.io/)
* [Bob](https://www.gobob.xyz/)
* [Rootstock](https://rootstock.io/)
* [Bitfinity](https://bitfinity.network/)
* [AILayer](https://ailayer.xyz/)
* [Core](https://coredao.org/)
* [Ethereum](https://ethereum.org/en/)
* [Osmosis](https://osmosis.zone/)
* [Solana](https://solana.com/)

## Social Media

* [X](https://twitter.com/OmnityNetwork)
* [Website](https://www.omnity.network/)
* [Dapp](https://bridge.omnity.network/)
* [Omnity API](https://docs.omnity.network/docs/intro)
* [OpenChat](https://oc.app/community/o5uz6-dqaaa-aaaar-bhnia-cai/channel/55564096078728941684293384519740574712/)
* [Medium](https://medium.com/omnity)
* [Red Envelope](https://oc.app/community/csmnf-nyaaa-aaaar-a2uda-cai/channel/257625026752796078802282812381756979432/?ref=iets5-biaaa-aaaaf-blpfq-cai)
* [Technical support](https://oc.app/community/o5uz6-dqaaa-aaaar-bhnia-cai/channel/209373796018851818071085429101874032721/)


## Audits

This repository has been audited by [Blocksec](https://blocksec.com/). See the [report](./auditing-reports/blocksec_omnity_v1.0_signed.pdf).
