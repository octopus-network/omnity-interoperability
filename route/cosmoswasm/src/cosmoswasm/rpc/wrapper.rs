use crate::*;

use super::{response_error::ResponseError, version::Version};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Wrapper<R> {
    /// JSON-RPC version
    pub jsonrpc: Version,

    /// Identifier included in request
    pub id: Id,

    /// Results of request (if successful)
    pub result: Option<R>,

    /// Error message if unsuccessful
    pub error: Option<ResponseError>,
}

impl<R> Wrapper<R> {
    /// Get JSON-RPC version
    pub fn version(&self) -> &Version {
        &self.jsonrpc
    }

    /// Get JSON-RPC ID
    #[allow(dead_code)]
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Convert this wrapper into the underlying error, if any
    pub fn into_error(self) -> Option<RouteError> {
        self.error.map(|e| RouteError::CustomError(e.to_string()))
    }

    /// Convert this wrapper into a result type
    pub fn into_result(self) -> Result<R> {
        // Ensure we're using a supported RPC version
        self.version().ensure_supported()?;

        if let Some(e) = self.error {
            Err(RouteError::CustomError(e.to_string()))
        } else if let Some(result) = self.result {
            Ok(result)
        } else {
            Err(RouteError::CustomError(
                "No result or error in response".to_string(),
            ))
        }
    }

    pub fn new_with_id(id: Id, result: Option<R>, error: Option<ResponseError>) -> Self {
        Self {
            jsonrpc: Version::current(),
            id,
            result,
            error,
        }
    }
}
