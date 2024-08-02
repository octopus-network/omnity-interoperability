use crate::*;

// use schnorr_canister::{SchnorrKeyIds, SignWithSchnorrArgs, SignWithSchnorrResult};
use crate::state;
use serde_bytes::ByteBuf;

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct SchnorrPublicKeyArgs {
    pub canister_id: Option<Principal>,
    pub derivation_path: Vec<ByteBuf>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SchnorrPublicKeyResult {
    pub public_key: ByteBuf,
    pub chain_code: ByteBuf,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct SignWithSchnorrArgs {
    pub message: ByteBuf,
    pub derivation_path: Vec<ByteBuf>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SignWithSchnorrResult {
    pub signature: ByteBuf,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchnorrAlgorithm {
    #[serde(rename = "bip340secp256k1")]
    Bip340Secp256k1,
    #[serde(rename = "ed25519")]
    Ed25519,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchnorrKeyId {
    algorithm: SchnorrAlgorithm,
    name: String,
}

pub enum SchnorrKeyIds {
    DfxTestKey,
    TestKey1,
    DfxTestKeyEd25519,
    TestKey1Ed25519,
}

impl SchnorrKeyIds {
    pub fn to_key_id(&self) -> SchnorrKeyId {
        match self {
            Self::DfxTestKey => SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Bip340Secp256k1,
                name: "dfx_test_key".to_string(),
            },
            Self::TestKey1 => SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Bip340Secp256k1,
                name: "test_key_1".to_string(),
            },
            Self::DfxTestKeyEd25519 => SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: "dfx_test_key".to_string(),
            },
            Self::TestKey1Ed25519 => SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: "test_key_1".to_string(),
            },
        }
    }

    fn variants() -> Vec<SchnorrKeyIds> {
        vec![
            SchnorrKeyIds::DfxTestKey,
            SchnorrKeyIds::TestKey1,
            SchnorrKeyIds::DfxTestKeyEd25519,
            SchnorrKeyIds::TestKey1Ed25519,
        ]
    }
}

pub async fn cw_schnorr_public_key() -> Result<SchnorrPublicKeyResult> {
    let schnorr_canister_principal: candid::Principal =
        state::read_state(|state| state.schnorr_canister_principal);

    let derivation_path: Vec<ByteBuf> = [vec![1u8; 4]] // Example derivation path for signing
        .iter()
        .map(|v| ByteBuf::from(v.clone()))
        .collect();

    let public_arg = SchnorrPublicKeyArgs {
        canister_id: Some(ic_cdk::api::id()),
        derivation_path: derivation_path.clone(),
        key_id: SchnorrKeyIds::TestKey1.to_key_id(),
    };

    let res: (SchnorrPublicKeyResult,) = ic_cdk::api::call::call(
        schnorr_canister_principal,
        "schnorr_public_key",
        (public_arg,),
    )
    .await
    .map_err(|(code, message)| {
        RouteError::CallError(
            "schnorr_public_key".to_string(),
            schnorr_canister_principal,
            format!("{:?}", code).to_string(),
            message,
        )
    })?;

    Ok(res.0)
}

pub async fn sign_with_schnorr(
    message: &[u8],
    key_id: SchnorrKeyId,
) -> Result<SignWithSchnorrResult> {
    let schnorr_canister_principal = state::read_state(|state| state.schnorr_canister_principal);

    let derivation_path: Vec<ByteBuf> = [vec![1u8; 4]] // Example derivation path for signing
        .iter()
        .map(|v| ByteBuf::from(v.clone()))
        .collect();

    let sign_with_schnorr_args = SignWithSchnorrArgs {
        message: ByteBuf::from(message.to_vec()),
        derivation_path,
        key_id: key_id,
    };

    let res: (SignWithSchnorrResult,) = ic_cdk::api::call::call(
        schnorr_canister_principal,
        "sign_with_schnorr",
        (sign_with_schnorr_args,),
    )
    .await
    .map_err(|(code, message)| {
        RouteError::CallError(
            "sign_with_schnorr".to_string(),
            schnorr_canister_principal,
            format!("{:?}", code).to_string(),
            message,
        )
    })?;
    Ok(res.0)
}
