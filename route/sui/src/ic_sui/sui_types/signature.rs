use crate::ic_sui::sui_types::{base_types::SuiAddress, crypto::Signature, error::SuiResult};
pub use enum_dispatch::enum_dispatch;

use serde::{Deserialize, Serialize};

use crate::ic_sui::sui_types::crypto::{JwkId, OIDCProvider, ZkLoginEnv, JWK};
use im::hashmap::HashMap as ImHashMap;

use std::hash::Hash;

use crate::ic_sui::shared_inent::intent::IntentMessage;

use super::transaction::EpochId;

#[derive(Default, Debug, Clone)]
pub struct VerifyParams {
    // map from JwkId (iss, kid) => JWK
    pub oidc_provider_jwks: ImHashMap<JwkId, JWK>,
    pub supported_providers: Vec<OIDCProvider>,
    pub zk_login_env: ZkLoginEnv,
    pub verify_legacy_zklogin_address: bool,
    pub accept_zklogin_in_multisig: bool,
}

impl VerifyParams {
    pub fn new(
        oidc_provider_jwks: ImHashMap<JwkId, JWK>,
        supported_providers: Vec<OIDCProvider>,
        zk_login_env: ZkLoginEnv,
        verify_legacy_zklogin_address: bool,
        accept_zklogin_in_multisig: bool,
    ) -> Self {
        Self {
            oidc_provider_jwks,
            supported_providers,
            zk_login_env,
            verify_legacy_zklogin_address,
            accept_zklogin_in_multisig,
        }
    }
}

/// A lightweight trait that all members of [enum GenericSignature] implement.
#[enum_dispatch]
pub trait AuthenticatorTrait {
    fn verify_user_authenticator_epoch(&self, epoch: EpochId) -> SuiResult;

    fn verify_claims<T>(
        &self,
        value: &IntentMessage<T>,
        author: SuiAddress,
        aux_verify_data: &VerifyParams,
    ) -> SuiResult
    where
        T: Serialize;

    fn verify_authenticator<T>(
        &self,
        value: &IntentMessage<T>,
        author: SuiAddress,
        epoch: Option<EpochId>,
        aux_verify_data: &VerifyParams,
    ) -> SuiResult
    where
        T: Serialize,
    {
        if let Some(epoch) = epoch {
            self.verify_user_authenticator_epoch(epoch)?;
        }
        self.verify_claims(value, author, aux_verify_data)
    }

    fn verify_uncached_checks<T>(
        &self,
        value: &IntentMessage<T>,
        author: SuiAddress,
        aux_verify_data: &VerifyParams,
    ) -> SuiResult
    where
        T: Serialize;
}

/// Due to the incompatibility of [enum Signature] (which dispatches a trait that
/// assumes signature and pubkey bytes for verification), here we add a wrapper
/// enum where member can just implement a lightweight [trait AuthenticatorTrait].
/// This way MultiSig (and future Authenticators) can implement its own `verify`.
#[enum_dispatch(AuthenticatorTrait)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenericSignature {
    // MultiSig,
    // MultiSigLegacy,
    Signature,
    // ZkLoginAuthenticator,
}

impl GenericSignature {}

/// Trait useful to get the bytes reference for [enum GenericSignature].
impl AsRef<[u8]> for GenericSignature {
    fn as_ref(&self) -> &[u8] {
        match self {
            // GenericSignature::MultiSig(s) => todo!(),
            // GenericSignature::MultiSigLegacy(s) => todo!(),
            GenericSignature::Signature(s) => s.as_ref(),
            // GenericSignature::ZkLoginAuthenticator(s) => todo!(),
        }
    }
}

/// This ports the wrapper trait to the verify_secure defined on [enum Signature].
impl AuthenticatorTrait for Signature {
    fn verify_user_authenticator_epoch(&self, _: EpochId) -> SuiResult {
        Ok(())
    }
    fn verify_uncached_checks<T>(
        &self,
        _value: &IntentMessage<T>,
        _author: SuiAddress,
        _aux_verify_data: &VerifyParams,
    ) -> SuiResult
    where
        T: Serialize,
    {
        Ok(())
    }

    fn verify_claims<T>(
        &self,
        _value: &IntentMessage<T>,
        _author: SuiAddress,
        _aux_verify_data: &VerifyParams,
    ) -> SuiResult
    where
        T: Serialize,
    {
        // self.verify_secure(value, author, self.scheme())
        todo!()
    }
}
