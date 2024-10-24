use std::str::FromStr;

use base64::Engine;
use bitcoin::psbt::Error;
use bitcoin::{Address, Amount, Psbt, PublicKey, Transaction, Txid};
use bitcoin::consensus::Encodable;
use candid::Deserialize;
use rust_decimal::Decimal;
use serde::Serialize;
use serde_with::serde_as;

use omnity_types::TokenId;

use crate::custom_to_bitcoin::CustomToBitcoinError::{
    ArgumentError, BuildTransactionFailed, SignFailed,
};
use crate::custom_to_bitcoin::{
    build_transfer_transfer, CustomToBitcoinError, CustomToBitcoinResult,
};
use crate::ord::builder::fees::Fees;
use crate::ord::builder::{
    CreateCommitTransactionArgsV2, OrdTransactionBuilder, RevealTransactionArgs,
    SignCommitTransactionArgs, Utxo,
};
use crate::ord::inscription::brc20::Brc20;
use crate::ord::parser::POSTAGE;
use crate::state::{bitcoin_network, deposit_addr, deposit_pubkey, mutate_state, read_state};
use crate::types::{FeesArgs, UtxoArgs};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Brc20Transfer {
    pub op: String,
    /// Protocol (required): Helps other systems identify and process brc-20 events
    #[serde(rename = "p")]
    protocol: String,
    /// Ticker (required): 4 or 5 letter identifier of the brc-20
    pub tick: String,
    /// Amount to transfer (required): States the amount of the brc-20 to transfer.
    pub amt: String,
    #[serde(rename = "ref", skip_serializing)]
    pub refx: String,
    #[serde(skip_serializing)]
    pub chain: String,
    #[serde(skip_serializing)]
    pub ext: String,
}

pub async fn build_commit(
    session_key: String,
    vins: Vec<UtxoArgs>,
    token_id: TokenId,
    amount: String,
    sender: String,
    target_chain: String,
    receiver: String,
    fee: FeesArgs,
) -> CustomToBitcoinResult<String> {
    let fees: Fees = fee.into();
    let vins = vins.into_iter().map(|u| u.into()).collect::<Vec<Utxo>>();
    let sender_addr = Address::from_str(&sender).unwrap().assume_checked();
    let token = read_state(|s| s.tokens.get(&token_id).cloned())
        .ok_or(ArgumentError("token not found".to_string()))?;
    let amount = Decimal::from_str(amount.as_str())
        .map_err(|e| CustomToBitcoinError::ArgumentError(e.to_string()))?;
    let key_id = read_state(|s| s.ecdsa_key_name.clone());
    let mut builder = OrdTransactionBuilder::p2tr(
        PublicKey::from_str(deposit_pubkey().as_str()).unwrap(),
        key_id,
        deposit_addr(),
    );
    let transfer = Brc20Transfer {
        op: "transfer".to_string(),
        protocol: "brc-20".to_string(),
        tick: token.symbol,
        amt: amount.normalize().to_string(),
        refx: receiver,
        chain: target_chain,
        ext: "bridge-out".to_string(),
    };
    let commit_tx = builder
        .build_commit_transaction_with_fixed_fees(
            bitcoin_network(),
            CreateCommitTransactionArgsV2 {
                inputs: vins.clone(),
                inscription: Brc20::transfer(token.name.clone(), amount),
                txin_script_pubkey: sender_addr.script_pubkey(),
                leftovers_recipient: sender_addr,
                fees: fees.clone(),
            },
        )
        .await
        .map_err(|e| BuildTransactionFailed(e.to_string()))?;
    let unsigned_tx = commit_tx.unsigned_tx.clone();
    mutate_state(|s|{
        s.temp_psbt_state.insert(session_key.clone(), commit_tx);
        s.temp_psbt_builder.insert(session_key, builder);
    });
    let psbt = bitcoin::psbt::Psbt::from_unsigned_tx(unsigned_tx)
        .map_err(|e| ArgumentError(e.to_string()))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(psbt.serialize().as_slice()))
}

pub async fn build_reveal_transfer(
    session_key: String,
    commit_tx_id: String,
    fee: FeesArgs,
) -> CustomToBitcoinResult<Vec<String>> {
    let fees: Fees = fee.into();
    let receiver = read_state(|s| s.deposit_addr.clone().unwrap());
    let key_id = read_state(|s| s.ecdsa_key_name.clone());
    let mut builder = read_state(|s|s.temp_psbt_builder.get(&session_key).cloned())
        .ok_or(ArgumentError("session key".to_string()))?;
    let commit_tx = read_state(|s| s.temp_psbt_state.get(&session_key).cloned())
        .ok_or(ArgumentError("session key".to_string()))?;
    let reveal_transaction = builder
        .build_reveal_transaction(RevealTransactionArgs {
            input: Utxo {
                id: Txid::from_str(&commit_tx_id).unwrap(),
                index: 0,
                amount: commit_tx.reveal_balance,
            },
            spend_fee: fees.spend_fee,
            recipient_address: deposit_addr(), // NOTE: it's correct, see README.md to read about how transfer works
            redeem_script: commit_tx.redeem_script,
        })
        .await
        .map_err(|e| BuildTransactionFailed(e.to_string()))?;
    let real_utxo = Utxo {
        id: reveal_transaction.txid(),
        index: 0,
        amount: Amount::from_sat(POSTAGE + fees.spend_fee.to_sat()),
    };
    let transfer_trasaction = build_transfer_transfer(&receiver, real_utxo, None).await?;
    let psbt =
        Psbt::from_unsigned_tx(transfer_trasaction).map_err(|e| ArgumentError(e.to_string()))?;
    let transfer_str =
        base64::engine::general_purpose::STANDARD.encode(psbt.serialize().as_slice());
    mutate_state(|s| {
        s.temp_psbt_state.remove(&session_key);
        s.temp_psbt_builder.remove(&session_key);
    });
    let mut bytes = vec![];
    reveal_transaction.consensus_encode(&mut bytes).map_err(|e|ArgumentError(e.to_string()))?;
    let txs = vec![hex::encode(bytes), transfer_str];
    Ok(txs)
}
