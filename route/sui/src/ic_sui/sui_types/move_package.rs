#![allow(unused)]

use crate::ic_sui::sui_types::execution_status::PackageUpgradeError;
use crate::ic_sui::sui_types::{
    base_types::{ObjectID, SequenceNumber},
    error::{ExecutionError, ExecutionErrorKind},
    id::{ID, UID},
};

use crate::ic_sui::move_core_types::language_storage::ModuleId;
use crate::ic_sui::move_core_types::{account_address::AccountAddress, identifier::IdentStr};
use crate::ident_str;

use serde::{Deserialize, Serialize};
// use serde_json::Value;
use serde_with::serde_as;
use serde_with::Bytes;
use std::collections::BTreeMap;

pub const PACKAGE_MODULE_NAME: &IdentStr = ident_str!("package");
pub const UPGRADECAP_STRUCT_NAME: &IdentStr = ident_str!("UpgradeCap");
pub const UPGRADETICKET_STRUCT_NAME: &IdentStr = ident_str!("UpgradeTicket");
pub const UPGRADERECEIPT_STRUCT_NAME: &IdentStr = ident_str!("UpgradeReceipt");

#[derive(Clone, Debug)]
/// Additional information about a function
pub struct FnInfo {
    /// If true, it's a function involved in testing (`[test]`, `[test_only]`, `[expected_failure]`)
    pub is_test: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
/// Uniquely identifies a function in a module
pub struct FnInfoKey {
    pub fn_name: String,
    pub mod_addr: AccountAddress,
}

/// A map from function info keys to function info
pub type FnInfoMap = BTreeMap<FnInfoKey, FnInfo>;

/// Identifies a struct and the module it was defined in
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, Hash)]
pub struct TypeOrigin {
    pub module_name: String,
    // `struct_name` alias to support backwards compatibility with the old name
    #[serde(alias = "struct_name")]
    pub datatype_name: String,
    pub package: ObjectID,
}

/// Upgraded package info for the linkage table
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct UpgradeInfo {
    /// ID of the upgraded packages
    pub upgraded_id: ObjectID,
    /// Version of the upgraded package
    pub upgraded_version: SequenceNumber,
}

// serde_bytes::ByteBuf is an analog of Vec<u8> with built-in fast serialization.
#[serde_as]
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MovePackage {
    id: ObjectID,
    /// Most move packages are uniquely identified by their ID (i.e. there is only one version per
    /// ID), but the version is still stored because one package may be an upgrade of another (at a
    /// different ID), in which case its version will be one greater than the version of the
    /// upgraded package.
    ///
    /// Framework packages are an exception to this rule -- all versions of the framework packages
    /// exist at the same ID, at increasing versions.
    ///
    /// In all cases, packages are referred to by move calls using just their ID, and they are
    /// always loaded at their latest version.
    version: SequenceNumber,
    // TODO use session cache
    #[serde_as(as = "BTreeMap<_, Bytes>")]
    module_map: BTreeMap<String, Vec<u8>>,

    /// Maps struct/module to a package version where it was first defined, stored as a vector for
    /// simple serialization and deserialization.
    type_origin_table: Vec<TypeOrigin>,

    // For each dependency, maps original package ID to the info about the (upgraded) dependency
    // version that this package is using
    linkage_table: BTreeMap<ObjectID, UpgradeInfo>,
}

/// Rust representation of `sui::package::UpgradeCap`.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeCap {
    pub id: UID,
    pub package: ID,
    pub version: u64,
    pub policy: u8,
}

/// Rust representation of `sui::package::UpgradeTicket`.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeTicket {
    pub cap: ID,
    pub package: ID,
    pub policy: u8,
    pub digest: Vec<u8>,
}

/// Rust representation of `sui::package::UpgradeReceipt`.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeReceipt {
    pub cap: ID,
    pub package: ID,
}

impl MovePackage {
    /// Create a package with all required data (including serialized modules, type origin and
    /// linkage tables) already supplied.
    pub fn new(
        id: ObjectID,
        version: SequenceNumber,
        module_map: BTreeMap<String, Vec<u8>>,
        max_move_package_size: u64,
        type_origin_table: Vec<TypeOrigin>,
        linkage_table: BTreeMap<ObjectID, UpgradeInfo>,
    ) -> Result<Self, ExecutionError> {
        let pkg = Self {
            id,
            version,
            module_map,
            type_origin_table,
            linkage_table,
        };
        let object_size = pkg.size() as u64;
        if object_size > max_move_package_size {
            return Err(ExecutionErrorKind::MovePackageTooBig {
                object_size,
                max_object_size: max_move_package_size,
            }
            .into());
        }
        Ok(pkg)
    }

    pub fn digest(&self, hash_modules: bool) -> [u8; 32] {
        // Self::compute_digest_for_modules_and_deps(
        //     self.module_map.values(),
        //     self.linkage_table
        //         .values()
        //         .map(|UpgradeInfo { upgraded_id, .. }| upgraded_id),
        //     hash_modules,
        // )
        todo!()
    }

    // Retrieve the module with `ModuleId` in the given package.
    // The module must be the `storage_id` or the call will return `None`.
    // Check if the address of the module is the same of the package
    // and return `None` if that is not the case.
    // All modules in a package share the address with the package.
    pub fn get_module(&self, storage_id: &ModuleId) -> Option<&Vec<u8>> {
        if self.id != ObjectID::from(*storage_id.address()) {
            None
        } else {
            self.module_map.get(&storage_id.name().to_string())
        }
    }

    /// Return the size of the package in bytes
    pub fn size(&self) -> usize {
        let module_map_size = self
            .module_map
            .iter()
            .map(|(name, module)| name.len() + module.len())
            .sum::<usize>();
        let type_origin_table_size = self
            .type_origin_table
            .iter()
            .map(
                |TypeOrigin {
                     module_name,
                     datatype_name: struct_name,
                     ..
                 }| module_name.len() + struct_name.len() + ObjectID::LENGTH,
            )
            .sum::<usize>();

        let linkage_table_size = self.linkage_table.len()
            * (ObjectID::LENGTH + (ObjectID::LENGTH + 8/* SequenceNumber */));

        8 /* SequenceNumber */ + module_map_size + type_origin_table_size + linkage_table_size
    }

    pub fn id(&self) -> ObjectID {
        self.id
    }

    pub fn version(&self) -> SequenceNumber {
        self.version
    }

    pub fn decrement_version(&mut self) {
        self.version.decrement();
    }

    pub fn increment_version(&mut self) {
        self.version.increment();
    }

    /// Approximate size of the package in bytes. This is used for gas metering.
    pub fn object_size_for_gas_metering(&self) -> usize {
        self.size()
    }

    pub fn serialized_module_map(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.module_map
    }

    pub fn type_origin_table(&self) -> &Vec<TypeOrigin> {
        &self.type_origin_table
    }

    pub fn type_origin_map(&self) -> BTreeMap<(String, String), ObjectID> {
        self.type_origin_table
            .iter()
            .map(
                |TypeOrigin {
                     module_name,
                     datatype_name: struct_name,
                     package,
                 }| { ((module_name.clone(), struct_name.clone()), *package) },
            )
            .collect()
    }

    pub fn linkage_table(&self) -> &BTreeMap<ObjectID, UpgradeInfo> {
        &self.linkage_table
    }
}
