#![allow(unused)]
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct ChainParams {
    pub chain_name: &'static str,
    pub genesis_block: &'static str,
    pub p2pkh_address_prefix: u8,
    pub p2sh_address_prefix: u8,
    pub pkey_prefix: u8,
    pub bip32_privkey_prefix: u32,
    pub bip32_pubkey_prefix: u32,
    pub bip32_wif_privkey_prefix: &'static str,
    pub bip32_wif_pubkey_prefix: &'static str,
}

pub static DOGE_MAIN_NET_CHAIN: ChainParams = ChainParams {
    chain_name: "main",
    genesis_block: "1a91e3dace36e2be3bf030a65679fe821aa1d6ef92e7c9902eb318182c355691",
    p2pkh_address_prefix: 0x1e,       // D
    p2sh_address_prefix: 0x16,        // 9 or A
    pkey_prefix: 0x9e,                // Q or 6
    bip32_privkey_prefix: 0x02fac398, // dgpv
    bip32_pubkey_prefix: 0x02facafd,  // dgub
    bip32_wif_privkey_prefix: "dgpv",
    bip32_wif_pubkey_prefix: "dgub",
};

pub static DOGE_TEST_NET_CHAIN: ChainParams = ChainParams {
    chain_name: "test",
    genesis_block: "bb0a78264637406b6360aad926284d544d7049f45189db5664f3c4d07350559e",
    p2pkh_address_prefix: 0x71,       // n
    p2sh_address_prefix: 0xc4,        // 2
    pkey_prefix: 0xf1,                // 9 or c
    bip32_privkey_prefix: 0x04358394, // tprv
    bip32_pubkey_prefix: 0x043587cf,  // tpub
    bip32_wif_privkey_prefix: "tprv",
    bip32_wif_pubkey_prefix: "tpub",
};

pub static DOGE_REG_TEST_CHAIN: ChainParams = ChainParams {
    chain_name: "regtest",
    genesis_block: "3d2160a3b5dc4a9d62e7e66a295f70313ac808440ef7400d6c0772171ce973a5",
    p2pkh_address_prefix: 0x6f,       // n
    p2sh_address_prefix: 0xc4,        // 2
    pkey_prefix: 0xef,                //
    bip32_privkey_prefix: 0x04358394, // tprv
    bip32_pubkey_prefix: 0x043587cf,  // tpub
    bip32_wif_privkey_prefix: "tprv",
    bip32_wif_pubkey_prefix: "tpub",
};

pub type KeyBits = u8; // keyECPriv,keyECPub,keyBip32Priv,keyBip32Pub,dogeMainNet,dogeTestNet

pub const KEY_NONE: KeyBits = 0;
pub const KEY_ECPRIV: KeyBits = 1;
pub const KEY_ECPUB: KeyBits = 2;
pub const KEY_BIP32_PRIV: KeyBits = 4;
pub const KEY_BIP32_PUB: KeyBits = 8;
pub const MAIN_NET_DOGE: KeyBits = 16;
pub const TEST_NET_DOGE: KeyBits = 32;
pub const MAIN_NET_BTC: KeyBits = 64;

pub fn chain_from_key_bits(key: KeyBits) -> &'static ChainParams {
    if (key & MAIN_NET_DOGE) != 0 {
        return &DOGE_MAIN_NET_CHAIN;
    }

    &DOGE_TEST_NET_CHAIN // fallback
}

pub fn key_bits_for_chain(chain: &ChainParams) -> KeyBits {
    let mut bits = 0u8;
    if chain == &DOGE_MAIN_NET_CHAIN {
        bits |= MAIN_NET_DOGE
    } else if chain == &DOGE_TEST_NET_CHAIN {
        bits |= TEST_NET_DOGE
    }
    bits
}

pub fn chain_from_wif(wif: &str) -> &'static ChainParams {
    let wif = wif.as_bytes();
    if wif.is_empty() {
        return &DOGE_TEST_NET_CHAIN; // fallback
    }
    match wif[0] {
        b'D' | b'9' | b'A' | b'Q' | b'6' | b'd' => &DOGE_MAIN_NET_CHAIN,
        _ => &DOGE_TEST_NET_CHAIN,
    }
}
