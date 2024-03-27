use bitcoin::Amount;
use serde::Deserialize;

type RuneId = String;
#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub inputs: Vec<RsTxIn>,
    pub outputs: Vec<RsTxOut>,
}

#[derive(Deserialize, Debug)]
pub struct RsTxIn {
    #[serde(with = "bitcoin::amount::serde::as_btc")]
    pub value: Amount,
    pub address: String,
    pub runes: Vec<(RuneId, u128)>,
}

#[derive(Deserialize, Debug)]
pub struct RsTxOut {
    #[serde(with = "bitcoin::amount::serde::as_btc")]
    pub value: Amount,
    pub address: Option<String>,
    pub op_return: Option<Runestone>,
    pub runes: Vec<(RuneId, u128)>,
}

#[derive(Deserialize, Debug)]
pub struct Runestone {
    pub cenotaph: bool,
    pub claim: Option<RuneId>,
    pub default_output: Option<u32>,
    pub edicts: Vec<Edict>,
    pub etching: Option<Etching>,
}

#[derive(Deserialize, Debug)]
pub struct Edict {
    pub id: RuneId,
    pub amount: u128,
    pub output: u32,
}

#[derive(Deserialize, Debug)]
pub struct Etching {
    pub divisibility: u8,
    pub mint: Option<Mint>,
    pub rune: Option<String>,
    pub spacers: u32,
    pub symbol: Option<char>,
}

#[derive(Deserialize, Debug)]
pub struct Mint {
    pub deadline: Option<u32>,
    pub limit: Option<u128>,
    pub term: Option<u32>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct RunesBalance {
    pub rune_id: String,
    pub address: String,
    pub vout: u32,
    pub amount: u128,
}

impl Transaction {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn get_runes_balances(&self) -> Vec<RunesBalance> {
        self.outputs
            .iter()
            .enumerate()
            .map(|(vout, output)| {
                output
                    .runes
                    .iter()
                    .map(|(rune_id, amount)| RunesBalance {
                        rune_id: rune_id.clone(),
                        address: output.address.clone().unwrap(),
                        vout: vout as u32,
                        amount: *amount,
                    })
                    .collect::<Vec<RunesBalance>>()
            })
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_transaction() {
        let json = json!({
            "inputs": [
                {
                    "runes": [["102:1", 21000000]],
                    "value": 0.0001,
                    "address": "bcrt1prrjgtxv4felauzampzhf7kvw3pwel75dsqcmuvptwugcr2knazps2342kq"
                },
                {
                    "runes": [],
                    "value": 49.999898,
                    "address": "bcrt1p3cpnh26x2xqfm7d0up7c6qxs9u2786hz2ddxvepm7j024v58ytys26fwar"
                }
            ],
            "outputs": [
                {
                    "runes": [],
                    "value": 0.0,
                    "address": null,
                    "op_return": {
                        "burn": false,
                        "claim": null,
                        "edicts": [{"id": "102:1", "amount": 7, "output": 2}],
                        "etching": null,
                        "default_output": null
                    }
                },
                {
                    "runes": [["102:1", 20999993]],
                    "value": 0.0001,
                    "address": "bcrt1pfqmk3a2s2my84t7zfv6vc6t3dtm35zrhlajsk5xuauasdlx3ywsszr8w7c",
                    "op_return": null
                },
                {
                    "runes": [["102:1", 7]],
                    "value": 0.0001,
                    "address": "bcrt1qnwc03kekz4zexmtd69fffy6ap6pl3x4xwagdqf",
                    "op_return": null
                },
                {
                    "runes": [],
                    "value": 49.99979529,
                    "address": "bcrt1pw673t0ktvdns86xgghef25jz4xeaewh24mgvc6yk2f26heuat5yqzmqw75",
                    "op_return": null
                }
            ]
        });

        let transaction: Transaction = serde_json::from_value(json).unwrap();

        assert_eq!(transaction.outputs[2].runes[0].0, "102:1");
        assert_eq!(transaction.outputs[2].runes[0].1, 7);

        let runes_balances = transaction.get_runes_balances();
        assert_eq!(runes_balances.len(), 2);
        assert_eq!(
            runes_balances[0],
            RunesBalance {
                rune_id: "102:1".into(),
                address: "bcrt1pfqmk3a2s2my84t7zfv6vc6t3dtm35zrhlajsk5xuauasdlx3ywsszr8w7c".into(),
                vout: 1,
                amount: 20999993,
            }
        );
        assert_eq!(
            runes_balances[1],
            RunesBalance {
                rune_id: "102:1".into(),
                address: "bcrt1qnwc03kekz4zexmtd69fffy6ap6pl3x4xwagdqf".into(),
                vout: 2,
                amount: 7,
            }
        );
    }
}
