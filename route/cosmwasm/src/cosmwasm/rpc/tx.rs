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
}

#[test]
pub fn test_query_tx_by_hash() {
    const query_tx_str: &str = r#"{"jsonrpc":"2.0","id":-1,"result":{"hash":"4FDE39D4C51D561473C03E090113BF9B4CAC3EE3D746B12F69ABC234DE008020","height":"10939065","index":0,"tx_result":{"code":0,"data":"Ei4KLC9jb3Ntd2FzbS53YXNtLnYxLk1zZ0V4ZWN1dGVDb250cmFjdFJlc3BvbnNl","log":"","info":"","gas_wanted":"250000","gas_used":"225593","events":[{"type":"coin_spent","attributes":[{"key":"spender","value":"osmo13s0f55s8ppwm35npn53pkndphzyctfl7gu8q9d","index":true},{"key":"amount","value":"1000uosmo","index":true}]},{"type":"coin_received","attributes":[{"key":"receiver","value":"osmo17xpfvakm2amg962yls6f84z3kell8c5lczssa0","index":true},{"key":"amount","value":"1000uosmo","index":true}]},{"type":"transfer","attributes":[{"key":"recipient","value":"osmo17xpfvakm2amg962yls6f84z3kell8c5lczssa0","index":true},{"key":"sender","value":"osmo13s0f55s8ppwm35npn53pkndphzyctfl7gu8q9d","index":true},{"key":"amount","value":"1000uosmo","index":true}]},{"type":"message","attributes":[{"key":"sender","value":"osmo13s0f55s8ppwm35npn53pkndphzyctfl7gu8q9d","index":true}]},{"type":"tx","attributes":[{"key":"fee","value":"1000uosmo","index":true}]},{"type":"tx","attributes":[{"key":"acc_seq","value":"osmo13s0f55s8ppwm35npn53pkndphzyctfl7gu8q9d/14422","index":true}]},{"type":"tx","attributes":[{"key":"signature","value":"yI+qD/PjOhda8OYRuta4zAoVAkd0kAbnreerE5R0u1AJuATubwrFbKWDAOz2rSVewMgPpBDjljYju1tpZ5P55g==","index":true}]},{"type":"message","attributes":[{"key":"action","value":"/cosmwasm.wasm.v1.MsgExecuteContract","index":true},{"key":"sender","value":"osmo13s0f55s8ppwm35npn53pkndphzyctfl7gu8q9d","index":true},{"key":"module","value":"wasm","index":true},{"key":"msg_index","value":"0","index":true}]},{"type":"execute","attributes":[{"key":"_contract_address","value":"osmo1zc29309zydx8cjnmahv5uuw2jmjzdkrru59xxa3f8zzmeqwqr00seuxnwh","index":true},{"key":"msg_index","value":"0","index":true}]}],"codespace":""},"tx":"Cv4FCr8FCiQvY29zbXdhc20ud2FzbS52MS5Nc2dFeGVjdXRlQ29udHJhY3QSlgUKK29zbW8xM3MwZjU1czhwcHdtMzVucG41M3BrbmRwaHp5Y3RmbDdndThxOWQSP29zbW8xemMyOTMwOXp5ZHg4Y2pubWFodjV1dXcyam1qemRrcnJ1NTl4eGEzZjh6em1lcXdxcjAwc2V1eG53aBqlBHsicG9zdF9wcmljZXMiOnsicHJpY2VzIjpbeyJpbnB1dCI6ImZhY3Rvcnkvb3NtbzEzczBmNTVzOHBwd20zNW5wbjUzcGtuZHBoenljdGZsN2d1OHE5ZC91dXNkdCIsIm91dHB1dCI6ImZhY3Rvcnkvb3NtbzEzczBmNTVzOHBwd20zNW5wbjUzcGtuZHBoenljdGZsN2d1OHE5ZC91ZXRoIiwicHJpY2UiOiIzNzQ5MjUwMTQuOTk3MDAwNjM0NjcwMjU3NTY4In0seyJpbnB1dCI6ImZhY3Rvcnkvb3NtbzEzczBmNTVzOHBwd20zNW5wbjUzcGtuZHBoenljdGZsN2d1OHE5ZC91YnRjIiwib3V0cHV0IjoiZmFjdG9yeS9vc21vMTNzMGY1NXM4cHB3bTM1bnBuNTNwa25kcGh6eWN0Zmw3Z3U4cTlkL3VldGgiLCJwcmljZSI6IjIyNDM4NjMwMzQ2MC4wMzY4MDQxOTkyMTg3NSJ9LHsiaW5wdXQiOiJmYWN0b3J5L29zbW8xM3MwZjU1czhwcHdtMzVucG41M3BrbmRwaHp5Y3RmbDdndThxOWQvdXVzZHQiLCJvdXRwdXQiOiJmYWN0b3J5L29zbW8xM3MwZjU1czhwcHdtMzVucG41M3BrbmRwaHp5Y3RmbDdndThxOWQvdWJ0YyIsInByaWNlIjoiMC4wMDE2NzEzMTg1MDMxNjcxNDkifV19fRI6TWxqc0pUeXRwU0pNdXNuaERFVlNJbUJZZFNRVFR1WVFOdUJqYm5SRWN2TVZ2dEdEeHNZYmtMRUtpbhJoClEKRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiECKIwKDdWKX2ZkyYv9drCNrJk8SfIlv1qvtkihvueg14ESBAoCCAEY1nASEwoNCgV1b3NtbxIEMTAwMBCQoQ8aQMiPqg/z4zoXWvDmEbrWuMwKFQJHdJAG563nqxOUdLtQCbgE7m8KxWylgwDs9q0lXsDID6QQ45Y2I7tbaWeT+eY="}}"#;
    let wrapper: Wrapper<TxResultByHashResponse> = serde_json::from_str(&query_tx_str).unwrap();

    dbg!(&wrapper);
}
