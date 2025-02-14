use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use super::{digests::Digest, transaction::ProtocolVersion};

/// Models the set of protocol versions supported by a validator.
/// The `sui-node` binary will always use the SYSTEM_DEFAULT constant, but for testing we need
/// to be able to inject arbitrary versions into SuiNode.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SupportedProtocolVersions {
    pub min: ProtocolVersion,
    pub max: ProtocolVersion,
}

impl SupportedProtocolVersions {
    pub const SYSTEM_DEFAULT: Self = Self {
        min: ProtocolVersion::MIN,
        max: ProtocolVersion::MAX,
    };

    /// Use by VersionedProtocolMessage implementors to describe in which range of versions a
    /// message variant is supported.
    pub fn new_for_message(min: u64, max: u64) -> Self {
        let min = ProtocolVersion::new(min);
        let max = ProtocolVersion::new(max);
        Self { min, max }
    }

    pub fn new_for_testing(min: u64, max: u64) -> Self {
        let min = min.into();
        let max = max.into();
        Self { min, max }
    }

    pub fn is_version_supported(&self, v: ProtocolVersion) -> bool {
        v.as_u64() >= self.min.as_u64() && v.as_u64() <= self.max.as_u64()
    }

    pub fn as_range(&self) -> RangeInclusive<u64> {
        self.min.as_u64()..=self.max.as_u64()
    }

    pub fn truncate_below(self, v: ProtocolVersion) -> Self {
        let min = std::cmp::max(self.min, v);
        Self { min, max: self.max }
    }
}

///// Models the set of protocol versions supported by a validator.
///// The `sui-node` binary will always use the SYSTEM_DEFAULT constant, but for testing we need
///// to be able to inject arbitrary versions into SuiNode.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SupportedProtocolVersionsWithHashes {
    pub versions: Vec<(ProtocolVersion, Digest)>,
}
