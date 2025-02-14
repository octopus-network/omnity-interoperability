use crate::ic_sui::sui_types::object::Owner;
use crate::ic_sui::sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber},
    digests::{ObjectDigest, TransactionDigest},
};

use serde::{Deserialize, Serialize};

/// A type containing all of the information needed to work with a deleted shared object in
/// execution and when committing the execution effects of the transaction. This holds:
/// 0. The object ID of the deleted shared object.
/// 1. The version of the shared object.
/// 2. Whether the object appeared as mutable (or owned) in the transaction, or as a read-only shared object.
/// 3. The transaction digest of the previous transaction that used this shared object mutably or
///    took it by value.
pub type DeletedSharedObjectInfo = (ObjectID, SequenceNumber, bool, TransactionDigest);

/// A sequence of information about deleted shared objects in the transaction's inputs.
pub type DeletedSharedObjects = Vec<DeletedSharedObjectInfo>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharedInput {
    Existing(ObjectRef),
    Deleted(DeletedSharedObjectInfo),
    Cancelled((ObjectID, SequenceNumber)),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DynamicallyLoadedObjectMetadata {
    pub version: SequenceNumber,
    pub digest: ObjectDigest,
    pub owner: Owner,
    pub storage_rebate: u64,
    pub previous_transaction: TransactionDigest,
}
