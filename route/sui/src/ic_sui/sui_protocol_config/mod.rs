use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// The minimum and maximum protocol versions supported by this build.
const MIN_PROTOCOL_VERSION: u64 = 1;
const MAX_PROTOCOL_VERSION: u64 = 70;

#[derive(Copy, Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolVersion(u64);

impl ProtocolVersion {
    // The minimum and maximum protocol version supported by this binary. Counterintuitively, this constant may
    // change over time as support for old protocol versions is removed from the source. This
    // ensures that when a new network (such as a testnet) is created, its genesis committee will
    // use a protocol version that is actually supported by the binary.
    pub const MIN: Self = Self(MIN_PROTOCOL_VERSION);

    pub const MAX: Self = Self(MAX_PROTOCOL_VERSION);

    pub fn new(v: u64) -> Self {
        Self(v)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    // For serde deserialization - we don't define a Default impl because there isn't a single
    // universally appropriate default value.
    pub fn max() -> Self {
        Self::MAX
    }
}

impl From<u64> for ProtocolVersion {
    fn from(v: u64) -> Self {
        Self::new(v)
    }
}

impl std::ops::Sub<u64> for ProtocolVersion {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        Self::new(self.0 - rhs)
    }
}

impl std::ops::Add<u64> for ProtocolVersion {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Copy, PartialOrd, Ord, Eq)]
pub enum Chain {
    Mainnet,
    Testnet,
    Unknown,
}

impl Default for Chain {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Chain {
    pub fn as_str(self) -> &'static str {
        match self {
            Chain::Mainnet => "mainnet",
            Chain::Testnet => "testnet",
            Chain::Unknown => "unknown",
        }
    }
}

pub struct Error(pub String);

// TODO: There are quite a few non boolean values in the feature flags. We should move them out.
/// Records on/off feature flags that may vary at each protocol version.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
struct FeatureFlags {
    // Add feature flags here, e.g.:
    // new_protocol_feature: bool,
    #[serde(skip_serializing_if = "is_false")]
    package_upgrades: bool,
    // If true, validators will commit to the root state digest
    // in end of epoch checkpoint proposals
    #[serde(skip_serializing_if = "is_false")]
    commit_root_state_digest: bool,
    // Pass epoch start time to advance_epoch safe mode function.
    #[serde(skip_serializing_if = "is_false")]
    advance_epoch_start_time_in_safe_mode: bool,
    // If true, apply the fix to correctly capturing loaded child object versions in execution's
    // object runtime.
    #[serde(skip_serializing_if = "is_false")]
    loaded_child_objects_fixed: bool,
    // If true, treat missing types in the upgraded modules when creating an upgraded package as a
    // compatibility error.
    #[serde(skip_serializing_if = "is_false")]
    missing_type_is_compatibility_error: bool,
    // If true, then the scoring decision mechanism will not get disabled when we do have more than
    // f low scoring authorities, but it will simply flag as low scoring only up to f authorities.
    #[serde(skip_serializing_if = "is_false")]
    scoring_decision_with_validity_cutoff: bool,

    // DEPRECATED: this was an ephemeral feature flag only used by consensus handler, which has now
    // been deployed everywhere.
    #[serde(skip_serializing_if = "is_false")]
    consensus_order_end_of_epoch_last: bool,

    // Disallow adding abilities to types during package upgrades.
    #[serde(skip_serializing_if = "is_false")]
    disallow_adding_abilities_on_upgrade: bool,
    // Disables unnecessary invariant check in the Move VM when swapping the value out of a local
    #[serde(skip_serializing_if = "is_false")]
    disable_invariant_violation_check_in_swap_loc: bool,
    // advance to highest supported protocol version at epoch change, instead of the next consecutive
    // protocol version.
    #[serde(skip_serializing_if = "is_false")]
    advance_to_highest_supported_protocol_version: bool,
    // If true, disallow entry modifiers on entry functions
    #[serde(skip_serializing_if = "is_false")]
    ban_entry_init: bool,
    // If true, hash module bytes individually when calculating package digests for upgrades
    #[serde(skip_serializing_if = "is_false")]
    package_digest_hash_module: bool,
    // If true, disallow changing struct type parameters during package upgrades
    #[serde(skip_serializing_if = "is_false")]
    disallow_change_struct_type_params_on_upgrade: bool,
    // If true, checks no extra bytes in a compiled module
    #[serde(skip_serializing_if = "is_false")]
    no_extraneous_module_bytes: bool,
    // If true, then use the versioned metadata format in narwhal entities.
    #[serde(skip_serializing_if = "is_false")]
    narwhal_versioned_metadata: bool,

    // Enable zklogin auth
    #[serde(skip_serializing_if = "is_false")]
    zklogin_auth: bool,
    // How we order transactions coming out of consensus before sending to execution.
    #[serde(skip_serializing_if = "ConsensusTransactionOrdering::is_none")]
    consensus_transaction_ordering: ConsensusTransactionOrdering,

    // Previously, the unwrapped_then_deleted field in TransactionEffects makes a distinction between
    // whether an object has existed in the store previously (i.e. whether there is a tombstone).
    // Such dependency makes effects generation inefficient, and requires us to include wrapped
    // tombstone in state root hash.
    // To prepare for effects V2, with this flag set to true, we simplify the definition of
    // unwrapped_then_deleted to always include unwrapped then deleted objects,
    // regardless of their previous state in the store.
    #[serde(skip_serializing_if = "is_false")]
    simplified_unwrap_then_delete: bool,
    // Enable upgraded multisig support
    #[serde(skip_serializing_if = "is_false")]
    upgraded_multisig_supported: bool,
    // If true minimum txn charge is a multiplier of the gas price
    #[serde(skip_serializing_if = "is_false")]
    txn_base_cost_as_multiplier: bool,

    // If true, the ability to delete shared objects is in effect
    #[serde(skip_serializing_if = "is_false")]
    shared_object_deletion: bool,

    // If true, then the new algorithm for the leader election schedule will be used
    #[serde(skip_serializing_if = "is_false")]
    narwhal_new_leader_election_schedule: bool,

    // A list of supported OIDC providers that can be used for zklogin.
    #[serde(skip_serializing_if = "is_empty")]
    zklogin_supported_providers: BTreeSet<String>,

    // If true, use the new child object format
    #[serde(skip_serializing_if = "is_false")]
    loaded_child_object_format: bool,

    #[serde(skip_serializing_if = "is_false")]
    enable_jwk_consensus_updates: bool,

    #[serde(skip_serializing_if = "is_false")]
    end_of_epoch_transaction_supported: bool,

    // Perform simple conservation checks keeping into account out of gas scenarios
    // while charging for storage.
    #[serde(skip_serializing_if = "is_false")]
    simple_conservation_checks: bool,

    // If true, use the new child object format type logging
    #[serde(skip_serializing_if = "is_false")]
    loaded_child_object_format_type: bool,

    // Enable receiving sent objects
    #[serde(skip_serializing_if = "is_false")]
    receive_objects: bool,

    // Enable random beacon protocol
    #[serde(skip_serializing_if = "is_false")]
    random_beacon: bool,

    // Enable bridge protocol
    #[serde(skip_serializing_if = "is_false")]
    bridge: bool,

    #[serde(skip_serializing_if = "is_false")]
    enable_effects_v2: bool,

    // If true, then use CertificateV2 in narwhal.
    #[serde(skip_serializing_if = "is_false")]
    narwhal_certificate_v2: bool,

    // If true, allow verify with legacy zklogin address
    #[serde(skip_serializing_if = "is_false")]
    verify_legacy_zklogin_address: bool,

    // Enable throughput aware consensus submission
    #[serde(skip_serializing_if = "is_false")]
    throughput_aware_consensus_submission: bool,

    // If true, recompute has_public_transfer from the type instead of what is stored in the object
    #[serde(skip_serializing_if = "is_false")]
    recompute_has_public_transfer_in_execution: bool,

    // If true, multisig containing zkLogin sig is accepted.
    #[serde(skip_serializing_if = "is_false")]
    accept_zklogin_in_multisig: bool,

    // If true, consensus prologue transaction also includes the consensus output digest.
    // It can be used to detect consensus output folk.
    #[serde(skip_serializing_if = "is_false")]
    include_consensus_digest_in_prologue: bool,

    // If true, use the hardened OTW check
    #[serde(skip_serializing_if = "is_false")]
    hardened_otw_check: bool,

    // If true allow calling receiving_object_id function
    #[serde(skip_serializing_if = "is_false")]
    allow_receiving_object_id: bool,

    // Enable the poseidon hash function
    #[serde(skip_serializing_if = "is_false")]
    enable_poseidon: bool,

    // If true, enable the coin deny list.
    #[serde(skip_serializing_if = "is_false")]
    enable_coin_deny_list: bool,

    // Enable native functions for group operations.
    #[serde(skip_serializing_if = "is_false")]
    enable_group_ops_native_functions: bool,

    // Enable native function for msm.
    #[serde(skip_serializing_if = "is_false")]
    enable_group_ops_native_function_msm: bool,

    // Reject functions with mutable Random.
    #[serde(skip_serializing_if = "is_false")]
    reject_mutable_random_on_entry_functions: bool,

    // Controls the behavior of per object congestion control in consensus handler.
    #[serde(skip_serializing_if = "PerObjectCongestionControlMode::is_none")]
    per_object_congestion_control_mode: PerObjectCongestionControlMode,

    // The consensus protocol to be used for the epoch.
    #[serde(skip_serializing_if = "ConsensusChoice::is_narwhal")]
    consensus_choice: ConsensusChoice,

    // Consensus network to use.
    #[serde(skip_serializing_if = "ConsensusNetwork::is_anemo")]
    consensus_network: ConsensusNetwork,

    // Set the upper bound allowed for max_epoch in zklogin signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    zklogin_max_epoch_upper_bound_delta: Option<u64>,

    // Controls leader scoring & schedule change in Mysticeti consensus.
    #[serde(skip_serializing_if = "is_false")]
    mysticeti_leader_scoring_and_schedule: bool,

    // Enable resharing of shared objects using the same initial shared version
    #[serde(skip_serializing_if = "is_false")]
    reshare_at_same_initial_version: bool,

    // Resolve Move abort locations to the package id instead of the runtime module ID.
    #[serde(skip_serializing_if = "is_false")]
    resolve_abort_locations_to_package_id: bool,

    // Enables the use of the Mysticeti committed sub dag digest to the `ConsensusCommitInfo` in checkpoints.
    // When disabled the default digest is used instead. It's important to have this guarded behind
    // a flag as it will lead to checkpoint forks.
    #[serde(skip_serializing_if = "is_false")]
    mysticeti_use_committed_subdag_digest: bool,

    // Enable VDF
    #[serde(skip_serializing_if = "is_false")]
    enable_vdf: bool,

    // Controls whether consensus handler should record consensus determined shared object version
    // assignments in consensus commit prologue transaction.
    // The purpose of doing this is to enable replaying transaction without transaction effects.
    #[serde(skip_serializing_if = "is_false")]
    record_consensus_determined_version_assignments_in_prologue: bool,

    // Run verification of framework upgrades using a new/fresh VM.
    #[serde(skip_serializing_if = "is_false")]
    fresh_vm_on_framework_upgrade: bool,

    // When set to true, the consensus commit prologue transaction will be placed first
    // in a consensus commit in checkpoints.
    // If a checkpoint contains multiple consensus commit, say [cm1][cm2]. The each commit's
    // consensus commit prologue will be the first transaction in each segment:
    //     [ccp1, rest cm1][ccp2, rest cm2]
    // The reason to prepose the prologue transaction is to provide information for transaction
    // cancellation.
    #[serde(skip_serializing_if = "is_false")]
    prepend_prologue_tx_in_consensus_commit_in_checkpoints: bool,

    // Set number of leaders per round for Mysticeti commits.
    #[serde(skip_serializing_if = "Option::is_none")]
    mysticeti_num_leaders_per_round: Option<usize>,

    // Enable Soft Bundle (SIP-19).
    #[serde(skip_serializing_if = "is_false")]
    soft_bundle: bool,

    // If true, enable the coin deny list V2.
    #[serde(skip_serializing_if = "is_false")]
    enable_coin_deny_list_v2: bool,

    // Enable passkey auth (SIP-9)
    #[serde(skip_serializing_if = "is_false")]
    passkey_auth: bool,

    // Use AuthorityCapabilitiesV2
    #[serde(skip_serializing_if = "is_false")]
    authority_capabilities_v2: bool,

    // Rethrow type layout errors during serialization instead of trying to convert them.
    #[serde(skip_serializing_if = "is_false")]
    rethrow_serialization_type_layout_errors: bool,

    // Use distributed vote leader scoring strategy in consensus.
    #[serde(skip_serializing_if = "is_false")]
    consensus_distributed_vote_scoring_strategy: bool,

    // Probe rounds received by peers from every authority.
    #[serde(skip_serializing_if = "is_false")]
    consensus_round_prober: bool,

    // Validate identifier inputs separately
    #[serde(skip_serializing_if = "is_false")]
    validate_identifier_inputs: bool,

    // Enables Mysticeti fastpath.
    #[serde(skip_serializing_if = "is_false")]
    mysticeti_fastpath: bool,

    // Makes the event's sending module version-aware.
    #[serde(skip_serializing_if = "is_false")]
    relocate_event_module: bool,

    // Enable uncompressed group elements in BLS123-81 G1
    #[serde(skip_serializing_if = "is_false")]
    uncompressed_g1_group_elements: bool,

    #[serde(skip_serializing_if = "is_false")]
    disallow_new_modules_in_deps_only_packages: bool,

    // Use smart ancestor selection in consensus.
    #[serde(skip_serializing_if = "is_false")]
    consensus_smart_ancestor_selection: bool,

    // Probe accepted rounds in round prober.
    #[serde(skip_serializing_if = "is_false")]
    consensus_round_prober_probe_accepted_rounds: bool,

    // Enable v2 native charging for natives.
    #[serde(skip_serializing_if = "is_false")]
    native_charging_v2: bool,
}

fn is_false(b: &bool) -> bool {
    !b
}

fn is_empty(b: &BTreeSet<String>) -> bool {
    b.is_empty()
}

/// Ordering mechanism for transactions in one Narwhal consensus output.
#[derive(Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ConsensusTransactionOrdering {
    /// No ordering. Transactions are processed in the order they appear in the consensus output.
    #[default]
    None,
    /// Order transactions by gas price, highest first.
    ByGasPrice,
}

impl ConsensusTransactionOrdering {
    pub fn is_none(&self) -> bool {
        matches!(self, ConsensusTransactionOrdering::None)
    }
}

// The config for per object congestion control in consensus handler.
#[derive(Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum PerObjectCongestionControlMode {
    #[default]
    None, // No congestion control.
    TotalGasBudget,        // Use txn gas budget as execution cost.
    TotalTxCount,          // Use total txn count as execution cost.
    TotalGasBudgetWithCap, // Use txn gas budget as execution cost with a cap.
}

impl PerObjectCongestionControlMode {
    pub fn is_none(&self) -> bool {
        matches!(self, PerObjectCongestionControlMode::None)
    }
}

// Configuration options for consensus algorithm.
#[derive(Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ConsensusChoice {
    #[default]
    Narwhal,
    SwapEachEpoch,
    Mysticeti,
}

impl ConsensusChoice {
    pub fn is_narwhal(&self) -> bool {
        matches!(self, ConsensusChoice::Narwhal)
    }
}

// Configuration options for consensus network.
#[derive(Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ConsensusNetwork {
    #[default]
    Anemo,
    Tonic,
}

impl ConsensusNetwork {
    pub fn is_anemo(&self) -> bool {
        matches!(self, ConsensusNetwork::Anemo)
    }
}

/// Constants that change the behavior of the protocol.
///
/// The value of each constant here must be fixed for a given protocol version. To change the value
/// of a constant, advance the protocol version, and add support for it in `get_for_version` under
/// the new version number.
/// (below).
///
/// To add a new field to this struct, use the following procedure:
/// - Advance the protocol version.
/// - Add the field as a private `Option<T>` to the struct.
/// - Initialize the field to `None` in prior protocol versions.
/// - Initialize the field to `Some(val)` for your new protocol version.
/// - Add a public getter that simply unwraps the field.
/// - Two public getters of the form `field(&self) -> field_type`
///     and `field_as_option(&self) -> Option<field_type>` will be automatically generated for you.
/// Example for a field: `new_constant: Option<u64>`
/// ```rust,ignore
///      pub fn new_constant(&self) -> u64 {
///         self.new_constant.expect(Self::CONSTANT_ERR_MSG)
///     }
///      pub fn new_constant_as_option(&self) -> Option<u64> {
///         self.new_constant.expect(Self::CONSTANT_ERR_MSG)
///     }
/// ```
/// With `pub fn new_constant(&self) -> u64`, if the constant is accessed in a protocol version
/// in which it is not defined, the validator will crash. (Crashing is necessary because
/// this type of error would almost always result in forking if not prevented here).
/// If you don't want the validator to crash, you can use the
/// `pub fn new_constant_as_option(&self) -> Option<u64>` getter, which will
/// return `None` if the field is not defined at that version.
/// - If you want a customized getter, you can add a method in the impl.
#[skip_serializing_none]
#[derive(Clone, Serialize, Debug)]
pub struct ProtocolConfig {
    pub version: ProtocolVersion,

    feature_flags: FeatureFlags,

    // ==== Transaction input limits ====
    /// Maximum serialized size of a transaction (in bytes).
    max_tx_size_bytes: Option<u64>,

    /// Maximum number of input objects to a transaction. Enforced by the transaction input checker
    max_input_objects: Option<u64>,

    /// Max size of objects a transaction can write to disk after completion. Enforce by the Sui adapter.
    /// This is the sum of the serialized size of all objects written to disk.
    /// The max size of individual objects on the other hand is `max_move_object_size`.
    max_size_written_objects: Option<u64>,
    /// Max size of objects a system transaction can write to disk after completion. Enforce by the Sui adapter.
    /// Similar to `max_size_written_objects` but for system transactions.
    max_size_written_objects_system_tx: Option<u64>,

    /// Maximum size of serialized transaction effects.
    max_serialized_tx_effects_size_bytes: Option<u64>,

    /// Maximum size of serialized transaction effects for system transactions.
    max_serialized_tx_effects_size_bytes_system_tx: Option<u64>,

    /// Maximum number of gas payment objects for a transaction.
    max_gas_payment_objects: Option<u32>,

    /// Maximum number of modules in a Publish transaction.
    max_modules_in_publish: Option<u32>,

    /// Maximum number of transitive dependencies in a package when publishing.
    max_package_dependencies: Option<u32>,

    /// Maximum number of arguments in a move call or a ProgrammableTransaction's
    /// TransferObjects command.
    max_arguments: Option<u32>,

    /// Maximum number of total type arguments, computed recursively.
    max_type_arguments: Option<u32>,

    /// Maximum depth of an individual type argument.
    max_type_argument_depth: Option<u32>,

    /// Maximum size of a Pure CallArg.
    max_pure_argument_size: Option<u32>,

    /// Maximum number of Commands in a ProgrammableTransaction.
    max_programmable_tx_commands: Option<u32>,

    // ==== Move VM, Move bytecode verifier, and execution limits ===
    /// Maximum Move bytecode version the VM understands. All older versions are accepted.
    move_binary_format_version: Option<u32>,
    min_move_binary_format_version: Option<u32>,

    /// Configuration controlling binary tables size.
    binary_module_handles: Option<u16>,
    binary_struct_handles: Option<u16>,
    binary_function_handles: Option<u16>,
    binary_function_instantiations: Option<u16>,
    binary_signatures: Option<u16>,
    binary_constant_pool: Option<u16>,
    binary_identifiers: Option<u16>,
    binary_address_identifiers: Option<u16>,
    binary_struct_defs: Option<u16>,
    binary_struct_def_instantiations: Option<u16>,
    binary_function_defs: Option<u16>,
    binary_field_handles: Option<u16>,
    binary_field_instantiations: Option<u16>,
    binary_friend_decls: Option<u16>,
    binary_enum_defs: Option<u16>,
    binary_enum_def_instantiations: Option<u16>,
    binary_variant_handles: Option<u16>,
    binary_variant_instantiation_handles: Option<u16>,

    /// Maximum size of the `contents` part of an object, in bytes. Enforced by the Sui adapter when effects are produced.
    max_move_object_size: Option<u64>,

    // TODO: Option<increase to 500 KB. currently, publishing a package > 500 KB exceeds the max computation gas cost
    /// Maximum size of a Move package object, in bytes. Enforced by the Sui adapter at the end of a publish transaction.
    max_move_package_size: Option<u64>,

    /// Max number of publish or upgrade commands allowed in a programmable transaction block.
    max_publish_or_upgrade_per_ptb: Option<u64>,

    /// Maximum gas budget in MIST that a transaction can use.
    max_tx_gas: Option<u64>,

    /// Maximum amount of the proposed gas price in MIST (defined in the transaction).
    max_gas_price: Option<u64>,

    /// The max computation bucket for gas. This is the max that can be charged for computation.
    max_gas_computation_bucket: Option<u64>,

    // Define the value used to round up computation gas charges
    gas_rounding_step: Option<u64>,

    /// Maximum number of nested loops. Enforced by the Move bytecode verifier.
    max_loop_depth: Option<u64>,

    /// Maximum number of type arguments that can be bound to generic type parameters. Enforced by the Move bytecode verifier.
    max_generic_instantiation_length: Option<u64>,

    /// Maximum number of parameters that a Move function can have. Enforced by the Move bytecode verifier.
    max_function_parameters: Option<u64>,

    /// Maximum number of basic blocks that a Move function can have. Enforced by the Move bytecode verifier.
    max_basic_blocks: Option<u64>,

    /// Maximum stack size value. Enforced by the Move bytecode verifier.
    max_value_stack_size: Option<u64>,

    /// Maximum number of "type nodes", a metric for how big a SignatureToken will be when expanded into a fully qualified type. Enforced by the Move bytecode verifier.
    max_type_nodes: Option<u64>,

    /// Maximum number of push instructions in one function. Enforced by the Move bytecode verifier.
    max_push_size: Option<u64>,

    /// Maximum number of struct definitions in a module. Enforced by the Move bytecode verifier.
    max_struct_definitions: Option<u64>,

    /// Maximum number of function definitions in a module. Enforced by the Move bytecode verifier.
    max_function_definitions: Option<u64>,

    /// Maximum number of fields allowed in a struct definition. Enforced by the Move bytecode verifier.
    max_fields_in_struct: Option<u64>,

    /// Maximum dependency depth. Enforced by the Move linker when loading dependent modules.
    max_dependency_depth: Option<u64>,

    /// Maximum number of Move events that a single transaction can emit. Enforced by the VM during execution.
    max_num_event_emit: Option<u64>,

    /// Maximum number of new IDs that a single transaction can create. Enforced by the VM during execution.
    max_num_new_move_object_ids: Option<u64>,

    /// Maximum number of new IDs that a single system transaction can create. Enforced by the VM during execution.
    max_num_new_move_object_ids_system_tx: Option<u64>,

    /// Maximum number of IDs that a single transaction can delete. Enforced by the VM during execution.
    max_num_deleted_move_object_ids: Option<u64>,

    /// Maximum number of IDs that a single system transaction can delete. Enforced by the VM during execution.
    max_num_deleted_move_object_ids_system_tx: Option<u64>,

    /// Maximum number of IDs that a single transaction can transfer. Enforced by the VM during execution.
    max_num_transferred_move_object_ids: Option<u64>,

    /// Maximum number of IDs that a single system transaction can transfer. Enforced by the VM during execution.
    max_num_transferred_move_object_ids_system_tx: Option<u64>,

    /// Maximum size of a Move user event. Enforced by the VM during execution.
    max_event_emit_size: Option<u64>,

    /// Maximum size of a Move user event. Enforced by the VM during execution.
    max_event_emit_size_total: Option<u64>,

    /// Maximum length of a vector in Move. Enforced by the VM during execution, and for constants, by the verifier.
    max_move_vector_len: Option<u64>,

    /// Maximum length of an `Identifier` in Move. Enforced by the bytecode verifier at signing.
    max_move_identifier_len: Option<u64>,

    /// Maximum depth of a Move value within the VM.
    max_move_value_depth: Option<u64>,

    /// Maximum number of variants in an enum. Enforced by the bytecode verifier at signing.
    max_move_enum_variants: Option<u64>,

    /// Maximum number of back edges in Move function. Enforced by the bytecode verifier at signing.
    max_back_edges_per_function: Option<u64>,

    /// Maximum number of back edges in Move module. Enforced by the bytecode verifier at signing.
    max_back_edges_per_module: Option<u64>,

    /// Maximum number of meter `ticks` spent verifying a Move function. Enforced by the bytecode verifier at signing.
    max_verifier_meter_ticks_per_function: Option<u64>,

    /// Maximum number of meter `ticks` spent verifying a Move module. Enforced by the bytecode verifier at signing.
    max_meter_ticks_per_module: Option<u64>,

    /// Maximum number of meter `ticks` spent verifying a Move package. Enforced by the bytecode verifier at signing.
    max_meter_ticks_per_package: Option<u64>,

    // === Object runtime internal operation limits ====
    // These affect dynamic fields
    /// Maximum number of cached objects in the object runtime ObjectStore. Enforced by object runtime during execution
    object_runtime_max_num_cached_objects: Option<u64>,

    /// Maximum number of cached objects in the object runtime ObjectStore in system transaction. Enforced by object runtime during execution
    object_runtime_max_num_cached_objects_system_tx: Option<u64>,

    /// Maximum number of stored objects accessed by object runtime ObjectStore. Enforced by object runtime during execution
    object_runtime_max_num_store_entries: Option<u64>,

    /// Maximum number of stored objects accessed by object runtime ObjectStore in system transaction. Enforced by object runtime during execution
    object_runtime_max_num_store_entries_system_tx: Option<u64>,

    // === Execution gas costs ====
    /// Base cost for any Sui transaction
    base_tx_cost_fixed: Option<u64>,

    /// Additional cost for a transaction that publishes a package
    /// i.e., the base cost of such a transaction is base_tx_cost_fixed + package_publish_cost_fixed
    package_publish_cost_fixed: Option<u64>,

    /// Cost per byte of a Move call transaction
    /// i.e., the cost of such a transaction is base_cost + (base_tx_cost_per_byte * size)
    base_tx_cost_per_byte: Option<u64>,

    /// Cost per byte for a transaction that publishes a package
    package_publish_cost_per_byte: Option<u64>,

    // Per-byte cost of reading an object during transaction execution
    obj_access_cost_read_per_byte: Option<u64>,

    // Per-byte cost of writing an object during transaction execution
    obj_access_cost_mutate_per_byte: Option<u64>,

    // Per-byte cost of deleting an object during transaction execution
    obj_access_cost_delete_per_byte: Option<u64>,

    /// Per-byte cost charged for each input object to a transaction.
    /// Meant to approximate the cost of checking locks for each object
    // TODO: Option<I'm not sure that this cost makes sense. Checking locks is "free"
    // in the sense that an invalid tx that can never be committed/pay gas can
    // force validators to check an arbitrary number of locks. If those checks are
    // "free" for invalid transactions, why charge for them in valid transactions
    // TODO: Option<if we keep this, I think we probably want it to be a fixed cost rather
    // than a per-byte cost. checking an object lock should not require loading an
    // entire object, just consulting an ID -> tx digest map
    obj_access_cost_verify_per_byte: Option<u64>,

    // Maximal nodes which are allowed when converting to a type layout.
    max_type_to_layout_nodes: Option<u64>,

    /// === Gas version. gas model ===

    /// Gas model version, what code we are using to charge gas
    gas_model_version: Option<u64>,

    /// === Storage gas costs ===

    /// Per-byte cost of storing an object in the Sui global object store. Some of this cost may be refundable if the object is later freed
    obj_data_cost_refundable: Option<u64>,

    // Per-byte cost of storing an object in the Sui transaction log (e.g., in CertifiedTransactionEffects)
    // This depends on the size of various fields including the effects
    // TODO: Option<I don't fully understand this^ and more details would be useful
    obj_metadata_cost_non_refundable: Option<u64>,

    /// === Tokenomics ===

    // TODO: Option<this should be changed to u64.
    /// Sender of a txn that touches an object will get this percent of the storage rebate back.
    /// In basis point.
    storage_rebate_rate: Option<u64>,

    /// 5% of the storage fund's share of rewards are reinvested into the storage fund.
    /// In basis point.
    storage_fund_reinvest_rate: Option<u64>,

    /// The share of rewards that will be slashed and redistributed is 50%.
    /// In basis point.
    reward_slashing_rate: Option<u64>,

    /// Unit gas price, Mist per internal gas unit.
    storage_gas_price: Option<u64>,

    /// === Core Protocol ===

    /// Max number of transactions per checkpoint.
    /// Note that this is a protocol constant and not a config as validators must have this set to
    /// the same value, otherwise they *will* fork.
    max_transactions_per_checkpoint: Option<u64>,

    /// Max size of a checkpoint in bytes.
    /// Note that this is a protocol constant and not a config as validators must have this set to
    /// the same value, otherwise they *will* fork.
    max_checkpoint_size_bytes: Option<u64>,

    /// A protocol upgrade always requires 2f+1 stake to agree. We support a buffer of additional
    /// stake (as a fraction of f, expressed in basis points) that is required before an upgrade
    /// can happen automatically. 10000bps would indicate that complete unanimity is required (all
    /// 3f+1 must vote), while 0bps would indicate that 2f+1 is sufficient.
    buffer_stake_for_protocol_upgrade_bps: Option<u64>,

    // === Native Function Costs ===

    // `address` module
    // Cost params for the Move native function `address::from_bytes(bytes: vector<u8>)`
    address_from_bytes_cost_base: Option<u64>,
    // Cost params for the Move native function `address::to_u256(address): u256`
    address_to_u256_cost_base: Option<u64>,
    // Cost params for the Move native function `address::from_u256(u256): address`
    address_from_u256_cost_base: Option<u64>,

    // `config` module
    // Cost params for the Move native function `read_setting_impl<Name: copy + drop + store,
    // SettingValue: key + store, SettingDataValue: store, Value: copy + drop + store,
    // >(config: address, name: address, current_epoch: u64): Option<Value>`
    config_read_setting_impl_cost_base: Option<u64>,
    config_read_setting_impl_cost_per_byte: Option<u64>,

    // `dynamic_field` module
    // Cost params for the Move native function `hash_type_and_key<K: copy + drop + store>(parent: address, k: K): address`
    dynamic_field_hash_type_and_key_cost_base: Option<u64>,
    dynamic_field_hash_type_and_key_type_cost_per_byte: Option<u64>,
    dynamic_field_hash_type_and_key_value_cost_per_byte: Option<u64>,
    dynamic_field_hash_type_and_key_type_tag_cost_per_byte: Option<u64>,
    // Cost params for the Move native function `add_child_object<Child: key>(parent: address, child: Child)`
    dynamic_field_add_child_object_cost_base: Option<u64>,
    dynamic_field_add_child_object_type_cost_per_byte: Option<u64>,
    dynamic_field_add_child_object_value_cost_per_byte: Option<u64>,
    dynamic_field_add_child_object_struct_tag_cost_per_byte: Option<u64>,
    // Cost params for the Move native function `borrow_child_object_mut<Child: key>(parent: &mut UID, id: address): &mut Child`
    dynamic_field_borrow_child_object_cost_base: Option<u64>,
    dynamic_field_borrow_child_object_child_ref_cost_per_byte: Option<u64>,
    dynamic_field_borrow_child_object_type_cost_per_byte: Option<u64>,
    // Cost params for the Move native function `remove_child_object<Child: key>(parent: address, id: address): Child`
    dynamic_field_remove_child_object_cost_base: Option<u64>,
    dynamic_field_remove_child_object_child_cost_per_byte: Option<u64>,
    dynamic_field_remove_child_object_type_cost_per_byte: Option<u64>,
    // Cost params for the Move native function `has_child_object(parent: address, id: address): bool`
    dynamic_field_has_child_object_cost_base: Option<u64>,
    // Cost params for the Move native function `has_child_object_with_ty<Child: key>(parent: address, id: address): bool`
    dynamic_field_has_child_object_with_ty_cost_base: Option<u64>,
    dynamic_field_has_child_object_with_ty_type_cost_per_byte: Option<u64>,
    dynamic_field_has_child_object_with_ty_type_tag_cost_per_byte: Option<u64>,

    // `event` module
    // Cost params for the Move native function `event::emit<T: copy + drop>(event: T)`
    event_emit_cost_base: Option<u64>,
    event_emit_value_size_derivation_cost_per_byte: Option<u64>,
    event_emit_tag_size_derivation_cost_per_byte: Option<u64>,
    event_emit_output_cost_per_byte: Option<u64>,

    //  `object` module
    // Cost params for the Move native function `borrow_uid<T: key>(obj: &T): &UID`
    object_borrow_uid_cost_base: Option<u64>,
    // Cost params for the Move native function `delete_impl(id: address)`
    object_delete_impl_cost_base: Option<u64>,
    // Cost params for the Move native function `record_new_uid(id: address)`
    object_record_new_uid_cost_base: Option<u64>,

    // Transfer
    // Cost params for the Move native function `transfer_impl<T: key>(obj: T, recipient: address)`
    transfer_transfer_internal_cost_base: Option<u64>,
    // Cost params for the Move native function `freeze_object<T: key>(obj: T)`
    transfer_freeze_object_cost_base: Option<u64>,
    // Cost params for the Move native function `share_object<T: key>(obj: T)`
    transfer_share_object_cost_base: Option<u64>,
    // Cost params for the Move native function
    // `receive_object<T: key>(p: &mut UID, recv: Receiving<T>T)`
    transfer_receive_object_cost_base: Option<u64>,

    // TxContext
    // Cost params for the Move native function `transfer_impl<T: key>(obj: T, recipient: address)`
    tx_context_derive_id_cost_base: Option<u64>,

    // Types
    // Cost params for the Move native function `is_one_time_witness<T: drop>(_: &T): bool`
    types_is_one_time_witness_cost_base: Option<u64>,
    types_is_one_time_witness_type_tag_cost_per_byte: Option<u64>,
    types_is_one_time_witness_type_cost_per_byte: Option<u64>,

    // Validator
    // Cost params for the Move native function `validate_metadata_bcs(metadata: vector<u8>)`
    validator_validate_metadata_cost_base: Option<u64>,
    validator_validate_metadata_data_cost_per_byte: Option<u64>,

    // Crypto natives
    crypto_invalid_arguments_cost: Option<u64>,
    // bls12381::bls12381_min_sig_verify
    bls12381_bls12381_min_sig_verify_cost_base: Option<u64>,
    bls12381_bls12381_min_sig_verify_msg_cost_per_byte: Option<u64>,
    bls12381_bls12381_min_sig_verify_msg_cost_per_block: Option<u64>,

    // bls12381::bls12381_min_pk_verify
    bls12381_bls12381_min_pk_verify_cost_base: Option<u64>,
    bls12381_bls12381_min_pk_verify_msg_cost_per_byte: Option<u64>,
    bls12381_bls12381_min_pk_verify_msg_cost_per_block: Option<u64>,

    // ecdsa_k1::ecrecover
    ecdsa_k1_ecrecover_keccak256_cost_base: Option<u64>,
    ecdsa_k1_ecrecover_keccak256_msg_cost_per_byte: Option<u64>,
    ecdsa_k1_ecrecover_keccak256_msg_cost_per_block: Option<u64>,
    ecdsa_k1_ecrecover_sha256_cost_base: Option<u64>,
    ecdsa_k1_ecrecover_sha256_msg_cost_per_byte: Option<u64>,
    ecdsa_k1_ecrecover_sha256_msg_cost_per_block: Option<u64>,

    // ecdsa_k1::decompress_pubkey
    ecdsa_k1_decompress_pubkey_cost_base: Option<u64>,

    // ecdsa_k1::secp256k1_verify
    ecdsa_k1_secp256k1_verify_keccak256_cost_base: Option<u64>,
    ecdsa_k1_secp256k1_verify_keccak256_msg_cost_per_byte: Option<u64>,
    ecdsa_k1_secp256k1_verify_keccak256_msg_cost_per_block: Option<u64>,
    ecdsa_k1_secp256k1_verify_sha256_cost_base: Option<u64>,
    ecdsa_k1_secp256k1_verify_sha256_msg_cost_per_byte: Option<u64>,
    ecdsa_k1_secp256k1_verify_sha256_msg_cost_per_block: Option<u64>,

    // ecdsa_r1::ecrecover
    ecdsa_r1_ecrecover_keccak256_cost_base: Option<u64>,
    ecdsa_r1_ecrecover_keccak256_msg_cost_per_byte: Option<u64>,
    ecdsa_r1_ecrecover_keccak256_msg_cost_per_block: Option<u64>,
    ecdsa_r1_ecrecover_sha256_cost_base: Option<u64>,
    ecdsa_r1_ecrecover_sha256_msg_cost_per_byte: Option<u64>,
    ecdsa_r1_ecrecover_sha256_msg_cost_per_block: Option<u64>,

    // ecdsa_r1::secp256k1_verify
    ecdsa_r1_secp256r1_verify_keccak256_cost_base: Option<u64>,
    ecdsa_r1_secp256r1_verify_keccak256_msg_cost_per_byte: Option<u64>,
    ecdsa_r1_secp256r1_verify_keccak256_msg_cost_per_block: Option<u64>,
    ecdsa_r1_secp256r1_verify_sha256_cost_base: Option<u64>,
    ecdsa_r1_secp256r1_verify_sha256_msg_cost_per_byte: Option<u64>,
    ecdsa_r1_secp256r1_verify_sha256_msg_cost_per_block: Option<u64>,

    // ecvrf::verify
    ecvrf_ecvrf_verify_cost_base: Option<u64>,
    ecvrf_ecvrf_verify_alpha_string_cost_per_byte: Option<u64>,
    ecvrf_ecvrf_verify_alpha_string_cost_per_block: Option<u64>,

    // ed25519
    ed25519_ed25519_verify_cost_base: Option<u64>,
    ed25519_ed25519_verify_msg_cost_per_byte: Option<u64>,
    ed25519_ed25519_verify_msg_cost_per_block: Option<u64>,

    // groth16::prepare_verifying_key
    groth16_prepare_verifying_key_bls12381_cost_base: Option<u64>,
    groth16_prepare_verifying_key_bn254_cost_base: Option<u64>,

    // groth16::verify_groth16_proof_internal
    groth16_verify_groth16_proof_internal_bls12381_cost_base: Option<u64>,
    groth16_verify_groth16_proof_internal_bls12381_cost_per_public_input: Option<u64>,
    groth16_verify_groth16_proof_internal_bn254_cost_base: Option<u64>,
    groth16_verify_groth16_proof_internal_bn254_cost_per_public_input: Option<u64>,
    groth16_verify_groth16_proof_internal_public_input_cost_per_byte: Option<u64>,

    // hash::blake2b256
    hash_blake2b256_cost_base: Option<u64>,
    hash_blake2b256_data_cost_per_byte: Option<u64>,
    hash_blake2b256_data_cost_per_block: Option<u64>,

    // hash::keccak256
    hash_keccak256_cost_base: Option<u64>,
    hash_keccak256_data_cost_per_byte: Option<u64>,
    hash_keccak256_data_cost_per_block: Option<u64>,

    // poseidon::poseidon_bn254
    poseidon_bn254_cost_base: Option<u64>,
    poseidon_bn254_cost_per_block: Option<u64>,

    // group_ops
    group_ops_bls12381_decode_scalar_cost: Option<u64>,
    group_ops_bls12381_decode_g1_cost: Option<u64>,
    group_ops_bls12381_decode_g2_cost: Option<u64>,
    group_ops_bls12381_decode_gt_cost: Option<u64>,
    group_ops_bls12381_scalar_add_cost: Option<u64>,
    group_ops_bls12381_g1_add_cost: Option<u64>,
    group_ops_bls12381_g2_add_cost: Option<u64>,
    group_ops_bls12381_gt_add_cost: Option<u64>,
    group_ops_bls12381_scalar_sub_cost: Option<u64>,
    group_ops_bls12381_g1_sub_cost: Option<u64>,
    group_ops_bls12381_g2_sub_cost: Option<u64>,
    group_ops_bls12381_gt_sub_cost: Option<u64>,
    group_ops_bls12381_scalar_mul_cost: Option<u64>,
    group_ops_bls12381_g1_mul_cost: Option<u64>,
    group_ops_bls12381_g2_mul_cost: Option<u64>,
    group_ops_bls12381_gt_mul_cost: Option<u64>,
    group_ops_bls12381_scalar_div_cost: Option<u64>,
    group_ops_bls12381_g1_div_cost: Option<u64>,
    group_ops_bls12381_g2_div_cost: Option<u64>,
    group_ops_bls12381_gt_div_cost: Option<u64>,
    group_ops_bls12381_g1_hash_to_base_cost: Option<u64>,
    group_ops_bls12381_g2_hash_to_base_cost: Option<u64>,
    group_ops_bls12381_g1_hash_to_cost_per_byte: Option<u64>,
    group_ops_bls12381_g2_hash_to_cost_per_byte: Option<u64>,
    group_ops_bls12381_g1_msm_base_cost: Option<u64>,
    group_ops_bls12381_g2_msm_base_cost: Option<u64>,
    group_ops_bls12381_g1_msm_base_cost_per_input: Option<u64>,
    group_ops_bls12381_g2_msm_base_cost_per_input: Option<u64>,
    group_ops_bls12381_msm_max_len: Option<u32>,
    group_ops_bls12381_pairing_cost: Option<u64>,
    group_ops_bls12381_g1_to_uncompressed_g1_cost: Option<u64>,
    group_ops_bls12381_uncompressed_g1_to_g1_cost: Option<u64>,
    group_ops_bls12381_uncompressed_g1_sum_base_cost: Option<u64>,
    group_ops_bls12381_uncompressed_g1_sum_cost_per_term: Option<u64>,
    group_ops_bls12381_uncompressed_g1_sum_max_terms: Option<u64>,

    // hmac::hmac_sha3_256
    hmac_hmac_sha3_256_cost_base: Option<u64>,
    hmac_hmac_sha3_256_input_cost_per_byte: Option<u64>,
    hmac_hmac_sha3_256_input_cost_per_block: Option<u64>,

    // zklogin::check_zklogin_id
    check_zklogin_id_cost_base: Option<u64>,
    // zklogin::check_zklogin_issuer
    check_zklogin_issuer_cost_base: Option<u64>,

    vdf_verify_vdf_cost: Option<u64>,
    vdf_hash_to_input_cost: Option<u64>,

    // Stdlib costs
    bcs_per_byte_serialized_cost: Option<u64>,
    bcs_legacy_min_output_size_cost: Option<u64>,
    bcs_failure_cost: Option<u64>,

    hash_sha2_256_base_cost: Option<u64>,
    hash_sha2_256_per_byte_cost: Option<u64>,
    hash_sha2_256_legacy_min_input_len_cost: Option<u64>,
    hash_sha3_256_base_cost: Option<u64>,
    hash_sha3_256_per_byte_cost: Option<u64>,
    hash_sha3_256_legacy_min_input_len_cost: Option<u64>,
    type_name_get_base_cost: Option<u64>,
    type_name_get_per_byte_cost: Option<u64>,

    string_check_utf8_base_cost: Option<u64>,
    string_check_utf8_per_byte_cost: Option<u64>,
    string_is_char_boundary_base_cost: Option<u64>,
    string_sub_string_base_cost: Option<u64>,
    string_sub_string_per_byte_cost: Option<u64>,
    string_index_of_base_cost: Option<u64>,
    string_index_of_per_byte_pattern_cost: Option<u64>,
    string_index_of_per_byte_searched_cost: Option<u64>,

    vector_empty_base_cost: Option<u64>,
    vector_length_base_cost: Option<u64>,
    vector_push_back_base_cost: Option<u64>,
    vector_push_back_legacy_per_abstract_memory_unit_cost: Option<u64>,
    vector_borrow_base_cost: Option<u64>,
    vector_pop_back_base_cost: Option<u64>,
    vector_destroy_empty_base_cost: Option<u64>,
    vector_swap_base_cost: Option<u64>,
    debug_print_base_cost: Option<u64>,
    debug_print_stack_trace_base_cost: Option<u64>,

    // ==== Ephemeral (consensus only) params deleted ====
    //
    // Const params for consensus scoring decision
    // The scaling factor property for the MED outlier detection
    // scoring_decision_mad_divisor: Option<f64>,
    // The cutoff value for the MED outlier detection
    // scoring_decision_cutoff_value: Option<f64>,
    /// === Execution Version ===
    execution_version: Option<u64>,

    // Dictates the threshold (percentage of stake) that is used to calculate the "bad" nodes to be
    // swapped when creating the consensus schedule. The values should be of the range [0 - 33]. Anything
    // above 33 (f) will not be allowed.
    consensus_bad_nodes_stake_threshold: Option<u64>,

    max_jwk_votes_per_validator_per_epoch: Option<u64>,
    // The maximum age of a JWK in epochs before it is removed from the AuthenticatorState object.
    // Applied at the end of an epoch as a delta from the new epoch value, so setting this to 1
    // will cause the new epoch to start with JWKs from the previous epoch still valid.
    max_age_of_jwk_in_epochs: Option<u64>,

    /// === random beacon ===

    /// Maximum allowed precision loss when reducing voting weights for the random beacon
    /// protocol.
    random_beacon_reduction_allowed_delta: Option<u16>,

    /// Minimum number of shares below which voting weights will not be reduced for the
    /// random beacon protocol.
    random_beacon_reduction_lower_bound: Option<u32>,

    /// Consensus Round after which DKG should be aborted and randomness disabled for
    /// the epoch, if it hasn't already completed.
    random_beacon_dkg_timeout_round: Option<u32>,

    /// Minimum interval between consecutive rounds of generated randomness.
    random_beacon_min_round_interval_ms: Option<u64>,

    /// Version of the random beacon DKG protocol.
    /// 0 was deprecated (and currently not supported), 1 is the default version.
    random_beacon_dkg_version: Option<u64>,

    /// The maximum serialised transaction size (in bytes) accepted by consensus. That should be bigger than the
    /// `max_tx_size_bytes` with some additional headroom.
    consensus_max_transaction_size_bytes: Option<u64>,
    /// The maximum size of transactions included in a consensus block.
    consensus_max_transactions_in_block_bytes: Option<u64>,
    /// The maximum number of transactions included in a consensus block.
    consensus_max_num_transactions_in_block: Option<u64>,

    /// The maximum number of rounds where transaction voting is allowed.
    consensus_voting_rounds: Option<u32>,

    /// DEPRECATED. Do not use.
    max_accumulated_txn_cost_per_object_in_narwhal_commit: Option<u64>,

    /// The max number of consensus rounds a transaction can be deferred due to shared object congestion.
    /// Transactions will be cancelled after this many rounds.
    max_deferral_rounds_for_congestion_control: Option<u64>,

    /// If >0, congestion control will allow up to one transaction per object to exceed
    /// the configured maximum accumulated cost by the given amount.
    max_txn_cost_overage_per_object_in_commit: Option<u64>,

    /// Minimum interval of commit timestamps between consecutive checkpoints.
    min_checkpoint_interval_ms: Option<u64>,

    /// Version number to use for version_specific_data in `CheckpointSummary`.
    checkpoint_summary_version_specific_data: Option<u64>,

    /// The max number of transactions that can be included in a single Soft Bundle.
    max_soft_bundle_size: Option<u64>,

    /// Whether to try to form bridge committee
    // Note: this is not a feature flag because we want to distinguish between
    // `None` and `Some(false)`, as committee was already finalized on Testnet.
    bridge_should_try_to_finalize_committee: Option<bool>,

    /// The max accumulated txn execution cost per object in a mysticeti. Transactions
    /// in a commit will be deferred once their touch shared objects hit this limit,
    /// unless the selected congestion control mode allows overage.
    /// This config plays the same role as `max_accumulated_txn_cost_per_object_in_narwhal_commit`
    /// but for mysticeti commits due to that mysticeti has higher commit rate.
    max_accumulated_txn_cost_per_object_in_mysticeti_commit: Option<u64>,

    /// As above, but separate per-commit budget for transactions that use randomness.
    /// If not configured, uses the setting for `max_accumulated_txn_cost_per_object_in_mysticeti_commit`.
    max_accumulated_randomness_txn_cost_per_object_in_mysticeti_commit: Option<u64>,

    /// Configures the garbage collection depth for consensus. When is unset or `0` then the garbage collection
    /// is disabled.
    consensus_gc_depth: Option<u32>,

    /// Used to calculate the max transaction cost when using TotalGasBudgetWithCap as shard
    /// object congestion control strategy. Basically the max transaction cost is calculated as
    /// (num of input object + num of commands) * this factor.
    gas_budget_based_txn_cost_cap_factor: Option<u64>,

    /// Adds an absolute cap on the maximum transaction cost when using TotalGasBudgetWithCap at
    /// the given multiple of the per-commit budget.
    gas_budget_based_txn_cost_absolute_cap_commit_count: Option<u64>,
}
