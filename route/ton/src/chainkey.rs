use candid::Principal;
use ic_management_canister_types::{
    DerivationPath, SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs, SchnorrPublicKeyResponse,
    SignWithSchnorrArgs, SignWithSchnorrReply,
};
use serde_bytes::ByteBuf;
use tonlib_core::cell::{Cell, CellBuilder};
use tonlib_core::mnemonic::KeyPair;
use tonlib_core::wallet::{TonWallet, WalletVersion};

use crate::state::{mutate_state, read_state};

pub const EDDSA_SIGN_COST: u128 = 26_200_000_000;

pub async fn init_chain_pubkey() {
    let schnorr_key_name = "key_1".to_owned();
    let derived_path = vec![ByteBuf::from(crate::state::TON_CHAIN_ID.as_bytes())];
    let res: Result<(SchnorrPublicKeyResponse,), _> = ic_cdk::call(
        Principal::management_canister(),
        "schnorr_public_key",
        (SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: DerivationPath::new(derived_path),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: schnorr_key_name,
            },
        },),
    )
    .await;
    mutate_state(|s| s.pubkey = res.unwrap().0.public_key);
}

pub fn minter_addr() -> String {
    let pubkey = read_state(|s| s.pubkey.clone());
    let version = WalletVersion::V4R2;
    let fake_keypair = KeyPair {
        public_key: pubkey,
        secret_key: vec![],
    };
    let wallet = TonWallet::derive_default(version, &fake_keypair)
        .unwrap()
        .address;
    wallet.to_base64_std_flags(true, false)
}

pub async fn sign_external_body(external_body: &Cell) -> anyhow::Result<Cell> {
    let message_hash = external_body.cell_hash().to_vec();
    let sig = chainkey_sign(message_hash).await;
    let mut body_builder = CellBuilder::new();
    body_builder.store_slice(sig.as_slice())?;
    body_builder.store_cell(external_body)?;
    Ok(body_builder.build()?)
}

pub async fn chainkey_sign(msg_hash: Vec<u8>) -> Vec<u8> {
    let (chain_id, schnorr_key_name) = ("Ton".to_owned(), "key_1".to_owned());
    let derived_path = vec![ByteBuf::from(chain_id.as_bytes())];
    let res: Result<(SignWithSchnorrReply,), _> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "sign_with_schnorr",
        (SignWithSchnorrArgs {
            message: msg_hash,
            derivation_path: DerivationPath::new(derived_path),
            key_id: SchnorrKeyId {
                name: schnorr_key_name,
                algorithm: SchnorrAlgorithm::Ed25519,
            },
            aux: None,
        },),
        EDDSA_SIGN_COST as u64,
    )
    .await;
    res.unwrap().0.signature
}
