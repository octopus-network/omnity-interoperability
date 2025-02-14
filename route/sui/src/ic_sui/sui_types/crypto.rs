#![allow(unused)]

use crate::ic_sui::sui_types::base_types::AuthorityName;

use crate::ic_sui::fastcrypto::encoding::{Base64, Encoding, Hex};
use crate::ic_sui::fastcrypto::error::FastCryptoError;
use crate::ic_sui::fastcrypto::hash::{Blake2b256, HashFunction};
// pub use crate::ic_sui::fastcrypto::traits::KeyPair as KeypairTraits;
pub use crate::ic_sui::fastcrypto::traits::Signer;
pub use crate::ic_sui::fastcrypto::traits::{
    AggregateAuthenticator, Authenticator, EncodeDecodeBase64, SigningKey, ToFromBytes,
    VerifyingKey,
};
use crate::ic_sui::sui_types::error::SuiError;
use crate::ic_sui::sui_types::signature::GenericSignature;
use crate::ic_sui::sui_types::sui_serde::Readable;
use anyhow::{anyhow, Error};
use derive_more::{AsMut, AsRef, From};
pub use enum_dispatch::enum_dispatch;
use sui_crypto::ed25519::Ed25519PrivateKey;
use sui_sdk_types::types::{Ed25519PublicKey, Ed25519Signature};

use crate::ic_sui::shared_inent::intent::{Intent, IntentMessage};
use serde::ser::Serializer;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, Bytes};
use sui_sdk_types::types::{Bls12381PublicKey, Bls12381Signature};
use sui_sdk_types::types::{Secp256k1PublicKey, Secp256k1Signature};
use sui_sdk_types::types::{Secp256r1PublicKey, Secp256r1Signature};

use std::fmt::Debug;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use strum::EnumString;

use super::transaction::EpochId;
// use tracing::{instrument, warn};

// Authority Objects
// pub type AuthorityKeyPair = BLS12381KeyPair;
pub type AuthorityPublicKey = Bls12381PublicKey;
// pub type AuthorityPrivateKey = BLS12381PrivateKey;
pub type AuthoritySignature = Bls12381Signature;
// pub type AggregateAuthoritySignature = BLS12381AggregateSignature;
// pub type AggregateAuthoritySignatureAsBytes = BLS12381AggregateSignatureAsBytes;

// TODO(joyqvq): prefix these types with Default, DefaultAccountKeyPair etc
// pub type AccountKeyPair = Ed25519KeyPair;
pub type AccountPublicKey = Ed25519PublicKey;
pub type AccountPrivateKey = Ed25519PrivateKey;

// pub type NetworkKeyPair = Ed25519KeyPair;
pub type NetworkPublicKey = Ed25519PublicKey;
pub type NetworkPrivateKey = Ed25519PrivateKey;

pub type DefaultHash = Blake2b256;

pub const DEFAULT_EPOCH_ID: EpochId = 0;
pub const SUI_PRIV_KEY_PREFIX: &str = "suiprivkey";

/// A wrapper struct to retrofit in [enum PublicKey] for zkLogin.
/// Useful to construct [struct MultiSigPublicKey].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZkLoginPublicIdentifier(pub Vec<u8>);

impl ZkLoginPublicIdentifier {}

/// Defines the compressed version of the public key that we pass around
/// in Sui
#[serde_as]
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, AsRef)]
#[as_ref(forward)]
pub struct AuthorityPublicKeyBytes(
    #[serde_as(as = "Readable<Base64, Bytes>")] pub [u8; AuthorityPublicKey::LENGTH],
);

impl AuthorityPublicKeyBytes {
    fn fmt_impl(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let s = Hex::encode(self.0);
        write!(f, "k#{}", s)?;
        Ok(())
    }
}

/// A wrapper around AuthorityPublicKeyBytes that provides a concise Debug impl.
pub struct ConciseAuthorityPublicKeyBytesRef<'a>(&'a AuthorityPublicKeyBytes);

impl Debug for ConciseAuthorityPublicKeyBytesRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let s = Hex::encode(self.0 .0.get(0..4).ok_or(std::fmt::Error)?);
        write!(f, "k#{}..", s)
    }
}

impl Display for ConciseAuthorityPublicKeyBytesRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// A wrapper around AuthorityPublicKeyBytes but owns it.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConciseAuthorityPublicKeyBytes(AuthorityPublicKeyBytes);

impl Debug for ConciseAuthorityPublicKeyBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let s = Hex::encode(self.0 .0.get(0..4).ok_or(std::fmt::Error)?);
        write!(f, "k#{}..", s)
    }
}

impl Display for ConciseAuthorityPublicKeyBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, f)
    }
}

impl From<&AuthorityPublicKey> for AuthorityPublicKeyBytes {
    fn from(pk: &AuthorityPublicKey) -> AuthorityPublicKeyBytes {
        AuthorityPublicKeyBytes::from_bytes(pk.as_ref()).unwrap()
    }
}

impl Debug for AuthorityPublicKeyBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.fmt_impl(f)
    }
}

impl Display for AuthorityPublicKeyBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.fmt_impl(f)
    }
}

impl ToFromBytes for AuthorityPublicKeyBytes {
    fn from_bytes(bytes: &[u8]) -> Result<Self, crate::ic_sui::fastcrypto::error::FastCryptoError> {
        let bytes: [u8; AuthorityPublicKey::LENGTH] = bytes
            .try_into()
            .map_err(|_| crate::ic_sui::fastcrypto::error::FastCryptoError::InvalidInput)?;
        Ok(AuthorityPublicKeyBytes(bytes))
    }
}

impl AuthorityPublicKeyBytes {
    pub const ZERO: Self = Self::new([0u8; AuthorityPublicKey::LENGTH]);

    /// This ensures it's impossible to construct an instance with other than registered lengths
    pub const fn new(bytes: [u8; AuthorityPublicKey::LENGTH]) -> AuthorityPublicKeyBytes
where {
        AuthorityPublicKeyBytes(bytes)
    }
}

impl FromStr for AuthorityPublicKeyBytes {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = Hex::decode(s).map_err(|e| anyhow!(e))?;
        Self::from_bytes(&value[..]).map_err(|e| anyhow!(e))
    }
}

impl Default for AuthorityPublicKeyBytes {
    fn default() -> Self {
        Self::ZERO
    }
}

//
// Add helper calls for Authority Signature
//

pub trait SuiAuthoritySignature {
    fn verify_secure<T>(
        &self,
        value: &IntentMessage<T>,
        epoch_id: EpochId,
        author: AuthorityPublicKeyBytes,
    ) -> Result<(), SuiError>
    where
        T: Serialize;

    fn new_secure<T>(
        value: &IntentMessage<T>,
        epoch_id: &EpochId,
        secret: &dyn Signer<Self>,
    ) -> Self
    where
        T: Serialize;
}

impl SuiAuthoritySignature for AuthoritySignature {
    fn new_secure<T>(value: &IntentMessage<T>, epoch: &EpochId, secret: &dyn Signer<Self>) -> Self
    where
        T: Serialize,
    {
        let mut intent_msg_bytes =
            bcs::to_bytes(&value).expect("Message serialization should not fail");
        epoch.write(&mut intent_msg_bytes);
        secret.sign(&intent_msg_bytes)
    }

    fn verify_secure<T>(
        &self,
        _value: &IntentMessage<T>,
        _epoch: EpochId,
        _author: AuthorityPublicKeyBytes,
    ) -> Result<(), SuiError>
    where
        T: Serialize,
    {
        // let mut message = bcs::to_bytes(&value).expect("Message serialization should not fail");
        // epoch.write(&mut message);

        // let public_key = AuthorityPublicKey::try_from(author).map_err(|_| {
        //     SuiError::KeyConversionError(
        //         "Failed to serialize public key bytes to valid public key".to_string(),
        //     )
        // })?;
        // public_key
        //     .verify(&message[..], self)
        //     .map_err(|e| SuiError::InvalidSignature {
        //         error: format!(
        //             "Fail to verify auth sig {} epoch: {} author: {}",
        //             e,
        //             epoch,
        //             author.concise()
        //         ),
        //     })
        todo!()
    }
}

// Enums for signature scheme signatures
#[enum_dispatch]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Signature {
    Ed25519SuiSignature,
    Secp256k1SuiSignature,
    Secp256r1SuiSignature,
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.as_ref();

        if serializer.is_human_readable() {
            let s = Base64::encode(bytes);
            serializer.serialize_str(&s)
        } else {
            serializer.serialize_bytes(bytes)
        }
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let bytes = if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Base64::decode(&s).map_err(|e| Error::custom(e.to_string()))?
        } else {
            let data: Vec<u8> = Vec::deserialize(deserializer)?;
            data
        };

        // Self::from_bytes(&bytes).map_err(|e| Error::custom(e.to_string()))
        todo!()
    }
}

impl Signature {
    /// The messaged passed in is already hashed form.
    pub fn new_hashed(hashed_msg: &[u8], secret: &dyn Signer<Signature>) -> Self {
        Signer::sign(secret, hashed_msg)
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        match self {
            Signature::Ed25519SuiSignature(sig) => sig.as_ref(),
            Signature::Secp256k1SuiSignature(sig) => sig.as_ref(),
            Signature::Secp256r1SuiSignature(sig) => sig.as_ref(),
        }
    }
}
impl AsMut<[u8]> for Signature {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Signature::Ed25519SuiSignature(sig) => sig.as_mut(),
            Signature::Secp256k1SuiSignature(sig) => sig.as_mut(),
            Signature::Secp256r1SuiSignature(sig) => sig.as_mut(),
        }
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct Ed25519SuiSignature(
    #[serde_as(as = "Readable<Base64, Bytes>")]
    [u8; Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1],
);

// Implementation useful for simplify testing when mock signature is needed
impl Default for Ed25519SuiSignature {
    fn default() -> Self {
        Self([0; Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1])
    }
}

impl Ed25519SuiSignature {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        if bytes.len() != 97 {
            return Err(FastCryptoError::InputLengthWrong(97));
        }
        let mut sig_bytes = [0; 97];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

//
// Secp256k1 Sui Signature port
//
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct Secp256k1SuiSignature(
    #[serde_as(as = "Readable<Base64, Bytes>")]
    [u8; Secp256k1PublicKey::LENGTH + Secp256k1Signature::LENGTH + 1],
);
//
// Secp256r1 Sui Signature port
//
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct Secp256r1SuiSignature(
    #[serde_as(as = "Readable<Base64, Bytes>")]
    [u8; Secp256r1PublicKey::LENGTH + Secp256r1Signature::LENGTH + 1],
);

pub trait SuiPublicKey: VerifyingKey {
    const SIGNATURE_SCHEME: SignatureScheme;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EmptySignInfo {}

#[derive(Clone, Debug, Eq, Serialize, Deserialize)]
pub struct AuthoritySignInfo {
    pub epoch: EpochId,
    pub authority: AuthorityName,
    pub signature: AuthoritySignature,
}

impl AuthoritySignInfo {
    pub fn new<T>(
        epoch: EpochId,
        value: &T,
        intent: Intent,
        name: AuthorityName,
        secret: &dyn Signer<AuthoritySignature>,
    ) -> Self
    where
        T: Serialize,
    {
        Self {
            epoch,
            authority: name,
            signature: AuthoritySignature::new_secure(
                &IntentMessage::new(intent, value),
                &epoch,
                secret,
            ),
        }
    }
}

impl Hash for AuthoritySignInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.epoch.hash(state);
        self.authority.hash(state);
    }
}

impl Display for AuthoritySignInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AuthoritySignInfo {{ epoch: {:?}, authority: {} }}",
            self.epoch, self.authority,
        )
    }
}

impl PartialEq for AuthoritySignInfo {
    fn eq(&self, other: &Self) -> bool {
        // We do not compare the signature, because there can be multiple
        // valid signatures for the same epoch and authority.
        self.epoch == other.epoch && self.authority == other.authority
    }
}

/// Something that we know how to hash and sign.
pub trait Signable<W> {
    fn write(&self, writer: &mut W);
}

pub trait SignableBytes
where
    Self: Sized,
{
    fn from_signable_bytes(bytes: &[u8]) -> Result<Self, Error>;
}

impl<W> Signable<W> for EpochId
where
    W: std::io::Write,
{
    fn write(&self, writer: &mut W) {
        bcs::serialize_into(writer, &self).expect("Message serialization should not fail");
    }
}

fn hash<S: Signable<H>, H: HashFunction<DIGEST_SIZE>, const DIGEST_SIZE: usize>(
    signable: &S,
) -> [u8; DIGEST_SIZE] {
    let mut digest = H::default();
    signable.write(&mut digest);
    let hash = digest.finalize();
    hash.into()
}

pub fn default_hash<S: Signable<DefaultHash>>(signable: &S) -> [u8; 32] {
    // hash::<S, DefaultHash, 32>(signable)
    todo!()
}

#[derive(
    Clone, Copy, Deserialize, Serialize, Debug, EnumString, strum_macros::Display, PartialEq, Eq,
)]
#[strum(serialize_all = "lowercase")]
pub enum SignatureScheme {
    ED25519,
    Secp256k1,
    Secp256r1,
    BLS12381, // This is currently not supported for user Sui Address.
    MultiSig,
    ZkLoginAuthenticator,
    PasskeyAuthenticator,
}

impl SignatureScheme {
    pub fn flag(&self) -> u8 {
        match self {
            SignatureScheme::ED25519 => 0x00,
            SignatureScheme::Secp256k1 => 0x01,
            SignatureScheme::Secp256r1 => 0x02,
            SignatureScheme::MultiSig => 0x03,
            SignatureScheme::BLS12381 => 0x04, // This is currently not supported for user Sui Address.
            SignatureScheme::ZkLoginAuthenticator => 0x05,
            SignatureScheme::PasskeyAuthenticator => 0x06,
        }
    }

    pub fn from_flag(flag: &str) -> Result<SignatureScheme, SuiError> {
        let byte_int = flag
            .parse::<u8>()
            .map_err(|_| SuiError::KeyConversionError("Invalid key scheme".to_string()))?;
        Self::from_flag_byte(&byte_int)
    }

    pub fn from_flag_byte(byte_int: &u8) -> Result<SignatureScheme, SuiError> {
        match byte_int {
            0x00 => Ok(SignatureScheme::ED25519),
            0x01 => Ok(SignatureScheme::Secp256k1),
            0x02 => Ok(SignatureScheme::Secp256r1),
            0x03 => Ok(SignatureScheme::MultiSig),
            0x04 => Ok(SignatureScheme::BLS12381),
            0x05 => Ok(SignatureScheme::ZkLoginAuthenticator),
            0x06 => Ok(SignatureScheme::PasskeyAuthenticator),
            _ => Err(SuiError::KeyConversionError(
                "Invalid key scheme".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZkLoginAuthenticatorAsBytes(pub Vec<u8>);

impl FromStr for GenericSignature {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Self::decode_base64(s).map_err(|e| eyre!("Fail to decode base64 {}", e.to_string()))
        todo!()
    }
}

//
// Types for randomness generation
//
// pub type RandomnessSignature = fastcrypto_tbls::types::Signature;
// pub type RandomnessPartialSignature = fastcrypto_tbls::tbls::PartialSignature<RandomnessSignature>;
// pub type RandomnessPrivateKey =
//     fastcrypto_tbls::ecies_v1::PrivateKey<fastcrypto::groups::bls12381::G2Element>;

/// Round number of generated randomness.
#[derive(Clone, Copy, Hash, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RandomnessRound(pub u64);

impl Display for RandomnessRound {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add for RandomnessRound {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Add<u64> for RandomnessRound {
    type Output = Self;
    fn add(self, other: u64) -> Self {
        Self(self.0 + other)
    }
}

impl std::ops::Sub for RandomnessRound {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl std::ops::Sub<u64> for RandomnessRound {
    type Output = Self;
    fn sub(self, other: u64) -> Self {
        Self(self.0 - other)
    }
}

impl RandomnessRound {
    pub fn new(round: u64) -> Self {
        Self(round)
    }

    pub fn checked_add(self, rhs: u64) -> Option<Self> {
        self.0.checked_add(rhs).map(Self)
    }

    pub fn signature_message(&self) -> Vec<u8> {
        "random_beacon round "
            .as_bytes()
            .iter()
            .cloned()
            .chain(bcs::to_bytes(&self.0).expect("serialization should not fail"))
            .collect()
    }
}

/// Key to identify a JWK, consists of iss and kid.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct JwkId {
    /// iss string that identifies the OIDC provider.
    pub iss: String,
    /// kid string that identifies the JWK.
    pub kid: String,
}

impl JwkId {
    /// Create a new JwkId.
    pub fn new(iss: String, kid: String) -> Self {
        // if a Microsoft iss is found, remove the tenant id from it
        if match_micrsoft_iss_substring(&iss) {
            return Self {
                iss: "https://login.microsoftonline.com/v2.0".to_string(),
                kid,
            };
        }
        Self { iss, kid }
    }
}

/// Struct that contains info for a JWK. A list of them for different kids can
/// be retrieved from the JWK endpoint (e.g. <https://www.googleapis.com/oauth2/v3/certs>).
/// The JWK is used to verify the JWT token.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct JWK {
    /// Key type parameter, https://datatracker.ietf.org/doc/html/rfc7517#section-4.1
    pub kty: String,
    /// RSA public exponent, https://datatracker.ietf.org/doc/html/rfc7517#section-9.3
    pub e: String,
    /// RSA modulus, https://datatracker.ietf.org/doc/html/rfc7517#section-9.3
    pub n: String,
    /// Algorithm parameter, https://datatracker.ietf.org/doc/html/rfc7517#section-4.4
    pub alg: String,
}

/// The provider config consists of iss string and jwk endpoint.
#[derive(Debug)]
pub struct ProviderConfig {
    /// iss string that identifies the OIDC provider.
    pub iss: String,
    /// The JWK url string for the given provider.
    pub jwk_endpoint: String,
}

impl ProviderConfig {
    /// Create a new provider config.
    pub fn new(iss: &str, jwk_endpoint: &str) -> Self {
        Self {
            iss: iss.to_string(),
            jwk_endpoint: jwk_endpoint.to_string(),
        }
    }
}

/// Supported OIDC providers.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OIDCProvider {
    /// See https://accounts.google.com/.well-known/openid-configuration
    Google,
    /// See https://id.twitch.tv/oauth2/.well-known/openid-configuration
    Twitch,
    /// See https://www.facebook.com/.well-known/openid-configuration/
    Facebook,
    /// See https://kauth.kakao.com/.well-known/openid-configuration
    Kakao,
    /// See https://appleid.apple.com/.well-known/openid-configuration
    Apple,
    /// See https://slack.com/.well-known/openid-configuration
    Slack,
    /// This is a test issuer maintained by Mysten that will return a JWT non-interactively.
    /// See https://login.microsoftonline.com/common/v2.0/.well-known/openid-configuration
    Microsoft,
    /// Example: https://cognito-idp.us-east-1.amazonaws.com/us-east-1_LPSLCkC3A/.well-known/jwks.json
    AwsTenant((String, String)),
    /// https://accounts.karrier.one/.well-known/openid-configuration
    KarrierOne,
    /// https://accounts.credenza3.com/openid-configuration
    Credenza3,
    /// This is a test issuer that will return a JWT non-interactively.
    TestIssuer,
    /// https://oauth2.playtron.one/.well-known/jwks.json
    Playtron,
    /// https://auth.3dos.io/.well-known/openid-configuration
    Threedos,
    /// https://login.onepassport.onefc.com/de3ee5c1-5644-4113-922d-e8336569a462/b2c_1a_prod_signupsignin_onesuizklogin/v2.0/.well-known/openid-configuration
    Onefc,
    /// https://accounts.fantv.world/.well-known/openid-configuration
    FanTV,
    /// https://api.arden.cc/auth/jwks
    Arden,
}

impl ToString for OIDCProvider {
    fn to_string(&self) -> String {
        match self {
            Self::Google => "Google".to_string(),
            Self::Twitch => "Twitch".to_string(),
            Self::Facebook => "Facebook".to_string(),
            Self::Kakao => "Kakao".to_string(),
            Self::Apple => "Apple".to_string(),
            Self::Slack => "Slack".to_string(),
            Self::TestIssuer => "TestIssuer".to_string(),
            Self::Microsoft => "Microsoft".to_string(),
            Self::KarrierOne => "KarrierOne".to_string(),
            Self::Credenza3 => "Credenza3".to_string(),
            Self::Playtron => "Playtron".to_string(),
            Self::Threedos => "Threedos".to_string(),
            Self::Onefc => "Onefc".to_string(),
            Self::FanTV => "FanTV".to_string(),
            Self::Arden => "Arden".to_string(),
            Self::AwsTenant((region, tenant_id)) => {
                format!("AwsTenant-region:{}-tenant_id:{}", region, tenant_id)
            }
        }
    }
}

impl OIDCProvider {
    /// Returns the provider config consisting of iss and jwk endpoint.
    pub fn get_config(&self) -> ProviderConfig {
        match self {
            OIDCProvider::Google => ProviderConfig::new(
                "https://accounts.google.com",
                "https://www.googleapis.com/oauth2/v2/certs",
            ),
            OIDCProvider::Twitch => ProviderConfig::new(
                "https://id.twitch.tv/oauth2",
                "https://id.twitch.tv/oauth2/keys",
            ),
            OIDCProvider::Facebook => ProviderConfig::new(
                "https://www.facebook.com",
                "https://www.facebook.com/.well-known/oauth/openid/jwks/",
            ),
            OIDCProvider::Kakao => ProviderConfig::new(
                "https://kauth.kakao.com",
                "https://kauth.kakao.com/.well-known/jwks.json",
            ),
            OIDCProvider::Apple => ProviderConfig::new(
                "https://appleid.apple.com",
                "https://appleid.apple.com/auth/keys",
            ),
            OIDCProvider::Slack => {
                ProviderConfig::new("https://slack.com", "https://slack.com/openid/connect/keys")
            }
            OIDCProvider::Microsoft => ProviderConfig::new(
                "https://login.microsoftonline.com/v2.0",
                "https://login.microsoftonline.com/common/discovery/v2.0/keys",
            ),
            OIDCProvider::AwsTenant((region, tenant_id)) => ProviderConfig::new(
                &format!("https://cognito-idp.{}.amazonaws.com/{}", region, tenant_id),
                &format!(
                    "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
                    region, tenant_id
                ),
            ),
            OIDCProvider::KarrierOne => ProviderConfig::new(
                "https://accounts.karrier.one/",
                "https://accounts.karrier.one/.well-known/jwks",
            ),
            OIDCProvider::Credenza3 => ProviderConfig::new(
                "https://accounts.credenza3.com",
                "https://accounts.credenza3.com/jwks",
            ),
            OIDCProvider::TestIssuer => ProviderConfig::new(
                "https://oauth.sui.io",
                "https://jwt-tester.mystenlabs.com/.well-known/jwks.json",
            ),
            OIDCProvider::Playtron => ProviderConfig::new(
                "https://oauth2.playtron.one",
                "https://oauth2.playtron.one/.well-known/jwks.json",
            ),
            OIDCProvider::Threedos => ProviderConfig::new(
                "https://auth.3dos.io",
                "https://auth.3dos.io/.well-known/jwks.json",
            ),
            OIDCProvider::Onefc => ProviderConfig::new(
                "https://login.onepassport.onefc.com/de3ee5c1-5644-4113-922d-e8336569a462/v2.0/",
                "https://login.onepassport.onefc.com/de3ee5c1-5644-4113-922d-e8336569a462/b2c_1a_prod_signupsignin_onesuizklogin/discovery/v2.0/keys",
            ),
            OIDCProvider::FanTV => ProviderConfig::new(
                "https://accounts.fantv.world",
                "https://fantv-apis.fantiger.com/v1/web3/jwks.json",
            ),
            OIDCProvider::Arden => ProviderConfig::new(
                "https://oidc.arden.cc",
                "https://api.arden.cc/auth/jwks",
            ),
        }
    }
}

/// Check if the iss string is formatted as Microsoft's pattern.
fn match_micrsoft_iss_substring(iss: &str) -> bool {
    iss.starts_with("https://login.microsoftonline.com/") && iss.ends_with("/v2.0")
}

/// Enum to specify the environment to use for verifying keys.
#[derive(Serialize, Clone, Deserialize, Debug, Eq, PartialEq, Copy)]
pub enum ZkLoginEnv {
    /// Use the secure global verifying key derived from ceremony.
    Prod,
    /// Use the insecure global verifying key.
    Test,
}

impl Default for ZkLoginEnv {
    fn default() -> Self {
        Self::Prod
    }
}
