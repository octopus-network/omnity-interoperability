#[allow(unused)]
// pub mod dynamic_field;
pub mod authenticator_state;
pub mod balance;
pub mod base_types;
pub mod coin;
pub mod crypto;
pub mod digests;
pub mod error;
pub mod gas;
// pub mod effects;
pub mod effects;
pub mod event;
pub mod execution;
pub mod execution_status;
pub mod gas_coin;
pub mod governance;
pub mod id;
pub mod message_envelope;
pub mod messages_checkpoint;
pub mod messages_consensus;
pub mod move_package;
pub mod object;
pub mod ptb;
pub mod quorum_driver_types;
pub mod signature;
pub mod storage;
pub mod sui_serde;
pub mod supported_protocol_versions;
pub mod transaction;
pub mod type_input;

use base_types::{ObjectID, SequenceNumber, SuiAddress};

use crate::ic_sui::move_core_types::language_storage::ModuleId;
use crate::ic_sui::move_core_types::{
    account_address::AccountAddress, language_storage::StructTag,
};
pub use crate::ic_sui::move_core_types::{identifier::Identifier, language_storage::TypeTag};
use object::OBJECT_START_VERSION;

macro_rules! built_in_ids {
    ($($addr:ident / $id:ident = $init:expr);* $(;)?) => {
        $(
            pub const $addr: AccountAddress = AccountAddress::from_suffix($init);
            pub const $id: ObjectID = ObjectID::from_address($addr);
        )*
    }
}

macro_rules! built_in_pkgs {
    ($($addr:ident / $id:ident = $init:expr);* $(;)?) => {
        built_in_ids! { $($addr / $id = $init;)* }
        pub const SYSTEM_PACKAGE_ADDRESSES: &[AccountAddress] = &[$($addr),*];
        pub fn is_system_package(addr: impl Into<AccountAddress>) -> bool {
            matches!(addr.into(), $($addr)|*)
        }
    }
}

built_in_pkgs! {
    MOVE_STDLIB_ADDRESS / MOVE_STDLIB_PACKAGE_ID = 0x1;
    SUI_FRAMEWORK_ADDRESS / SUI_FRAMEWORK_PACKAGE_ID = 0x2;
    SUI_SYSTEM_ADDRESS / SUI_SYSTEM_PACKAGE_ID = 0x3;
    BRIDGE_ADDRESS / BRIDGE_PACKAGE_ID = 0xb;
    DEEPBOOK_ADDRESS / DEEPBOOK_PACKAGE_ID = 0xdee9;
}

built_in_ids! {
    SUI_SYSTEM_STATE_ADDRESS / SUI_SYSTEM_STATE_OBJECT_ID = 0x5;
    SUI_CLOCK_ADDRESS / SUI_CLOCK_OBJECT_ID = 0x6;
    SUI_AUTHENTICATOR_STATE_ADDRESS / SUI_AUTHENTICATOR_STATE_OBJECT_ID = 0x7;
    SUI_RANDOMNESS_STATE_ADDRESS / SUI_RANDOMNESS_STATE_OBJECT_ID = 0x8;
    SUI_BRIDGE_ADDRESS / SUI_BRIDGE_OBJECT_ID = 0x9;
    SUI_DENY_LIST_ADDRESS / SUI_DENY_LIST_OBJECT_ID = 0x403;
}

pub const SUI_SYSTEM_STATE_OBJECT_SHARED_VERSION: SequenceNumber = OBJECT_START_VERSION;
pub const SUI_CLOCK_OBJECT_SHARED_VERSION: SequenceNumber = OBJECT_START_VERSION;
pub const SUI_AUTHENTICATOR_STATE_OBJECT_SHARED_VERSION: SequenceNumber = OBJECT_START_VERSION;

pub fn sui_framework_address_concat_string(suffix: &str) -> String {
    format!("{}{suffix}", SUI_FRAMEWORK_ADDRESS.to_hex_literal())
}

/// Parses `s` as an address. Valid formats for addresses are:
///
/// - A 256bit number, encoded in decimal, or hexadecimal with a leading "0x" prefix.
/// - One of a number of pre-defined named addresses: std, sui, sui_system, deepbook.
///
/// Parsing succeeds if and only if `s` matches one of these formats exactly, with no remaining
/// suffix. This function is intended for use within the authority codebases.
pub fn parse_sui_address(s: &str) -> anyhow::Result<SuiAddress> {
    use crate::ic_sui::move_core_types::parsing::address::ParsedAddress;
    Ok(ParsedAddress::parse(s)?
        .into_account_address(&resolve_address)?
        .into())
}

/// Parse `s` as a Module ID: An address (see `parse_sui_address`), followed by `::`, and then a
/// module name (an identifier). Parsing succeeds if and only if `s` matches this format exactly,
/// with no remaining input. This function is intended for use within the authority codebases.
pub fn parse_sui_module_id(s: &str) -> anyhow::Result<ModuleId> {
    use crate::ic_sui::move_core_types::parsing::types::ParsedModuleId;
    ParsedModuleId::parse(s)?.into_module_id(&resolve_address)
}

/// Parse `s` as a fully-qualified name: A Module ID (see `parse_sui_module_id`), followed by `::`,
/// and then an identifier (for the module member). Parsing succeeds if and only if `s` matches this
/// format exactly, with no remaining input. This function is intended for use within the authority
/// codebases.
pub fn parse_sui_fq_name(s: &str) -> anyhow::Result<(ModuleId, String)> {
    use crate::ic_sui::move_core_types::parsing::types::ParsedFqName;
    ParsedFqName::parse(s)?.into_fq_name(&resolve_address)
}

/// Parse `s` as a struct type: A fully-qualified name, optionally followed by a list of type
/// parameters (types -- see `parse_sui_type_tag`, separated by commas, surrounded by angle
/// brackets). Parsing succeeds if and only if `s` matches this format exactly, with no remaining
/// input. This function is intended for use within the authority codebase.
pub fn parse_sui_struct_tag(s: &str) -> anyhow::Result<StructTag> {
    use crate::ic_sui::move_core_types::parsing::types::ParsedStructType;
    ParsedStructType::parse(s)?.into_struct_tag(&resolve_address)
}

/// Parse `s` as a type: Either a struct type (see `parse_sui_struct_tag`), a primitive type, or a
/// vector with a type parameter. Parsing succeeds if and only if `s` matches this format exactly,
/// with no remaining input. This function is intended for use within the authority codebase.
pub fn parse_sui_type_tag(s: &str) -> anyhow::Result<TypeTag> {
    use crate::ic_sui::move_core_types::parsing::types::ParsedType;
    ParsedType::parse(s)?.into_type_tag(&resolve_address)
}

/// Resolve well-known named addresses into numeric addresses.
pub fn resolve_address(addr: &str) -> Option<AccountAddress> {
    match addr {
        "deepbook" => Some(DEEPBOOK_ADDRESS),
        "std" => Some(MOVE_STDLIB_ADDRESS),
        "sui" => Some(SUI_FRAMEWORK_ADDRESS),
        "sui_system" => Some(SUI_SYSTEM_ADDRESS),
        "bridge" => Some(BRIDGE_ADDRESS),
        _ => None,
    }
}

pub trait MoveTypeTagTrait {
    fn get_type_tag() -> TypeTag;
}

impl MoveTypeTagTrait for u8 {
    fn get_type_tag() -> TypeTag {
        TypeTag::U8
    }
}

impl MoveTypeTagTrait for u64 {
    fn get_type_tag() -> TypeTag {
        TypeTag::U64
    }
}

impl MoveTypeTagTrait for ObjectID {
    fn get_type_tag() -> TypeTag {
        TypeTag::Address
    }
}

impl MoveTypeTagTrait for SuiAddress {
    fn get_type_tag() -> TypeTag {
        TypeTag::Address
    }
}

impl<T: MoveTypeTagTrait> MoveTypeTagTrait for Vec<T> {
    fn get_type_tag() -> TypeTag {
        TypeTag::Vector(Box::new(T::get_type_tag()))
    }
}
