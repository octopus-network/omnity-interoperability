use bitcoin::absolute::LockTime;
use bitcoin::bip32::DerivationPath;
use bitcoin::script::{Builder as ScriptBuilder, PushBytesBuf};
use bitcoin::transaction::Version;
use bitcoin::{
    secp256k1, Address, Amount, FeeRate, Network, OutPoint, PublicKey, ScriptBuf, Sequence,
    Transaction, TxIn, TxOut, Txid, Witness, XOnlyPublicKey,
};
use log::debug;
use serde::{Deserialize, Serialize};
use crate::ord::builder::fees::MultisigConfig;
use crate::ord::builder::signer::MixSigner;
use crate::ord::builder::wallet::Wallet;
use crate::ord::builder::taproot::{generate_keypair, TaprootPayload};
use crate::ord::inscription::Inscription;
use crate::ord::parser::POSTAGE;
use crate::ord::parser::push_bytes::bytes_to_push_bytes;
use crate::ord::result::{OrdError, OrdResult};

pub mod wallet;
pub mod taproot;
pub mod fees;
pub mod signer;
pub mod spend_transaction;

/// Ordinal-aware transaction builder for arbitrary (`Nft`)
/// and `Brc20` inscriptions.
pub struct OrdTransactionBuilder {
    public_key: PublicKey,
    script_type: ScriptType,
    /// used to sign the reveal transaction when using P2TR
    taproot_payload: Option<TaprootPayload>,
    signer: Wallet,
}

#[derive(Debug)]
/// Arguments for creating a commit transaction
pub struct CreateCommitTransactionArgs<T>
    where
        T: Inscription,
{
    /// UTXOs to be used as inputs of the transaction
    pub inputs: Vec<Utxo>,
    /// Inscription to write
    pub inscription: T,
    /// Address to send the leftovers BTC of the trasnsaction
    pub leftovers_recipient: Address,
    /// Script pubkey of the inputs
    pub txin_script_pubkey: ScriptBuf,
    /// Current fee rate on the network
    pub fee_rate: FeeRate,
    /// Multisig configuration, if applicable
    pub multisig_config: Option<MultisigConfig>,
}

#[derive(Debug, Clone)]
pub struct SignCommitTransactionArgs {
    /// UTXOs to be used as inputs of the transaction
    pub inputs: Vec<Utxo>,
    /// Script pubkey of the inputs
    pub txin_script_pubkey: ScriptBuf,
}

#[derive(Debug, Clone)]
pub struct CreateCommitTransaction {
    /// The unsigned commit transaction
    pub unsigned_tx: Transaction,
    /// The redeem script to be used in the reveal transaction
    pub redeem_script: ScriptBuf,
    /// Balance to be passed to reveal transaction
    pub reveal_balance: Amount,
    /// Commit transaction fee
    pub commit_fee: Amount,
    /// Reveal transaction fee
    pub reveal_fee: Amount,
    /// Leftover amount to be sent to the leftovers recipient
    pub leftover_amount: Amount,
}

/// Arguments for creating a reveal transaction
#[derive(Debug, Clone)]
pub struct RevealTransactionArgs {
    /// Transaction input (output of commit transaction)
    pub input: Utxo,
    /// Recipient address of the inscription, only support P2PKH
    pub recipient_address: Address,
    /// The redeem script returned by `create_commit_transaction`
    pub redeem_script: ScriptBuf,
}

/// Type of the script to use. Both are supported, but P2WSH may not be supported by all the indexers
/// So P2TR is preferred
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptType {
    P2WSH,
    P2TR,
}

#[derive(Debug)]
pub enum RedeemScriptPubkey {
    Ecdsa(PublicKey),
    XPublickey(XOnlyPublicKey),
}

impl RedeemScriptPubkey {
    /// Encode the public key to a push bytes buffer
    pub fn encode(&self) -> OrdResult<PushBytesBuf> {
        let encoded_pubkey = match self {
            RedeemScriptPubkey::Ecdsa(pubkey) => bytes_to_push_bytes(&pubkey.to_bytes())?,
            RedeemScriptPubkey::XPublickey(pubkey) => bytes_to_push_bytes(&pubkey.serialize())?,
        };

        Ok(encoded_pubkey)
    }
}

impl OrdTransactionBuilder {
    pub fn new(public_key: PublicKey, script_type: ScriptType, signer: Wallet) -> Self {
        Self {
            public_key,
            script_type,
            taproot_payload: None,
            signer,
        }
    }

    pub fn signer(&self) -> MixSigner {
        self.signer.signer.clone()
    }
    /// A constructor that allows to set the taproot payload, in case the user wants to resume a previous session
    pub fn new_with_taproot_payload(
        public_key: PublicKey,
        script_type: ScriptType,
        signer: Wallet,
        taproot_payload: Option<TaprootPayload>,
    ) -> Self {
        Self {
            public_key,
            script_type,
            taproot_payload,
            signer,
        }
    }

    pub fn taproot_payload(&self) -> Option<&TaprootPayload> {
        self.taproot_payload.as_ref()
    }


    /// Sign the commit transaction
    pub async fn sign_commit_transaction(
        &mut self,
        unsigned_tx: Transaction,
        args: SignCommitTransactionArgs,
    ) -> OrdResult<Transaction> {
        // sign transaction and update witness
        self.signer
            .sign_commit_transaction(
                &self.public_key,
                &args.inputs,
                unsigned_tx,
                &args.txin_script_pubkey,
            )
            .await
    }



    /// Create the reveal transaction
    pub async fn build_reveal_transaction(
        &mut self,
        args: RevealTransactionArgs,
    ) -> OrdResult<Transaction> {
        // previous output
        let previous_output = OutPoint {
            txid: args.input.id,
            vout: args.input.index,
        };

        // tx out
        let tx_out = vec![TxOut {
            value: Amount::from_sat(POSTAGE),
            script_pubkey: args.recipient_address.script_pubkey(),
        }];

        // txin\
        let tx_in = vec![TxIn {
            previous_output,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::from_consensus(0xffffffff),
            witness: Witness::new(),
        }];

        // make transaction and sign it
        let unsigned_tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: tx_in,
            output: tx_out,
        };

        let tx = match self.taproot_payload.as_ref() {
            Some(taproot_payload) => self.signer.sign_reveal_transaction_schnorr(
                taproot_payload,
                &args.redeem_script,
                unsigned_tx,
            ),
            None => {
               panic!("taproot error");
            }
        }?;

        Ok(tx)
    }

    /// Generate redeem script from script pubkey and inscription
    fn generate_redeem_script<T>(
        &self,
        inscription: &T,
        pubkey: RedeemScriptPubkey,
    ) -> OrdResult<ScriptBuf>
        where
            T: Inscription,
    {
        Ok(inscription
            .generate_redeem_script(ScriptBuilder::new(), pubkey)?
            .into_script())
    }

    /// Initialize a new `OrdTransactionBuilder` with the given private key and use P2TR as script type (preferred).
    pub fn p2tr(public_key: PublicKey, key_id: String, address: Address) -> Self {
        let wallet = Wallet::new_with_signer(signer::MixSigner::new(key_id, public_key.clone(),address));
        Self::new(public_key, ScriptType::P2TR, wallet)
    }

    /// Creates the commit transaction with predetermined commit and reveal fees.
    pub async fn build_commit_transaction_with_fixed_fees<T>(
        &mut self,
        network: Network,
        args: CreateCommitTransactionArgsV2<T>,
    ) -> OrdResult<CreateCommitTransaction>
        where
            T: Inscription,
    {
        let secp_ctx = secp256k1::Secp256k1::new();

        // generate P2TR keyts
        let p2tr_keys = match self.script_type {
            ScriptType::P2WSH => None,
            ScriptType::P2TR => Some(generate_keypair(&secp_ctx).await
                .map_err(|e| OrdError::ManagementError(format!("code: {:?}, msg:{}", e.0, e.1)))?),
        };

        // generate redeem script pubkey based on the current script type
        let redeem_script_pubkey = match self.script_type {
            ScriptType::P2WSH => RedeemScriptPubkey::Ecdsa(self.public_key),
            ScriptType::P2TR => RedeemScriptPubkey::XPublickey(p2tr_keys.unwrap().1),
        };

        // calc balance
        // exceeding amount of transaction to send to leftovers recipient
        let input_amount = args
            .inputs
            .iter()
            .map(|input| input.amount.to_sat())
            .sum::<u64>();
        let leftover_amount = input_amount
            .checked_sub(POSTAGE)
            .and_then(|v| v.checked_sub(args.commit_fee.to_sat()))
            .and_then(|v| v.checked_sub(args.reveal_fee.to_sat()))
            .ok_or(OrdError::InsufficientBalance {
                available: input_amount,
                required: POSTAGE + args.commit_fee.to_sat() + args.reveal_fee.to_sat(),
            })?;
        debug!("leftover_amount: {leftover_amount}");

        let reveal_balance = POSTAGE + args.reveal_fee.to_sat();
        debug!("reveal_balance: {reveal_balance}");

        // get p2wsh or p2tr address for output of inscription
        let redeem_script = self.generate_redeem_script(&args.inscription, redeem_script_pubkey)?;
        debug!("redeem_script: {redeem_script}");
        let script_output_address = match self.script_type {
            ScriptType::P2WSH => Address::p2wsh(&redeem_script, network),
            ScriptType::P2TR => {
                let taproot_payload = TaprootPayload::build(
                    &secp_ctx,
                    p2tr_keys.unwrap().0,
                    p2tr_keys.unwrap().1,
                    &redeem_script,
                    reveal_balance,
                    network,
                )?;

                let address = taproot_payload.address.clone();
                self.taproot_payload = Some(taproot_payload);
                address
            }
        };
        debug!("script_output_address: {script_output_address}");

        let mut tx_out = vec![
            TxOut {
                value: Amount::from_sat(reveal_balance),
                script_pubkey: script_output_address.script_pubkey(),
            }
        ];
        if leftover_amount > 0 {
            tx_out.push( TxOut {
                value: Amount::from_sat(leftover_amount),
                script_pubkey: args.txin_script_pubkey.clone(),
            });
        }

        // txin
        let tx_in = args
            .inputs
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

        // make transaction and sign it
        let unsigned_tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: tx_in,
            output: tx_out,
        };

        Ok(CreateCommitTransaction {
            unsigned_tx,
            redeem_script,
            reveal_balance: Amount::from_sat(reveal_balance),
            reveal_fee: args.reveal_fee,
            commit_fee: args.commit_fee,
            leftover_amount: Amount::from_sat(leftover_amount),
        })
    }
}

#[derive(Debug)]
/// Arguments for creating a commit transaction
pub struct CreateCommitTransactionArgsV2<T>
    where
        T: Inscription,
{
    /// UTXOs to be used as inputs of the transaction
    pub inputs: Vec<Utxo>,
    /// Inscription to write
    pub inscription: T,
    /// Address to send the leftovers BTC of the trasnsaction
    pub leftovers_recipient: Address,
    /// Fee to pay for the commit transaction
    pub commit_fee: Amount,
    /// Fee to pay for the reveal transaction
    pub reveal_fee: Amount,
    /// Script pubkey of the inputs
    pub txin_script_pubkey: ScriptBuf,
}

/// Unspent transaction output to be used as input of a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    pub id: Txid,
    pub index: u32,
    pub amount: Amount,
}

#[derive(Debug, Clone)]
pub struct TxInputInfo {
    /// ID of the output.
    pub outpoint: OutPoint,
    /// Contents of the output.
    pub tx_out: TxOut,
    pub derivation_path: DerivationPath,
}