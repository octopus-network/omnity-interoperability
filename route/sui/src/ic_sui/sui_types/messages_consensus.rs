use crate::ic_sui::sui_types::base_types::{AuthorityName, ObjectRef, TransactionDigest};
use crate::ic_sui::sui_types::base_types::{ObjectID, SequenceNumber};
use crate::ic_sui::sui_types::digests::ConsensusCommitDigest;

use crate::ic_sui::sui_types::transaction::Transaction;

use serde::{Deserialize, Serialize};

use std::fmt::Debug;
use std::hash::Hash;

use std::time::{SystemTime, UNIX_EPOCH};

use super::crypto::{JwkId, JWK};

use super::messages_checkpoint::{CheckpointSequenceNumber, CheckpointSignatureMessage};
use super::supported_protocol_versions::{
    SupportedProtocolVersions, SupportedProtocolVersionsWithHashes,
};

/// The index of an authority in the consensus committee.
/// The value should be the same in Sui committee.
pub type AuthorityIndex = u32;

/// Consensus round number.
pub type Round = u32;

/// The index of a transaction in a consensus block.
pub type TransactionIndex = u16;

/// Non-decreasing timestamp produced by consensus in ms.
pub type TimestampMs = u64;

/// Only commit_timestamp_ms is passed to the move call currently.
/// However we include epoch and round to make sure each ConsensusCommitPrologue has a unique tx digest.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct ConsensusCommitPrologue {
    /// Epoch of the commit prologue transaction
    pub epoch: u64,
    /// Consensus round of the commit. Using u64 for compatibility.
    pub round: u64,
    /// Unix timestamp from consensus commit.
    pub commit_timestamp_ms: TimestampMs,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct ConsensusCommitPrologueV2 {
    /// Epoch of the commit prologue transaction
    pub epoch: u64,
    /// Consensus round of the commit
    pub round: u64,
    /// Unix timestamp from consensus commit.
    pub commit_timestamp_ms: TimestampMs,
    /// Digest of consensus output
    pub consensus_commit_digest: ConsensusCommitDigest,
}

/// Uses an enum to allow for future expansion of the ConsensusDeterminedVersionAssignments.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum ConsensusDeterminedVersionAssignments {
    // Cancelled transaction version assignment.
    CancelledTransactions(Vec<(TransactionDigest, Vec<(ObjectID, SequenceNumber)>)>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct ConsensusCommitPrologueV3 {
    /// Epoch of the commit prologue transaction
    pub epoch: u64,
    /// Consensus round of the commit
    pub round: u64,
    /// The sub DAG index of the consensus commit. This field will be populated if there
    /// are multiple consensus commits per round.
    pub sub_dag_index: Option<u64>,
    /// Unix timestamp from consensus commit.
    pub commit_timestamp_ms: TimestampMs,
    /// Digest of consensus output
    pub consensus_commit_digest: ConsensusCommitDigest,
    /// Stores consensus handler determined shared object version assignments.
    pub consensus_determined_version_assignments: ConsensusDeterminedVersionAssignments,
}

// In practice, JWKs are about 500 bytes of json each, plus a bit more for the ID.
// 4096 should give us plenty of space for any imaginable JWK while preventing DoSes.
static MAX_TOTAL_JWK_SIZE: usize = 4096;

pub fn check_total_jwk_size(id: &JwkId, jwk: &JWK) -> bool {
    id.iss.len() + id.kid.len() + jwk.kty.len() + jwk.alg.len() + jwk.e.len() + jwk.n.len()
        <= MAX_TOTAL_JWK_SIZE
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConsensusTransaction {
    /// Encodes an u64 unique tracking id to allow us trace a message between Sui and consensus.
    /// Use an byte array instead of u64 to ensure stable serialization.
    pub tracking_id: [u8; 8],
    pub kind: ConsensusTransactionKind,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum ConsensusTransactionKey {
    Certificate(TransactionDigest),
    CheckpointSignature(AuthorityName, CheckpointSequenceNumber),
    EndOfPublish(AuthorityName),
    CapabilityNotification(AuthorityName, u64 /* generation */),
    // Key must include both id and jwk, because honest validators could be given multiple jwks for
    // the same id by malfunctioning providers.
    NewJWKFetched(Box<(AuthorityName, JwkId, JWK)>),
    RandomnessDkgMessage(AuthorityName),
    RandomnessDkgConfirmation(AuthorityName),
}

/// Used to advertise capabilities of each authority via consensus. This allows validators to
/// negotiate the creation of the ChangeEpoch transaction.
#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct AuthorityCapabilitiesV1 {
    /// Originating authority - must match consensus transaction source.
    pub authority: AuthorityName,
    /// Generation number set by sending authority. Used to determine which of multiple
    /// AuthorityCapabilities messages from the same authority is the most recent.
    ///
    /// (Currently, we just set this to the current time in milliseconds since the epoch, but this
    /// should not be interpreted as a timestamp.)
    pub generation: u64,

    /// ProtocolVersions that the authority supports.
    pub supported_protocol_versions: SupportedProtocolVersions,

    /// The ObjectRefs of all versions of system packages that the validator possesses.
    /// Used to determine whether to do a framework/movestdlib upgrade.
    pub available_system_packages: Vec<ObjectRef>,
}

impl AuthorityCapabilitiesV1 {
    pub fn new(
        authority: AuthorityName,
        supported_protocol_versions: SupportedProtocolVersions,
        available_system_packages: Vec<ObjectRef>,
    ) -> Self {
        let generation = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Sui did not exist prior to 1970")
            .as_millis()
            .try_into()
            .expect("This build of sui is not supported in the year 500,000,000");
        Self {
            authority,
            generation,
            supported_protocol_versions,
            available_system_packages,
        }
    }
}

/// Used to advertise capabilities of each authority via consensus. This allows validators to
/// negotiate the creation of the ChangeEpoch transaction.
#[derive(Serialize, Deserialize, Clone, Hash)]
pub struct AuthorityCapabilitiesV2 {
    /// Originating authority - must match transaction source authority from consensus.
    pub authority: AuthorityName,
    /// Generation number set by sending authority. Used to determine which of multiple
    /// AuthorityCapabilities messages from the same authority is the most recent.
    ///
    /// (Currently, we just set this to the current time in milliseconds since the epoch, but this
    /// should not be interpreted as a timestamp.)
    pub generation: u64,

    /// ProtocolVersions that the authority supports.
    pub supported_protocol_versions: SupportedProtocolVersionsWithHashes,

    /// The ObjectRefs of all versions of system packages that the validator possesses.
    /// Used to determine whether to do a framework/movestdlib upgrade.
    pub available_system_packages: Vec<ObjectRef>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ConsensusTransactionKind {
    // CertifiedTransaction(Box<CertifiedTransaction>),
    CheckpointSignature(Box<CheckpointSignatureMessage>),
    EndOfPublish(AuthorityName),

    CapabilityNotification(AuthorityCapabilitiesV1),

    NewJWKFetched(AuthorityName, JwkId, JWK),
    RandomnessStateUpdate(u64, Vec<u8>), // deprecated
    // DKG is used to generate keys for use in the random beacon protocol.
    // `RandomnessDkgMessage` is sent out at start-of-epoch to initiate the process.
    // Contents are a serialized `fastcrypto_tbls::dkg::Message`.
    RandomnessDkgMessage(AuthorityName, Vec<u8>),
    // `RandomnessDkgConfirmation` is the second DKG message, sent as soon as a threshold amount of
    // `RandomnessDkgMessages` have been received locally, to complete the key generation process.
    // Contents are a serialized `fastcrypto_tbls::dkg::Confirmation`.
    RandomnessDkgConfirmation(AuthorityName, Vec<u8>),

    CapabilityNotificationV2(AuthorityCapabilitiesV2),

    UserTransaction(Box<Transaction>),
}

impl ConsensusTransactionKind {
    pub fn is_dkg(&self) -> bool {
        matches!(
            self,
            ConsensusTransactionKind::RandomnessDkgMessage(_, _)
                | ConsensusTransactionKind::RandomnessDkgConfirmation(_, _)
        )
    }
}
