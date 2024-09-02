use crate::*;
use tendermint::{
    abci::{self, Event},
    block, serializers, tx, Hash,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxResultByHashResponse {
    /// The hash of the transaction.
    ///
    /// Deserialized from a hex-encoded string (there is a discrepancy between
    /// the format used for the request and the format used for the response in
    /// the Tendermint RPC).
    pub hash: Hash,
    pub height: block::Height,
    pub index: u32,
    pub tx_result: abci::types::ExecTxResult,
    #[serde(with = "serializers::bytes::base64string")]
    pub tx: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<tx::Proof>,
}

impl TxResultByHashResponse {
    pub fn find_first_event_by_kind(&self, kind: String) -> Option<Event> {
        self.tx_result
            .events
            .iter()
            .find(|&e| e.kind.eq(&kind))
            .cloned()
    }

    pub fn assert_event_exist(&self, event: &Event) -> Result<&Event> {
        self.tx_result
            .events
            .iter()
            .find(|&e| e.kind.eq(&event.kind))
            .ok_or(RouteError::EventNotFound(event.kind.clone()))
    }
}