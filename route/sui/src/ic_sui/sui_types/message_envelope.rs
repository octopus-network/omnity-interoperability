use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_name::{DeserializeNameAdapter, SerializeNameAdapter};

use crate::ic_sui::shared_inent::intent::IntentScope;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use super::crypto::EmptySignInfo;

pub trait Message {
    type DigestType: Clone + Debug;
    const SCOPE: IntentScope;

    fn scope(&self) -> IntentScope {
        Self::SCOPE
    }

    fn digest(&self) -> Self::DigestType;
}

#[derive(Clone, Debug, Eq, Serialize, Deserialize)]
#[serde(remote = "Envelope")]
pub struct Envelope<T: Message, S> {
    #[serde(skip)]
    digest: OnceCell<T::DigestType>,

    data: T,
    auth_signature: S,
}

impl<'de, T, S> Deserialize<'de> for Envelope<T, S>
where
    T: Message + Deserialize<'de>,
    S: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Envelope::deserialize(DeserializeNameAdapter::new(
            deserializer,
            std::any::type_name::<Self>(),
        ))
    }
}

impl<T, Sig> Serialize for Envelope<T, Sig>
where
    T: Message + Serialize,
    Sig: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        Envelope::serialize(
            self,
            SerializeNameAdapter::new(serializer, std::any::type_name::<Self>()),
        )
    }
}

impl<T: Message, S> Envelope<T, S> {
    pub fn new_from_data_and_sig(data: T, sig: S) -> Self {
        Self {
            digest: Default::default(),
            data,
            auth_signature: sig,
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn into_data(self) -> T {
        self.data
    }

    pub fn into_sig(self) -> S {
        self.auth_signature
    }

    pub fn into_data_and_sig(self) -> (T, S) {
        let Self {
            data,
            auth_signature,
            ..
        } = self;
        (data, auth_signature)
    }

    pub fn auth_sig(&self) -> &S {
        &self.auth_signature
    }

    pub fn auth_sig_mut_for_testing(&mut self) -> &mut S {
        &mut self.auth_signature
    }

    pub fn digest(&self) -> &T::DigestType {
        self.digest.get_or_init(|| self.data.digest())
    }

    pub fn data_mut_for_testing(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: Message + PartialEq, S: PartialEq> PartialEq for Envelope<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.auth_signature == other.auth_signature
    }
}

impl<T: Message> Envelope<T, EmptySignInfo> {
    pub fn new(data: T) -> Self {
        Self {
            digest: OnceCell::new(),
            data,
            auth_signature: EmptySignInfo {},
        }
    }
}

/// TrustedEnvelope is a serializable wrapper around Envelope which is
/// `Into<VerifiedEnvelope>` - in other words it models a verified message which has been
/// written to the db (or some other trusted store), and may be read back from the db without
/// further signature verification.
///
/// TrustedEnvelope should *only* appear in database interfaces.
///
/// DO NOT USE in networked APIs.
///
/// Because it is used very sparingly, it can be audited easily: Use rust-analyzer,
/// or run: git grep -E 'TrustedEnvelope'
///
/// And verify that none of the uses appear in any network APIs.
#[derive(Clone, Serialize, Deserialize)]
pub struct TrustedEnvelope<T: Message, S>(Envelope<T, S>);

impl<T, S: Debug> Debug for TrustedEnvelope<T, S>
where
    T: Message + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T: Message, S> TrustedEnvelope<T, S> {
    pub fn into_inner(self) -> Envelope<T, S> {
        self.0
    }

    pub fn inner(&self) -> &Envelope<T, S> {
        &self.0
    }
}

// An empty marker struct that can't be serialized.
#[derive(Clone)]
struct NoSer;
// Never remove this assert!
// static_assertions::assert_not_impl_any!(NoSer: Serialize, DeserializeOwned);

#[derive(Clone)]
pub struct VerifiedEnvelope<T: Message, S>(TrustedEnvelope<T, S>, NoSer);

impl<T, S: Debug> Debug for VerifiedEnvelope<T, S>
where
    T: Message + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0 .0)
    }
}

impl<T: Message, S> Deref for Envelope<T, S> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Message, S> DerefMut for Envelope<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
