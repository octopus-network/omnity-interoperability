use bitcoin::absolute::LockTime;
use bitcoin::hashes::Hash as _;
use bitcoin::secp256k1::{self};
use bitcoin::sighash::SighashCache;
use bitcoin::transaction::Version;
use bitcoin::{Address, Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness};

use crate::custom_to_bitcoin::CustomToBitcoinError;
use crate::custom_to_bitcoin::CustomToBitcoinError::SignFailed;
use crate::ord::builder::signer::MixSigner;
use crate::ord::builder::Utxo;

#[allow(dead_code)]
pub async fn spend_utxo_transaction(
    signer: &MixSigner,
    recipient: Address,
    utxo_value: Amount,
    inputs: Vec<Utxo>,
) -> Result<Transaction, CustomToBitcoinError> {

    let tx_out = vec![
        TxOut {
            value: utxo_value,
            script_pubkey: recipient.script_pubkey(),
        },
    ];

    let tx_in = inputs
        .iter()
        .map(|input| TxIn {
            previous_output: OutPoint {
                txid: input.id,
                vout: input.index,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::from_consensus(0xffffffff),
            witness: Witness::new(),
        })
        .collect();

    let unsigned_tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: tx_in,
        output: tx_out,
    };

    let tx = sign_transaction(
        signer,
        unsigned_tx,
        inputs,
        &signer.signer_addr.script_pubkey(),
    )
    .await?;
    Ok(tx)
}

async fn sign_transaction(
    signer: &MixSigner,
    unsigned_tx: Transaction,
    inputs: Vec<Utxo>,
    sender_script_pubkey: &ScriptBuf,
) -> Result<Transaction, CustomToBitcoinError> {
    let mut hash = SighashCache::new(unsigned_tx);

    for (index, input) in inputs.iter().enumerate() {
        let signature_hash = hash
            .p2wpkh_signature_hash(
                index,
                sender_script_pubkey,
                input.amount,
                bitcoin::EcdsaSighashType::All,
            )
            .map_err(|e| SignFailed(e.to_string()))?;

        let message = secp256k1::Message::from_digest(signature_hash.to_byte_array());
        let signature = signer
            .sign_with_ecdsa(message)
            .await
            .map_err(|e| SignFailed(e.to_string()))?;
        let signature = bitcoin::ecdsa::Signature::sighash_all(signature);

        // append witness to input
        let witness = Witness::p2wpkh(&signature, &signer.pubkey.inner.clone());
        *hash
            .witness_mut(index)
            .ok_or(CustomToBitcoinError::SignFailed(
                "withness none".to_string(),
            ))? = witness;
    }

    Ok(hash.into_transaction())
}
