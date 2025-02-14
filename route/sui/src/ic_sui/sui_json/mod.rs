use std::fmt::{Debug, Formatter};

// use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct SuiJsonValue(JsonValue);

impl Debug for SuiJsonValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
