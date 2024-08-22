//! JSON-RPC versions

use core::{
    fmt::{self, Display},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::error::RouteError;

/// Supported JSON-RPC version
const SUPPORTED_VERSION: &str = "2.0";

/// JSON-RPC version
// TODO(tarcieri): add restrictions/validations on these formats? Use an `enum`?
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Version(String);

impl Version {
    /// Get the currently supported JSON-RPC version
    pub fn current() -> Self {
        Version(SUPPORTED_VERSION.to_owned())
    }

    /// Is this JSON-RPC version supported?
    pub fn is_supported(&self) -> bool {
        self.0.eq(SUPPORTED_VERSION)
    }

    /// Ensure we have a supported version or return an error
    pub fn ensure_supported(&self) -> Result<(), RouteError> {
        if self.is_supported() {
            Ok(())
        } else {
            Err(RouteError::CustomError(format!(
                "Unsupported JSON-RPC version: {}",
                self.0
            )))
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Version {
    type Err = RouteError;

    fn from_str(s: &str) -> Result<Self, RouteError> {
        Ok(Version(s.to_owned()))
    }
}
