use super::base_types::{AuthorityName, ExecutionDigests};
use super::transaction::{EpochId, ProtocolVersion, StakeUnit};
use crate::ic_sui::shared_inent::intent::IntentScope;
use crate::ic_sui::sui_types::crypto::{AuthoritySignInfo, RandomnessRound};
pub use crate::ic_sui::sui_types::digests::CheckpointContentsDigest;
pub use crate::ic_sui::sui_types::digests::CheckpointDigest;
use crate::ic_sui::sui_types::digests::Digest;
use crate::ic_sui::sui_types::gas::GasCostSummary;
use crate::ic_sui::sui_types::message_envelope::{Envelope, Message};
use crate::ic_sui::sui_types::signature::GenericSignature;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
pub type CheckpointSequenceNumber = u64;
pub type CheckpointTimestamp = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointRequest {
    /// if a sequence number is specified, return the checkpoint with that sequence number;
    /// otherwise if None returns the latest authenticated checkpoint stored.
    pub sequence_number: Option<CheckpointSequenceNumber>,
    // A flag, if true also return the contents of the
    // checkpoint besides the meta-data.
    pub request_content: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointRequestV2 {
    /// if a sequence number is specified, return the checkpoint with that sequence number;
    /// otherwise if None returns the latest checkpoint stored (authenticated or pending,
    /// depending on the value of `certified` flag)
    pub sequence_number: Option<CheckpointSequenceNumber>,
    // A flag, if true also return the contents of the
    // checkpoint besides the meta-data.
    pub request_content: bool,
    // If true, returns certified checkpoint, otherwise returns pending checkpoint
    pub certified: bool,
}

// The Sha256 digest of an EllipticCurveMultisetHash committing to the live object set.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ECMHLiveObjectSetDigest {
    pub digest: Digest,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckpointCommitment {
    ECMHLiveObjectSetDigest(ECMHLiveObjectSetDigest),
    // Other commitment types (e.g. merkle roots) go here.
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EndOfEpochData {
    /// next_epoch_committee is `Some` if and only if the current checkpoint is
    /// the last checkpoint of an epoch.
    /// Therefore next_epoch_committee can be used to pick the last checkpoint of an epoch,
    /// which is often useful to get epoch level summary stats like total gas cost of an epoch,
    /// or the total number of transactions from genesis to the end of an epoch.
    /// The committee is stored as a vector of validator pub key and stake pairs. The vector
    /// should be sorted based on the Committee data structure.
    pub next_epoch_committee: Vec<(AuthorityName, StakeUnit)>,

    /// The protocol version that is in effect during the epoch that starts immediately after this
    /// checkpoint.
    pub next_epoch_protocol_version: ProtocolVersion,

    /// Commitments to epoch specific state (e.g. live object set)
    pub epoch_commitments: Vec<CheckpointCommitment>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointSummary {
    pub epoch: EpochId,
    pub sequence_number: CheckpointSequenceNumber,
    /// Total number of transactions committed since genesis, including those in this
    /// checkpoint.
    pub network_total_transactions: u64,
    pub content_digest: CheckpointContentsDigest,
    pub previous_digest: Option<CheckpointDigest>,
    /// The running total gas costs of all transactions included in the current epoch so far
    /// until this checkpoint.
    pub epoch_rolling_gas_cost_summary: GasCostSummary,

    /// Timestamp of the checkpoint - number of milliseconds from the Unix epoch
    /// Checkpoint timestamps are monotonic, but not strongly monotonic - subsequent
    /// checkpoints can have same timestamp if they originate from the same underlining consensus commit
    pub timestamp_ms: CheckpointTimestamp,

    /// Commitments to checkpoint-specific state (e.g. txns in checkpoint, objects read/written in
    /// checkpoint).
    pub checkpoint_commitments: Vec<CheckpointCommitment>,

    /// Present only on the final checkpoint of the epoch.
    pub end_of_epoch_data: Option<EndOfEpochData>,

    /// CheckpointSummary is not an evolvable structure - it must be readable by any version of the
    /// code. Therefore, in order to allow extensions to be added to CheckpointSummary, we allow
    /// opaque data to be added to checkpoints which can be deserialized based on the current
    /// protocol version.
    ///
    /// This is implemented with BCS-serialized `CheckpointVersionSpecificData`.
    pub version_specific_data: Vec<u8>,
}

impl Message for CheckpointSummary {
    type DigestType = CheckpointDigest;
    const SCOPE: IntentScope = IntentScope::CheckpointSummary;

    fn digest(&self) -> Self::DigestType {
        // CheckpointDigest::new(default_hash(self))
        todo!()
    }
}
// Checkpoints are signed by an authority and 2f+1 form a
// certificate that others can use to catch up. The actual
// content of the digest must at the very least commit to
// the set of transactions contained in the certificate but
// we might extend this to contain roots of merkle trees,
// or other authenticated data structures to support light
// clients and more efficient sync protocols.

pub type CheckpointSummaryEnvelope<S> = Envelope<CheckpointSummary, S>;
// pub type CertifiedCheckpointSummary = CheckpointSummaryEnvelope<AuthorityStrongQuorumSignInfo>;
pub type SignedCheckpointSummary = CheckpointSummaryEnvelope<AuthoritySignInfo>;

/// This is a message validators publish to consensus in order to sign checkpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointSignatureMessage {
    pub summary: SignedCheckpointSummary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointContents {
    V1(CheckpointContentsV1),
}

/// CheckpointContents are the transactions included in an upcoming checkpoint.
/// They must have already been causally ordered. Since the causal order algorithm
/// is the same among validators, we expect all honest validators to come up with
/// the same order for each checkpoint content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointContentsV1 {
    #[serde(skip)]
    digest: OnceCell<CheckpointContentsDigest>,

    transactions: Vec<ExecutionDigests>,
    /// This field 'pins' user signatures for the checkpoint
    /// The length of this vector is same as length of transactions vector
    /// System transactions has empty signatures
    user_signatures: Vec<Vec<GenericSignature>>,
}

/// Holds data in CheckpointSummary that is serialized into the `version_specific_data` field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointVersionSpecificData {
    V1(CheckpointVersionSpecificDataV1),
}

impl CheckpointVersionSpecificData {
    pub fn as_v1(&self) -> &CheckpointVersionSpecificDataV1 {
        match self {
            Self::V1(v) => v,
        }
    }

    pub fn into_v1(self) -> CheckpointVersionSpecificDataV1 {
        match self {
            Self::V1(v) => v,
        }
    }

    pub fn empty_for_tests() -> CheckpointVersionSpecificData {
        CheckpointVersionSpecificData::V1(CheckpointVersionSpecificDataV1 {
            randomness_rounds: Vec::new(),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointVersionSpecificDataV1 {
    /// Lists the rounds for which RandomnessStateUpdate transactions are present in the checkpoint.
    pub randomness_rounds: Vec<RandomnessRound>,
}
