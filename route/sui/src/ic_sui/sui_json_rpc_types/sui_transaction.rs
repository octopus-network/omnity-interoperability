#![allow(unused)]

pub use enum_dispatch::enum_dispatch;
use std::fmt::{self, Display, Formatter};

use crate::ic_sui::fastcrypto::encoding::Base64;
use crate::ic_sui::sui_json::SuiJsonValue;
use crate::ic_sui::sui_types::base_types::{ObjectID, SequenceNumber};
use crate::ic_sui::sui_types::digests::{
    CheckpointDigest, ConsensusCommitDigest, ObjectDigest, TransactionEventsDigest,
};
use crate::ic_sui::sui_types::gas::GasCostSummary;
use crate::ic_sui::sui_types::messages_consensus::ConsensusDeterminedVersionAssignments;
use crate::ic_sui::sui_types::object::Owner;
use crate::ic_sui::sui_types::storage::{DeleteKind, WriteKind};
use crate::ic_sui::sui_types::transaction::EpochId;
use crate::ic_sui::sui_types::TypeTag;
use crate::ic_sui::sui_types::{
    base_types::SuiAddress, digests::TransactionDigest,
    messages_checkpoint::CheckpointSequenceNumber,
    quorum_driver_types::ExecuteTransactionRequestType,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::sui_object::SuiObjectRef;
use super::ObjectChange;
use super::{BalanceChange, SuiEvent};

use crate::ic_sui::sui_types::sui_serde::{
    BigInt, SequenceNumber as AsSequenceNumber, SuiTypeTag as AsSuiTypeTag,
};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, Default)]
#[serde(
    rename_all = "camelCase",
    rename = "TransactionBlockResponseOptions",
    default
)]
pub struct SuiTransactionBlockResponseOptions {
    /// Whether to show transaction input data. Default to be False
    pub show_input: bool,
    /// Whether to show bcs-encoded transaction input data
    pub show_raw_input: bool,
    /// Whether to show transaction effects. Default to be False
    pub show_effects: bool,
    /// Whether to show transaction events. Default to be False
    pub show_events: bool,
    /// Whether to show object_changes. Default to be False
    pub show_object_changes: bool,
    /// Whether to show balance_changes. Default to be False
    pub show_balance_changes: bool,
    /// Whether to show raw transaction effects. Default to be False
    pub show_raw_effects: bool,
}

impl SuiTransactionBlockResponseOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn full_content() -> Self {
        Self {
            show_effects: true,
            show_input: true,
            show_raw_input: true,
            show_events: true,
            show_object_changes: true,
            show_balance_changes: true,
            // This field is added for graphql execution. We keep it false here
            // so current users of `full_content` will not get raw effects unexpectedly.
            show_raw_effects: false,
        }
    }

    pub fn with_input(mut self) -> Self {
        self.show_input = true;
        self
    }

    pub fn with_raw_input(mut self) -> Self {
        self.show_raw_input = true;
        self
    }

    pub fn with_effects(mut self) -> Self {
        self.show_effects = true;
        self
    }

    pub fn with_events(mut self) -> Self {
        self.show_events = true;
        self
    }

    pub fn with_balance_changes(mut self) -> Self {
        self.show_balance_changes = true;
        self
    }

    pub fn with_object_changes(mut self) -> Self {
        self.show_object_changes = true;
        self
    }

    pub fn with_raw_effects(mut self) -> Self {
        self.show_raw_effects = true;
        self
    }

    /// default to return `WaitForEffectsCert` unless some options require
    /// local execution
    pub fn default_execution_request_type(&self) -> ExecuteTransactionRequestType {
        // if people want effects or events, they typically want to wait for local execution
        if self.require_effects() {
            ExecuteTransactionRequestType::WaitForLocalExecution
        } else {
            ExecuteTransactionRequestType::WaitForEffectsCert
        }
    }

    pub fn require_input(&self) -> bool {
        self.show_input || self.show_raw_input || self.show_object_changes
    }

    pub fn require_effects(&self) -> bool {
        self.show_effects
            || self.show_events
            || self.show_balance_changes
            || self.show_object_changes
            || self.show_raw_effects
    }

    pub fn only_digest(&self) -> bool {
        self == &Self::default()
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", rename = "TransactionBlockResponse")]
pub struct SuiTransactionBlockResponse {
    pub digest: TransactionDigest,
    /// Transaction input data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<SuiTransactionBlock>,
    /// BCS encoded [SenderSignedData] that includes input object references
    /// returns empty array if `show_raw_transaction` is false
    #[serde_as(as = "Base64")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_transaction: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<SuiTransactionBlockEffects>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<SuiTransactionBlockEvents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_changes: Option<Vec<ObjectChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_changes: Option<Vec<BalanceChange>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<BigInt<u64>>")]
    pub timestamp_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_local_execution: Option<bool>,
    /// The checkpoint number when this transaction was included and hence finalized.
    /// This is only returned in the read api, not in the transaction execution api.

    #[serde_as(as = "Option<BigInt<u64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<CheckpointSequenceNumber>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_effects: Vec<u8>,
}

impl SuiTransactionBlockResponse {
    pub fn new(digest: TransactionDigest) -> Self {
        Self {
            digest,
            ..Default::default()
        }
    }

    pub fn status_ok(&self) -> Option<bool> {
        self.effects.as_ref().map(|e| e.status().is_ok())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "TransactionBlockKind", tag = "kind")]
pub enum SuiTransactionBlockKind {
    /// A system transaction that will update epoch information on-chain.
    ChangeEpoch(SuiChangeEpoch),
    /// A system transaction used for initializing the initial state of the chain.
    Genesis(SuiGenesisTransaction),
    /// A system transaction marking the start of a series of transactions scheduled as part of a
    /// checkpoint
    ConsensusCommitPrologue(SuiConsensusCommitPrologue),
    /// A series of transactions where the results of one transaction can be used in future
    /// transactions
    ProgrammableTransaction(SuiProgrammableTransactionBlock),
    /// A transaction which updates global authenticator state
    AuthenticatorStateUpdate(SuiAuthenticatorStateUpdate),
    /// A transaction which updates global randomness state
    RandomnessStateUpdate(SuiRandomnessStateUpdate),
    /// The transaction which occurs only at the end of the epoch
    EndOfEpochTransaction(SuiEndOfEpochTransaction),
    ConsensusCommitPrologueV2(SuiConsensusCommitPrologueV2),
    ConsensusCommitPrologueV3(SuiConsensusCommitPrologueV3),
    // .. more transaction types go here
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiChangeEpoch {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: EpochId,
    #[serde_as(as = "BigInt<u64>")]
    pub storage_charge: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub computation_charge: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub storage_rebate: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub epoch_start_timestamp_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[enum_dispatch(SuiTransactionBlockEffectsAPI)]
#[serde(
    rename = "TransactionBlockEffects",
    rename_all = "camelCase",
    tag = "messageVersion"
)]
pub enum SuiTransactionBlockEffects {
    V1(SuiTransactionBlockEffectsV1),
}

#[enum_dispatch]
pub trait SuiTransactionBlockEffectsAPI {
    fn status(&self) -> &SuiExecutionStatus;
    fn into_status(self) -> SuiExecutionStatus;
    fn shared_objects(&self) -> &[SuiObjectRef];
    fn created(&self) -> &[OwnedObjectRef];
    fn mutated(&self) -> &[OwnedObjectRef];
    fn unwrapped(&self) -> &[OwnedObjectRef];
    fn deleted(&self) -> &[SuiObjectRef];
    fn unwrapped_then_deleted(&self) -> &[SuiObjectRef];
    fn wrapped(&self) -> &[SuiObjectRef];
    fn gas_object(&self) -> &OwnedObjectRef;
    fn events_digest(&self) -> Option<&TransactionEventsDigest>;
    fn dependencies(&self) -> &[TransactionDigest];
    fn executed_epoch(&self) -> EpochId;
    fn transaction_digest(&self) -> &TransactionDigest;
    fn gas_cost_summary(&self) -> &GasCostSummary;

    /// Return an iterator of mutated objects, but excluding the gas object.
    fn mutated_excluding_gas(&self) -> Vec<OwnedObjectRef>;
    fn modified_at_versions(&self) -> Vec<(ObjectID, SequenceNumber)>;
    fn all_changed_objects(&self) -> Vec<(&OwnedObjectRef, WriteKind)>;
    fn all_deleted_objects(&self) -> Vec<(&SuiObjectRef, DeleteKind)>;
}

#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(
    rename = "TransactionBlockEffectsModifiedAtVersions",
    rename_all = "camelCase"
)]
pub struct SuiTransactionBlockEffectsModifiedAtVersions {
    object_id: ObjectID,
    #[serde_as(as = "AsSequenceNumber")]
    sequence_number: SequenceNumber,
}

/// The response from processing a transaction or a certified transaction
#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "TransactionBlockEffectsV1", rename_all = "camelCase")]
pub struct SuiTransactionBlockEffectsV1 {
    /// The status of the execution
    pub status: SuiExecutionStatus,
    /// The epoch when this transaction was executed.

    #[serde_as(as = "BigInt<u64>")]
    pub executed_epoch: EpochId,
    pub gas_used: GasCostSummary,
    /// The version that every modified (mutated or deleted) object had before it was modified by
    /// this transaction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modified_at_versions: Vec<SuiTransactionBlockEffectsModifiedAtVersions>,
    /// The object references of the shared objects used in this transaction. Empty if no shared objects were used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_objects: Vec<SuiObjectRef>,
    /// The transaction digest
    pub transaction_digest: TransactionDigest,
    /// ObjectRef and owner of new objects created.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub created: Vec<OwnedObjectRef>,
    /// ObjectRef and owner of mutated objects, including gas object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutated: Vec<OwnedObjectRef>,
    /// ObjectRef and owner of objects that are unwrapped in this transaction.
    /// Unwrapped objects are objects that were wrapped into other objects in the past,
    /// and just got extracted out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unwrapped: Vec<OwnedObjectRef>,
    /// Object Refs of objects now deleted (the old refs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deleted: Vec<SuiObjectRef>,
    /// Object refs of objects previously wrapped in other objects but now deleted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unwrapped_then_deleted: Vec<SuiObjectRef>,
    /// Object refs of objects now wrapped in other objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wrapped: Vec<SuiObjectRef>,
    /// The updated gas object reference. Have a dedicated field for convenient access.
    /// It's also included in mutated.
    pub gas_object: OwnedObjectRef,
    /// The digest of the events emitted during execution,
    /// can be None if the transaction does not emit any event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_digest: Option<TransactionEventsDigest>,
    /// The set of transaction digests this transaction depends on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<TransactionDigest>,
}

impl SuiTransactionBlockEffectsAPI for SuiTransactionBlockEffectsV1 {
    fn status(&self) -> &SuiExecutionStatus {
        &self.status
    }
    fn into_status(self) -> SuiExecutionStatus {
        self.status
    }
    fn shared_objects(&self) -> &[SuiObjectRef] {
        &self.shared_objects
    }
    fn created(&self) -> &[OwnedObjectRef] {
        &self.created
    }
    fn mutated(&self) -> &[OwnedObjectRef] {
        &self.mutated
    }
    fn unwrapped(&self) -> &[OwnedObjectRef] {
        &self.unwrapped
    }
    fn deleted(&self) -> &[SuiObjectRef] {
        &self.deleted
    }
    fn unwrapped_then_deleted(&self) -> &[SuiObjectRef] {
        &self.unwrapped_then_deleted
    }
    fn wrapped(&self) -> &[SuiObjectRef] {
        &self.wrapped
    }
    fn gas_object(&self) -> &OwnedObjectRef {
        &self.gas_object
    }
    fn events_digest(&self) -> Option<&TransactionEventsDigest> {
        self.events_digest.as_ref()
    }
    fn dependencies(&self) -> &[TransactionDigest] {
        &self.dependencies
    }

    fn executed_epoch(&self) -> EpochId {
        self.executed_epoch
    }

    fn transaction_digest(&self) -> &TransactionDigest {
        &self.transaction_digest
    }

    fn gas_cost_summary(&self) -> &GasCostSummary {
        &self.gas_used
    }

    fn mutated_excluding_gas(&self) -> Vec<OwnedObjectRef> {
        self.mutated
            .iter()
            .filter(|o| *o != &self.gas_object)
            .cloned()
            .collect()
    }

    fn modified_at_versions(&self) -> Vec<(ObjectID, SequenceNumber)> {
        self.modified_at_versions
            .iter()
            .map(|v| (v.object_id, v.sequence_number))
            .collect::<Vec<_>>()
    }

    fn all_changed_objects(&self) -> Vec<(&OwnedObjectRef, WriteKind)> {
        self.mutated
            .iter()
            .map(|owner_ref| (owner_ref, WriteKind::Mutate))
            .chain(
                self.created
                    .iter()
                    .map(|owner_ref| (owner_ref, WriteKind::Create)),
            )
            .chain(
                self.unwrapped
                    .iter()
                    .map(|owner_ref| (owner_ref, WriteKind::Unwrap)),
            )
            .collect()
    }

    fn all_deleted_objects(&self) -> Vec<(&SuiObjectRef, DeleteKind)> {
        self.deleted
            .iter()
            .map(|r| (r, DeleteKind::Normal))
            .chain(
                self.unwrapped_then_deleted
                    .iter()
                    .map(|r| (r, DeleteKind::UnwrapThenDelete)),
            )
            .chain(self.wrapped.iter().map(|r| (r, DeleteKind::Wrap)))
            .collect()
    }
}

fn owned_objref_string(obj: &OwnedObjectRef) -> String {
    format!(
        " ┌──\n │ ID: {} \n │ Owner: {} \n │ Version: {} \n │ Digest: {}\n └──",
        obj.reference.object_id,
        obj.owner,
        u64::from(obj.reference.version),
        obj.reference.digest
    )
}

fn objref_string(obj: &SuiObjectRef) -> String {
    format!(
        " ┌──\n │ ID: {} \n │ Version: {} \n │ Digest: {}\n └──",
        obj.object_id,
        u64::from(obj.version),
        obj.digest
    )
}

#[derive(Eq, PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename = "TransactionBlockEvents", transparent)]
pub struct SuiTransactionBlockEvents {
    pub data: Vec<SuiEvent>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "ExecutionStatus", rename_all = "camelCase", tag = "status")]
pub enum SuiExecutionStatus {
    // Gas used in the success case.
    Success,
    // Gas used in the failed case, and the error.
    Failure { error: String },
}

impl Display for SuiExecutionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure { error } => write!(f, "failure due to {error}"),
        }
    }
}

impl SuiExecutionStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, SuiExecutionStatus::Success { .. })
    }
    pub fn is_err(&self) -> bool {
        matches!(self, SuiExecutionStatus::Failure { .. })
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename = "GasData", rename_all = "camelCase")]
pub struct SuiGasData {
    pub payment: Vec<SuiObjectRef>,
    pub owner: SuiAddress,
    #[serde_as(as = "BigInt<u64>")]
    pub price: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub budget: u64,
}

impl Display for SuiGasData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Gas Owner: {}", self.owner)?;
        writeln!(f, "Gas Budget: {} MIST", self.budget)?;
        writeln!(f, "Gas Price: {} MIST", self.price)?;
        writeln!(f, "Gas Payment:")?;
        for payment in &self.payment {
            write!(f, "{} ", objref_string(payment))?;
        }
        writeln!(f)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
// #[enum_dispatch(SuiTransactionBlockDataAPI)]
#[serde(
    rename = "TransactionBlockData",
    rename_all = "camelCase",
    tag = "messageVersion"
)]
pub enum SuiTransactionBlockData {
    V1(SuiTransactionBlockDataV1),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename = "TransactionBlockDataV1", rename_all = "camelCase")]
pub struct SuiTransactionBlockDataV1 {
    pub transaction: SuiTransactionBlockKind,
    pub sender: SuiAddress,
    pub gas_data: SuiGasData,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenericSignature {
    Signature(String),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename = "TransactionBlock", rename_all = "camelCase")]
pub struct SuiTransactionBlock {
    pub data: SuiTransactionBlockData,
    // pub tx_signatures: Vec<GenericSignature>,
    pub tx_signatures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiGenesisTransaction {
    pub objects: Vec<ObjectID>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiConsensusCommitPrologue {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,

    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,

    #[serde_as(as = "BigInt<u64>")]
    pub commit_timestamp_ms: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiConsensusCommitPrologueV2 {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,

    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,

    #[serde_as(as = "BigInt<u64>")]
    pub commit_timestamp_ms: u64,
    pub consensus_commit_digest: ConsensusCommitDigest,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiConsensusCommitPrologueV3 {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,

    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,

    #[serde_as(as = "Option<BigInt<u64>>")]
    pub sub_dag_index: Option<u64>,

    #[serde_as(as = "BigInt<u64>")]
    pub commit_timestamp_ms: u64,
    pub consensus_commit_digest: ConsensusCommitDigest,
    pub consensus_determined_version_assignments: ConsensusDeterminedVersionAssignments,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiAuthenticatorStateUpdate {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,

    pub new_active_jwks: Vec<SuiActiveJwk>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiRandomnessStateUpdate {
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[serde_as(as = "BigInt<u64>")]
    pub randomness_round: u64,
    pub random_bytes: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiEndOfEpochTransaction {
    pub transactions: Vec<SuiEndOfEpochTransactionKind>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SuiEndOfEpochTransactionKind {
    ChangeEpoch(SuiChangeEpoch),
    AuthenticatorStateCreate,
    AuthenticatorStateExpire(SuiAuthenticatorStateExpire),
    RandomnessStateCreate,
    CoinDenyListStateCreate,
    BridgeStateCreate(CheckpointDigest),
    BridgeCommitteeUpdate(SequenceNumber),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiAuthenticatorStateExpire {
    #[serde_as(as = "BigInt<u64>")]
    pub min_epoch: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiActiveJwk {
    pub jwk_id: SuiJwkId,
    pub jwk: SuiJWK,

    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiJwkId {
    pub iss: String,
    pub kid: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiJWK {
    pub kty: String,
    pub e: String,
    pub n: String,
    pub alg: String,
}

/// A series of commands where the results of one command can be used in future
/// commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiProgrammableTransactionBlock {
    /// Input objects or primitive values
    pub inputs: Vec<SuiCallArg>,
    #[serde(rename = "transactions")]
    /// The transactions to be executed sequentially. A failure in any transaction will
    /// result in the failure of the entire programmable transaction block.
    pub commands: Vec<SuiCommand>,
}

/// A single transaction in a programmable transaction block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "SuiTransaction")]
pub enum SuiCommand {
    /// A call to either an entry or a public Move function
    MoveCall(Box<SuiProgrammableMoveCall>),
    /// `(Vec<forall T:key+store. T>, address)`
    /// It sends n-objects to the specified address. These objects must have store
    /// (public transfer) and either the previous owner must be an address or the object must
    /// be newly created.
    TransferObjects(Vec<SuiArgument>, SuiArgument),
    /// `(&mut Coin<T>, Vec<u64>)` -> `Vec<Coin<T>>`
    /// It splits off some amounts into a new coins with those amounts
    SplitCoins(SuiArgument, Vec<SuiArgument>),
    /// `(&mut Coin<T>, Vec<Coin<T>>)`
    /// It merges n-coins into the first coin
    MergeCoins(SuiArgument, Vec<SuiArgument>),
    /// Publishes a Move package. It takes the package bytes and a list of the package's transitive
    /// dependencies to link against on-chain.
    Publish(Vec<ObjectID>),
    /// Upgrades a Move package
    Upgrade(Vec<ObjectID>, ObjectID, SuiArgument),
    /// `forall T: Vec<T> -> vector<T>`
    /// Given n-values of the same type, it constructs a vector. For non objects or an empty vector,
    /// the type tag must be specified.
    MakeMoveVec(Option<String>, Vec<SuiArgument>),
}

/// An argument to a transaction in a programmable transaction block
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SuiArgument {
    /// The gas coin. The gas coin can only be used by-ref, except for with
    /// `TransferObjects`, which can use it by-value.
    GasCoin,
    /// One of the input objects or primitive values (from
    /// `ProgrammableTransactionBlock` inputs)
    Input(u16),
    /// The result of another transaction (from `ProgrammableTransactionBlock` transactions)
    Result(u16),
    /// Like a `Result` but it accesses a nested result. Currently, the only usage
    /// of this is to access a value from a Move call with multiple return values.
    NestedResult(u16, u16),
}

/// The transaction for calling a Move function, either an entry function or a public
/// function (which cannot return references).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiProgrammableMoveCall {
    /// The package containing the module and function.
    pub package: ObjectID,
    /// The specific module in the package containing the function.
    pub module: String,
    /// The function to be called.
    pub function: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// The type arguments to the function.
    pub type_arguments: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// The arguments to the function.
    pub arguments: Vec<SuiArgument>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "OwnedObjectRef")]
pub struct OwnedObjectRef {
    pub owner: Owner,
    pub reference: SuiObjectRef,
}

impl OwnedObjectRef {
    pub fn object_id(&self) -> ObjectID {
        self.reference.object_id
    }
    pub fn version(&self) -> SequenceNumber {
        self.reference.version
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SuiCallArg {
    // Needs to become an Object Ref or Object ID, depending on object type
    Object(SuiObjectArg),
    // pure value, bcs encoded
    Pure(SuiPureValue),
}

#[serde_as]
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiPureValue {
    #[serde_as(as = "Option<AsSuiTypeTag>")]
    value_type: Option<TypeTag>,
    value: SuiJsonValue,
}

#[serde_as]
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "objectType", rename_all = "camelCase")]
pub enum SuiObjectArg {
    // A Move object, either immutable, or owned mutable.
    #[serde(rename_all = "camelCase")]
    ImmOrOwnedObject {
        object_id: ObjectID,

        #[serde_as(as = "AsSequenceNumber")]
        version: SequenceNumber,
        digest: ObjectDigest,
    },
    // A Move object that's shared.
    // SharedObject::mutable controls whether caller asks for a mutable reference to shared object.
    #[serde(rename_all = "camelCase")]
    SharedObject {
        object_id: ObjectID,

        #[serde_as(as = "AsSequenceNumber")]
        initial_shared_version: SequenceNumber,
        mutable: bool,
    },
    // A reference to a Move object that's going to be received in the transaction.
    #[serde(rename_all = "camelCase")]
    Receiving {
        object_id: ObjectID,

        #[serde_as(as = "AsSequenceNumber")]
        version: SequenceNumber,
        digest: ObjectDigest,
    },
}
