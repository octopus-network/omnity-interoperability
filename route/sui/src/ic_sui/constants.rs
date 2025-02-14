pub const GET_BLOCK_RESPONSE_SIZE_ESTIMATE: u64 = 516_000;
pub const GET_SUPPLY_SIZE_ESTIMATE: u64 = 1024;
pub const GET_EPOCH_INFO_SIZE_ESTIMATE: u64 = 56;

pub const NODES_IN_SUBNET: u32 = 34;
// https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
// pub const EDDSA_SIGN_COST: u128 = 26_153_846_153;
pub const EDDSA_SIGN_COST: u128 = 26_200_000_000;

// HTTP outcall cost calculation
// See https://internetcomputer.org/docs/current/developer-docs/gas-cost#special-features

// pub const INGRESS_OVERHEAD_BYTES: u128 = 100;
// pub const INGRESS_MESSAGE_RECEIVED_COST: u128 = 1_200_000;
// pub const INGRESS_MESSAGE_BYTE_RECEIVED_COST: u128 = 2_000;
pub const INGRESS_OVERHEAD_BYTES: u128 = 100;
pub const INGRESS_MESSAGE_RECEIVED_COST: u128 = 1_200_000;
pub const INGRESS_MESSAGE_BYTE_RECEIVED_COST: u128 = 2_000;

pub const HTTP_OUTCALL_REQUEST_BASE_COST: u128 = 3_000_000;
pub const HTTP_OUTCALL_REQUEST_PER_NODE_COST: u128 = 60_000;
pub const HTTP_OUTCALL_REQUEST_COST_PER_BYTE: u128 = 400;
pub const HTTP_OUTCALL_RESPONSE_COST_PER_BYTE: u128 = 800;

// /// Minimum number of bytes charged for a URL; improves consistency of costs between providers
pub const RPC_URL_COST_BYTES: u32 = 256;

// /// Additional cost of operating the canister per subnet node
pub const CANISTER_OVERHEAD: u128 = 1_000_000;

/// This constant is our approximation of the expected header size.
/// The HTTP standard doesn't define any limit, and many implementations limit
/// the headers size to 8 KiB. We chose a lower limit because headers observed on most providers
/// fit in the constant defined below, and if there is a spike, then the payload size adjustment
/// should take care of that.
pub const HEADER_SIZE_LIMIT: u64 = 2 * 1024;

// /// Maximum permitted size of account data (10 MiB).
// pub const MAX_ACCOUNT_DATA_LENGTH: u64 = 10 * 1024 * 1024;

/// Maximum permitted size of PDA account data (10 KiB).
/// However, a PDA can be resized up to the 10 MB limit.
pub const MAX_PDA_ACCOUNT_DATA_LENGTH: u64 = 10 * 1024;

/// In case no memo is set signature object should be around 175 bytes long.
pub const SIGNATURE_RESPONSE_SIZE_ESTIMATE: u64 = 500;

// pub const TRANSACTION_RESPONSE_SIZE_ESTIMATE: u64 = 1024 * 1024;
pub const TRANSACTION_RESPONSE_SIZE_ESTIMATE: u64 = 15_000;

pub const TRANSACTION_STATUS_RESPONSE_SIZE_ESTIMATE: u64 = 256;

/// a tx includes tansfer + burn+ memo  should be around 5000 bytes long.
pub const TX_MEMO_RESP_SIZE_ESTIMATE: u64 = 5 * 1024;

/// Idempotency key
pub const IDEMPOTENCY_KEY: &str = "X-Idempotency";

/// forward key
pub const FORWARD_KEY: &str = "X-Forward-Solana";

pub const CLIENT_REQUEST_METHOD_HEADER: &str = "client-request-method";
pub const CLIENT_SDK_TYPE_HEADER: &str = "client-sdk-type";
/// The version number of the SDK itself. This can be different from the API version.
pub const CLIENT_SDK_VERSION_HEADER: &str = "client-sdk-version";
/// The RPC API version that the client is targeting. Different SDK versions may target the same
/// API version.
pub const CLIENT_TARGET_API_VERSION_HEADER: &str = "client-target-api-version";

pub const CLIENT_VERSION: &str = "1.38.1";
