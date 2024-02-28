use serde::Deserialize;

#[derive(Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub size: u32,
    pub vsize: u32,
    pub vout: Vec<TxOut>,
}

#[derive(Deserialize)]
pub struct TxOut {
    pub n: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: ScriptPubkey,
    pub runestone: Option<RuneStone>,
}

#[derive(Deserialize)]
pub struct ScriptPubkey {
    pub address: Option<String>,
}

#[derive(Deserialize)]
pub struct RuneStone {
    pub edicts: Option<Vec<Edict>>,
}

#[derive(Deserialize)]
pub struct Edict {
    pub rune: String,
    pub rune_id: String,
    pub amount: u128,
    pub output: u32,
}

pub struct RuneBalance {
    pub address: String,
    pub vout: u32,
    pub rune_id: u128,
    pub amount: u128,
}

impl Transaction {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn get_runes_balance(&self) -> Vec<RuneBalance> {
        let mut result = vec![];
        for out in &self.vout {
            if out.runestone.is_none() {
                continue;
            }
            let runestone = out.runestone.as_ref().unwrap();
            if runestone.edicts.is_none() {
                // Only one output of a transaction has runestone.
                return result;
            }
            let edicts = runestone.edicts.as_ref().unwrap();
            for edict in edicts {
                let vout = edict.output;
                let output = &self.vout[vout as usize];
                result.push(RuneBalance {
                    // The address must exist as long as the transaction is valid.
                    address: output
                        .script_pubkey
                        .address
                        .as_ref()
                        .expect("address shoud not be null")
                        .clone(),
                    vout,
                    rune_id: u128::from_str_radix(edict.rune_id.clone().as_str(), 16)
                        .expect("runes id should be legal hex number"),
                    amount: edict.amount,
                })
            }
        }
        result
    }
}
