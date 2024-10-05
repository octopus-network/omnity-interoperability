//! Utilities to derive, display, and parse bitcoin addresses.

use bech32::Variant;
use ic_btc_interface::Network;
use ic_crypto_extended_bip32::{DerivationIndex, DerivationPath, ExtendedBip32DerivationOutput};
use ic_crypto_sha2::Sha256;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::fmt;
use candid::CandidType;

// See https://en.bitcoin.it/wiki/List_of_address_prefixes.
const BTC_MAINNET_PREFIX: u8 = 0;
const BTC_MAINNET_P2SH_PREFIX: u8 = 5;
const BTC_TESTNET_PREFIX: u8 = 111;
const BTC_TESTNET_P2SH_PREFIX: u8 = 196;

const MAIN_DEST_TOKEN_SUFFIX: &str = "_PROD";

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ECDSAPublicKey {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitcoinAddress {
    /// Pay to witness public key hash address.
    /// See BIP-173.
    #[serde(rename = "p2wpkh_v0")]
    P2wpkhV0([u8; 20]),
    /// Pay to witness script hash address.
    /// See BIP-141.
    #[serde(rename = "p2wsh_v0")]
    P2wshV0([u8; 32]),
    /// Pay to taproot address.
    /// See BIP-341.
    #[serde(rename = "p2tr_v1")]
    P2trV1([u8; 32]),
    /// Pay to public key hash address.
    #[serde(rename = "p2pkh")]
    P2pkh([u8; 20]),
    /// Pay to script hash address.
    #[serde(rename = "p2sh")]
    P2sh([u8; 20]),
    /// Pay to OP_RETURN
    OpReturn(Vec<u8>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WitnessVersion {
    V0 = 0,
    V1 = 1,
}

impl BitcoinAddress {
    /// Converts the address to the textual representation.
    pub fn display(&self, network: Network) -> String {
        match self {
            Self::P2wpkhV0(pkhash) => encode_bech32(network, pkhash, WitnessVersion::V0),
            Self::P2wshV0(pkhash) => encode_bech32(network, pkhash, WitnessVersion::V0),
            Self::P2pkh(pkhash) => version_and_hash_to_address(
                match network {
                    Network::Mainnet => BTC_MAINNET_PREFIX,
                    Network::Testnet | Network::Regtest => BTC_TESTNET_PREFIX,
                },
                pkhash,
            ),
            Self::P2sh(script_hash) => version_and_hash_to_address(
                match network {
                    Network::Mainnet => BTC_MAINNET_P2SH_PREFIX,
                    Network::Testnet | Network::Regtest => BTC_TESTNET_P2SH_PREFIX,
                },
                script_hash,
            ),
            Self::P2trV1(pkhash) => encode_bech32(network, pkhash, WitnessVersion::V1),
            Self::OpReturn(_) => String::new(),
        }
    }

    /// Parses a bitcoin address and checks that it belongs to the specified network.
    pub fn parse(address: &str, network: Network) -> Result<BitcoinAddress, ParseAddressError> {
        // See https://en.bitcoin.it/wiki/Base58Check_encoding#Version_bytes.
        match address.chars().next() {
            Some('1') => parse_base58_address(address, network),
            Some('2') => parse_base58_address(address, network),
            Some('3') => parse_base58_address(address, network),
            Some('m') => parse_base58_address(address, network),
            Some('n') => parse_base58_address(address, network),
            Some('b') => parse_bip173_address(address, network),
            Some('B') => parse_bip173_address(address, network),
            Some('t') => parse_bip173_address(address, network),
            Some('T') => parse_bip173_address(address, network),
            Some(_) => Err(ParseAddressError::UnsupportedAddressType),
            None => Err(ParseAddressError::NoData),
        }
    }
}

pub fn main_bitcoin_address(
    ecdsa_public_key: &ECDSAPublicKey,
) -> BitcoinAddress {
    pubkey_to_bitcoin_address(ecdsa_public_key)
}

/// Derives a Bitcoin address for the specified destination and converts it into
/// bech32 textual representation.
pub fn network_publich_to_p2wpkh_address(
    network: Network,
    ecdsa_public_key: &ECDSAPublicKey,
) -> String {
    network_and_public_key_to_p2wpkh(
        network,
        &ecdsa_public_key.public_key,
    )
}

/// Constructs the bitcoin address corresponding to the specified destination.
pub fn pubkey_to_bitcoin_address(
    ecdsa_public_key: &ECDSAPublicKey,
) -> BitcoinAddress {
    use ripemd::{Digest, Ripemd160};
    BitcoinAddress::P2wpkhV0(Ripemd160::digest(Sha256::hash(&ecdsa_public_key.public_key)).into())
}

fn encode_bech32(network: Network, hash: &[u8], version: WitnessVersion) -> String {
    use bech32::u5;

    let hrp = hrp(network);
    let witness_version: u5 =
        u5::try_from_u8(version as u8).expect("bug: witness version must be smaller than 32");
    let data: Vec<u5> = std::iter::once(witness_version)
        .chain(
            bech32::convert_bits(hash, 8, 5, true)
                .expect("bug: bech32 bit conversion failed on valid inputs")
                .into_iter()
                .map(|b| {
                    u5::try_from_u8(b).expect("bug: bech32 bit conversion produced invalid outputs")
                }),
        )
        .collect();
    match version {
        WitnessVersion::V0 => bech32::encode(hrp, data, bech32::Variant::Bech32)
            .expect("bug: bech32 encoding failed on valid inputs"),
        WitnessVersion::V1 => bech32::encode(hrp, data, bech32::Variant::Bech32m)
            .expect("bug: bech32m encoding failed on valid inputs"),
    }
}

pub fn version_and_hash_to_address(version: u8, hash: &[u8; 20]) -> String {
    let mut buf = Vec::with_capacity(25);
    buf.push(version);
    buf.extend_from_slice(hash);
    let sha256d = Sha256::hash(&Sha256::hash(&buf));
    buf.extend_from_slice(&sha256d[0..4]);
    bs58::encode(&buf).into_string()
}

/// Calculates the p2wpkh address as described in [BIP-0173](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki).
///
/// # Panics
///
/// This function panics if the public key in not compressed.
pub fn network_and_public_key_to_p2wpkh(network: Network, public_key: &[u8]) -> String {
    assert_eq!(public_key.len(), 33);
    assert!(public_key[0] == 0x02 || public_key[0] == 0x03);
    use ripemd::{Digest, Ripemd160};
    let hash: [u8;20] =Ripemd160::digest(Sha256::hash(public_key)).into();
    encode_bech32(network, hash.as_ref(), WitnessVersion::V0)
}

/// Returns the human-readable part of a bech32 address
pub fn hrp(network: Network) -> &'static str {
    match network {
        ic_btc_interface::Network::Mainnet => "bc",
        ic_btc_interface::Network::Testnet => "tb",
        ic_btc_interface::Network::Regtest => "bcrt",
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseAddressError {
    InvalidBech32Variant { expected: Variant, found: Variant },
    UnsupportedAddressType,
    WrongNetwork { expected: Network, actual: Network },
    MalformedAddress(String),
    UnsupportedWitnessVersion(u8),
    UnexpectedHumanReadablePart { expected: String, actual: String },
    BadWitnessLength { expected: usize, actual: usize },
    NoData,
}

impl fmt::Display for ParseAddressError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedAddress(msg) => write!(fmt, "{}", msg),
            Self::UnsupportedWitnessVersion(v) => write!(fmt, "unsupported witness version {}", v),
            Self::UnexpectedHumanReadablePart { expected, actual } => {
                write!(fmt, "expected address HRP {}, got {}", expected, actual)
            }
            Self::BadWitnessLength { expected, actual } => write!(
                fmt,
                "expected witness program of length {}, got {}",
                expected, actual
            ),
            Self::UnsupportedAddressType => {
                write!(fmt, "Customs supports only P2WPKH and P2PKH addresses")
            }
            Self::WrongNetwork { expected, actual } => {
                write!(
                    fmt,
                    "expected an address from network {}, got an address from network {}",
                    expected, actual
                )
            }
            Self::NoData => write!(fmt, "the address contains no data"),
            Self::InvalidBech32Variant { expected, found } => write!(
                fmt,
                "invalid bech32 variant, expected: {:?}, found: {:?}",
                expected, found
            ),
        }
    }
}

fn parse_base58_address(
    address: &str,
    network: Network,
) -> Result<BitcoinAddress, ParseAddressError> {
    let bytes = bs58::decode(address)
        .into_vec()
        .map_err(|e| ParseAddressError::MalformedAddress(e.to_string()))?;

    if bytes.is_empty() {
        return Err(ParseAddressError::NoData);
    }

    // P2PKH or P2SH address
    // 1 byte address type + 20 bytes of PK hash + 4 bytes of checksum
    if bytes.len() != 25 {
        return Err(ParseAddressError::MalformedAddress(format!(
            "Expected the address to be 25 bytes, got {}",
            bytes.len(),
        )));
    }
    let checksum = Sha256::hash(&Sha256::hash(&bytes[0..21]));
    if checksum[0..4] != bytes[21..25] {
        return Err(ParseAddressError::MalformedAddress(format!(
            "checksum mismatch expected {}, got {}",
            hex::encode(&checksum[0..4]),
            hex::encode(&bytes[21..25]),
        )));
    }
    let mut data: [u8; 20] = [0; 20];
    data.copy_from_slice(&bytes[1..21]);

    if bytes[0] == BTC_MAINNET_PREFIX {
        if network != Network::Mainnet {
            return Err(ParseAddressError::WrongNetwork {
                expected: network,
                actual: Network::Mainnet,
            });
        }
        return Ok(BitcoinAddress::P2pkh(data));
    }

    if bytes[0] == BTC_TESTNET_PREFIX {
        if network != Network::Testnet && network != Network::Regtest {
            return Err(ParseAddressError::WrongNetwork {
                expected: network,
                actual: Network::Testnet,
            });
        }
        let mut pkhash: [u8; 20] = [0; 20];
        pkhash.copy_from_slice(&bytes[1..21]);
        return Ok(BitcoinAddress::P2pkh(data));
    }

    if bytes[0] == BTC_MAINNET_P2SH_PREFIX {
        if network != Network::Mainnet {
            return Err(ParseAddressError::WrongNetwork {
                expected: network,
                actual: Network::Mainnet,
            });
        }
        return Ok(BitcoinAddress::P2sh(data));
    }

    if bytes[0] == BTC_TESTNET_P2SH_PREFIX {
        if network != Network::Testnet && network != Network::Regtest {
            return Err(ParseAddressError::WrongNetwork {
                expected: network,
                actual: Network::Testnet,
            });
        }
        return Ok(BitcoinAddress::P2sh(data));
    }

    Err(ParseAddressError::UnsupportedAddressType)
}

/// Parses a BIP-0173 address.
fn parse_bip173_address(
    address: &str,
    network: Network,
) -> Result<BitcoinAddress, ParseAddressError> {
    let (found_hrp, five_bit_groups, variant) =
        bech32::decode(address).map_err(|e| ParseAddressError::MalformedAddress(e.to_string()))?;
    let expected_hrp = hrp(network);

    if found_hrp.to_lowercase() != expected_hrp {
        return Err(ParseAddressError::UnexpectedHumanReadablePart {
            expected: expected_hrp.to_string(),
            actual: found_hrp,
        });
    }

    if five_bit_groups.is_empty() {
        return Err(ParseAddressError::NoData);
    }

    let witness_version = five_bit_groups[0].to_u8();

    match witness_version {
        0 => {
            if variant != bech32::Variant::Bech32 {
                return Err(ParseAddressError::InvalidBech32Variant {
                    expected: bech32::Variant::Bech32,
                    found: variant,
                });
            }

            let data = bech32::convert_bits(
                &five_bit_groups[1..],
                /*from=*/ 5,
                /*to=*/ 8,
                /*pad=*/ false,
            )
                .map_err(|e| {
                    ParseAddressError::MalformedAddress(format!(
                        "failed to decode witness from address {}: {}",
                        address, e
                    ))
                })?;

            match data.len() {
                20 => {
                    let mut pkhash = [0u8; 20];
                    pkhash[..].copy_from_slice(&data[..]);

                    Ok(BitcoinAddress::P2wpkhV0(pkhash))
                }
                32 => {
                    let mut script_hash = [0u8; 32];
                    script_hash[..].copy_from_slice(&data[..]);

                    Ok(BitcoinAddress::P2wshV0(script_hash))
                }
                _ => Err(ParseAddressError::BadWitnessLength {
                    expected: 20,
                    actual: data.len(),
                }),
            }
        }
        1 => {
            if variant != bech32::Variant::Bech32m {
                return Err(ParseAddressError::InvalidBech32Variant {
                    expected: bech32::Variant::Bech32m,
                    found: variant,
                });
            }
            let data = bech32::convert_bits(
                &five_bit_groups[1..],
                /*from=*/ 5,
                /*to=*/ 8,
                /*pad=*/ false,
            )
                .map_err(|e| {
                    ParseAddressError::MalformedAddress(format!(
                        "failed to decode witness from address {}: {}",
                        address, e
                    ))
                })?;

            if data.len() != 32 {
                return Err(ParseAddressError::BadWitnessLength {
                    expected: 32,
                    actual: data.len(),
                });
            }
            let mut pkhash = [0u8; 32];
            pkhash[..].copy_from_slice(&data[..]);

            Ok(BitcoinAddress::P2trV1(pkhash))
        }
        _ => Err(ParseAddressError::UnsupportedWitnessVersion(
            witness_version,
        )),
    }
}