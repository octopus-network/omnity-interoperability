use bitcoin::Amount;
use serde::Deserialize;

type RuneId = String;

#[derive(Deserialize, Debug)]
pub struct TxResponse {
    pub transactions: Vec<TxResult>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TxResult {
    pub transaction: Transaction,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    pub inputs: Vec<RsTxIn>,
    pub outputs: Vec<RsTxOut>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RsTxIn {
    #[serde(with = "bitcoin::amount::serde::as_btc")]
    pub value: Amount,
    pub address: String,
    pub runes: Vec<(RuneId, u128)>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RsTxOut {
    #[serde(with = "bitcoin::amount::serde::as_btc")]
    pub value: Amount,
    pub address: Option<String>,
    pub op_return: Option<Artifact>,
    pub runes: Vec<(RuneId, u128)>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum Artifact {
    Cenotaph(Cenotaph),
    Runestone(Runestone),
}

#[derive(Deserialize, Debug, Clone)]
pub struct Cenotaph {
    pub etching: Option<String>,
    pub flaw: Option<Flaw>,
    pub mint: Option<RuneId>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Flaw {
    EdictOutput,
    EdictRuneId,
    InvalidScript,
    Opcode,
    SupplyOverflow,
    TrailingIntegers,
    TruncatedField,
    UnrecognizedEvenTag,
    UnrecognizedFlag,
    Varint,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Runestone {
    pub edicts: Vec<Edict>,
    pub etching: Option<Etching>,
    pub mint: Option<RuneId>,
    pub pointer: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Edict {
    pub id: RuneId,
    pub amount: u128,
    pub output: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Etching {
    pub divisibility: Option<u8>,
    pub premine: Option<u128>,
    pub rune: Option<String>,
    pub spacers: Option<u32>,
    pub symbol: Option<char>,
    pub terms: Option<Terms>,
    pub turbo: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Terms {
    pub amount: Option<u128>,
    pub cap: Option<u128>,
    pub height: (Option<u64>, Option<u64>),
    pub offset: (Option<u64>, Option<u64>),
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
        let response: TxResponse = serde_json::from_str(json_str)?;
        assert!(response.transactions.len() == 1, "Expected 1 transaction");
        Ok(response.transactions[0].transaction.clone())
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
        let json = json!(
         {"transactions" : [{
          "transaction":
          {
          "inputs": [
            {
              "runes": [
                [
                  "108:1",
                  100000
                ]
              ],
              "value": 0.0001,
              "address": "bcrt1pgyplwd4sea9kee9vdz9gc95evswxxmuwfpv67g64h53g5jvyt39supadnx"
            },
            {
              "runes": [],
              "value": 49.99979605,
              "address": "bcrt1pmthgfs90kez2wxmerpt5q6w4r60qpu59jun76rdzy8fw7n40tpxqrgwrp8"
            }
          ],
          "outputs": [
            {
              "runes": [],
              "value": 0,
              "address": null,
              "op_return": {
                "Runestone": {
                  "mint": null,
                  "edicts": [
                    {
                      "id": "108:1",
                      "amount": 700,
                      "output": 2
                    }
                  ],
                  "etching": null,
                  "pointer": null
                }
              }
            },
            {
              "runes": [
                [
                  "108:1",
                  99300
                ]
              ],
              "value": 0.0001,
              "address": "bcrt1pkhtp5hh6rxvnr7qjus30zm5l5vszxqm8vrdyk0xg09j3xd8ugq9q9p4yhj",
              "op_return": null
            },
            {
              "runes": [
                [
                  "108:1",
                  700
                ]
              ],
              "value": 0.0001,
              "address": "bcrt1qp5ezzetuwc4jtzjfc9w2t47n7yvhgl4xz842pf",
              "op_return": null
            },
            {
              "runes": [],
              "value": 49.99969344,
              "address": "bcrt1p75mc0x7zec4xnc8vezgx9e6ekukujzfztnes456rhhylwqwhd7zs5l2ekt",
              "op_return": null
            }
          ]
        }}]});

        let transaction = Transaction::from_json(&json.to_string()).unwrap();

        assert_eq!(transaction.outputs[2].runes[0].0, "108:1");
        assert_eq!(transaction.outputs[2].runes[0].1, 700);

        let runes_balances = transaction.get_runes_balances();
        assert_eq!(runes_balances.len(), 2);
        assert_eq!(
            runes_balances[0],
            RunesBalance {
                rune_id: "108:1".into(),
                address: "bcrt1pkhtp5hh6rxvnr7qjus30zm5l5vszxqm8vrdyk0xg09j3xd8ugq9q9p4yhj".into(),
                vout: 1,
                amount: 99300,
            }
        );
        assert_eq!(
            runes_balances[1],
            RunesBalance {
                rune_id: "108:1".into(),
                address: "bcrt1qp5ezzetuwc4jtzjfc9w2t47n7yvhgl4xz842pf".into(),
                vout: 2,
                amount: 700,
            }
        );
    }
}
