# README

## Project Overview

This repository implements the functionality of allowing users to perform cross-chain transactions between BTC and CosmWasm utilizing ckBTC, without needing an ICP account.

## Feature Summary

* Generate a btc mint address controlled by ckBTC based on the user's Osmosis account.
* Assist users in holding ckBTC.
* Interact with Omnity Customs to initiate cross-chain transfers and withdrawals.
* Provide related querying services.


## High Level Design

![](../../img/proxy-transport.jpg)

(1) The Proxy Canister provides the get_account_identity interface, which maps the user's passed osmo address to a sub-account of the Proxy Canister as the AccountIdentity.

(2) Use the AccountIdentity obtained in (1) to query the btc_address controlled by the ckBTC Canister, and transfer funds to this address to trigger ckBTC's cross-chain functionality.

(3) An on-chain process continuously monitors transactions to the Proxy Canister on the ckBTC Canister. If a transfer is detected, it will trigger the Proxy Canister's trigger_generate_ticket interface along with the block index of the transaction.

(4) After the Proxy Canister's trigger_generate_ticket interface is called, it will request ckBTC to verify the relevant transaction and then approve the corresponding amount to the Icp Custom Canister.

(5) Calling the generate_ticket interface of the Icp Custom Canister will trigger the cross-chain process.