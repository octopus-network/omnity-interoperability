use crate::*;
use tendermint::{
    abci::{self, Event},
    block, Hash,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxCommitResponse {
    /// `CheckTx` result
    pub check_tx: abci::response::CheckTx,

    /// Result of executing the transaction.
    ///
    /// The JSON field carrying this data is named `deliver_tx` in
    /// CometBFT versions before 0.38.
    #[serde(alias = "deliver_tx")]
    pub tx_result: abci::types::ExecTxResult,

    /// Transaction
    pub hash: Hash,

    /// Height
    pub height: block::Height,
}

impl TxCommitResponse {
    pub fn assert_event_exist(&self, event: &Event) -> Result<&Event> {
        self.tx_result
            .events
            .iter()
            .find(|&e| e.kind.eq(&event.kind))
            .ok_or(RouteError::EventNotFound(event.kind.clone()))
    }
}
