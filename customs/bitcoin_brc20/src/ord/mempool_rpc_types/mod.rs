use bitcoin::absolute::LockTime;
use bitcoin::transaction::Version;
use bitcoin::{Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TxInfo {
    pub version: i32,
    pub locktime: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub status: TxStatus,
    pub fee: u128,
    pub size: u32,
    pub weight: u32,
    pub sigops: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TxStatus {
    pub confirmed: bool,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Vin {
    pub txid: String,
    pub vout: u32,
    pub sequence: u32,
    pub is_coinbase: bool,
    pub prevout: Prevout,
    pub witness: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Prevout {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Vout {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

impl TryFrom<TxInfo> for Transaction {
    type Error = anyhow::Error;

    fn try_from(info: TxInfo) -> Result<Self, Self::Error> {
        let version = Version(info.version);
        let lock_time = LockTime::from_consensus(info.locktime);

        let mut tx_in = Vec::with_capacity(info.vin.len());
        for input in info.vin {
            let txid = Txid::from_str(&input.txid)?;
            let vout = input.vout;
            let script_sig = ScriptBuf::from_hex(&input.prevout.scriptpubkey)?;

            let mut witness = Witness::new();
            for item in input.witness {
                witness.push(ScriptBuf::from_hex(&item)?);
            }

            let tx_input = TxIn {
                previous_output: OutPoint { txid, vout },
                script_sig,
                sequence: Sequence(input.sequence),
                witness,
            };

            tx_in.push(tx_input);
        }

        let mut tx_out = Vec::with_capacity(info.vout.len());
        for output in info.vout {
            let script_pubkey = ScriptBuf::from_hex(&output.scriptpubkey)?;
            let value = Amount::from_sat(output.value);

            tx_out.push(TxOut {
                script_pubkey,
                value,
            });
        }

        Ok(Transaction {
            version,
            lock_time,
            input: tx_in,
            output: tx_out,
        })
    }
}
