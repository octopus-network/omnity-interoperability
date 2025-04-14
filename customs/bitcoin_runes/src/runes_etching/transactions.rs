use std::borrow::Cow;
use std::collections::BTreeMap;
use std::str::FromStr;

use anyhow::anyhow;
use bitcoin::{Address, Amount, PublicKey, Transaction, Txid};
use candid::{CandidType, Deserialize, Principal};
use ic_canister_log::log;
use ic_cdk::caller;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use omnity_types::ic_log::INFO;
use ordinals::{Etching, Rune, SpacedRune, Terms};
use serde::Serialize;

use omnity_types::call_error::{CallError, Reason};
use crate::runes_etching::constants::POSTAGE;
use crate::runes_etching::fee_calculator::{
    check_allowance, select_utxos, transfer_etching_fees, FIXED_COMMIT_TX_VBYTES, INPUT_SIZE_VBYTES,
};
use crate::runes_etching::fees::Fees;
use crate::runes_etching::icp_swap::estimate_etching_fee;
use crate::runes_etching::transactions::EtchingStatus::{SendCommitFailed, SendCommitSuccess};
use crate::runes_etching::wallet::builder::{EtchingKey, EtchingTransactionArgs};
use crate::runes_etching::wallet::{CreateCommitTransactionArgsV2, Runestone};
use crate::runes_etching::{
    EtchingArgs, InternalEtchingArgs, LogoParams, Nft, OrdResult, OrdTransactionBuilder,
    SignCommitTransactionArgs, Utxo,
};
use crate::runes_etching::topup::topup;
use crate::state::{mutate_state, read_state};
use crate::updates::etching::init_etching_account_info;
use crate::updates::get_btc_address::GetBtcAddressArgs;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SendEtchingRequest {
    pub etching_args: InternalEtchingArgs,
    pub txs: Vec<Transaction>,
    pub err_info: Option<CallError>,
    pub commit_at: u64,
    pub reveal_at: u64,
    pub script_out_address: String,
    pub status: EtchingStatus,
}

impl From<SendEtchingRequest> for SendEtchingInfo {
    fn from(value: SendEtchingRequest) -> Self {
        let err_info = match value.err_info {
            None => "".to_string(),
            Some(e) => e.to_string(),
        };
        SendEtchingInfo {
            etching_args: value.etching_args.clone().into(),
            err_info,
            commit_txid: value.txs[0].txid().to_string(),
            reveal_txid: value.txs[1].txid().to_string(),
            time_at: value.commit_at,
            script_out_address: value.script_out_address,
            status: value.status,
            receiver: value.etching_args.premine_receiver_principal,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, CandidType)]
pub struct SendEtchingInfo {
    pub etching_args: EtchingArgs,
    pub commit_txid: String,
    pub reveal_txid: String,
    pub err_info: String,
    pub time_at: u64,
    pub script_out_address: String,
    pub status: EtchingStatus,
    pub receiver: String,
}

impl Storable for SendEtchingRequest {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let dire = ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode Directive");
        dire
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, CandidType)]
pub enum EtchingStatus {
    Initial,
    SendCommitSuccess,
    SendCommitFailed,
    SendRevealSuccess,
    SendRevealFailed,
    TokenAdded,
    Final,
}

pub fn find_commit_remain_fee(t: &Transaction) -> Option<Utxo> {
    if t.output.len() > 1 {
        let r = t.output.last().cloned().unwrap();
        let utxo = Utxo {
            id: t.txid(),
            index: (t.output.len() - 1) as u32,
            amount: r.value,
        };
        Some(utxo)
    } else {
        None
    }
}

pub async fn etching_rune(
    fee_rate: u64,
    args: &InternalEtchingArgs,
) -> anyhow::Result<(SendEtchingRequest, u64)> {
    let (commit_tx_size, reveal_size) =
        estimate_tx_vbytes(args.rune_name.as_str(), args.logo.clone()).await?;
    let icp_fee_amt = estimate_etching_fee(fee_rate, (commit_tx_size + reveal_size) as u128)
        .await
        .map_err(|e| anyhow!(e))?;
    let allowance = if args.premine_receiver_principal.contains("-") {
        check_allowance(icp_fee_amt as u64).await?
    } else {
        0u64
    };
    let vins = select_utxos(fee_rate, reveal_size as u64 + FIXED_COMMIT_TX_VBYTES)?;
    log!(INFO, "selected fee utxos: {:?}", vins);
    let commit_size = vins.len() as u64 * INPUT_SIZE_VBYTES + FIXED_COMMIT_TX_VBYTES;
    let fee = Fees {
        commit_fee: Amount::from_sat(commit_size * fee_rate),
        reveal_fee: Amount::from_sat(reveal_size as u64 * fee_rate + POSTAGE * 2),
    };
    let result = generate_etching_transactions(fee, vins.clone(), args)
        .await
        .map_err(|e| {
            mutate_state(|s| {
                for in_utxo in vins.clone() {
                    s.etching_fee_utxos
                        .push(&in_utxo)
                        .expect("retire utxo failed");
                }
            });
            e
        })?;
    let mut send_res = SendEtchingRequest {
        etching_args: args.clone(),
        txs: result.txs.clone(),
        err_info: None,
        commit_at: ic_cdk::api::time(),
        reveal_at: 0,
        script_out_address: result.script_out_address.clone(),
        status: SendCommitSuccess,
    };
    if let Err(e) = crate::management::send_etching(&result.txs[0]).await {
        send_res.status = SendCommitFailed;
        send_res.err_info = Some(e);
    }
    //修改fee utxo列表
    if send_res.status == SendCommitSuccess {
        //insert_utxo
        if let Some(u) = find_commit_remain_fee(&send_res.txs.first().cloned().unwrap()) {
            let _ = mutate_state(|s| s.etching_fee_utxos.push(&u));
        }
    } else {
        mutate_state(|s| {
            for in_utxo in vins {
                s.etching_fee_utxos
                    .push(&in_utxo)
                    .expect("retire utxo failed1");
            }
        });
    }
    Ok((send_res, allowance))
}


pub async fn etching_rune_v2(
    fee_rate: u64,
    args: &InternalEtchingArgs,
) -> anyhow::Result<SendEtchingRequest> {
    let (_, reveal_size) =
        estimate_tx_vbytes(args.rune_name.as_str(), args.logo.clone()).await?;
    let vins = select_utxos(fee_rate, reveal_size as u64 + FIXED_COMMIT_TX_VBYTES)?;
    log!(INFO, "selected fee utxos: {:?}", vins);
    let commit_size = vins.len() as u64 * INPUT_SIZE_VBYTES + FIXED_COMMIT_TX_VBYTES;
    let fee = Fees {
        commit_fee: Amount::from_sat(commit_size * fee_rate),
        reveal_fee: Amount::from_sat(reveal_size as u64 * fee_rate + POSTAGE * 2),
    };
    let result = generate_etching_transactions(fee, vins.clone(), args)
        .await
        .map_err(|e| {
            mutate_state(|s| {
                for in_utxo in vins.clone() {
                    s.etching_fee_utxos
                        .push(&in_utxo)
                        .expect("retire utxo failed");
                }
            });
            e
        })?;
    let mut send_res = SendEtchingRequest {
        etching_args: args.clone(),
        txs: result.txs.clone(),
        err_info: None,
        commit_at: ic_cdk::api::time(),
        reveal_at: 0,
        script_out_address: result.script_out_address.clone(),
        status: SendCommitSuccess,
    };
    if let Err(e) = crate::management::send_etching(&result.txs[0]).await {
        send_res.status = SendCommitFailed;
        send_res.err_info = Some(e);
    }
    //修改fee utxo列表
    if send_res.status == SendCommitSuccess {
        //insert_utxo
        if let Some(u) = find_commit_remain_fee(&send_res.txs.first().cloned().unwrap()) {
            let _ = mutate_state(|s| s.etching_fee_utxos.push(&u));
        }
    } else {
        mutate_state(|s| {
            for in_utxo in vins {
                s.etching_fee_utxos
                    .push(&in_utxo)
                    .expect("retire utxo failed1");
            }
        });
    }
    Ok(send_res)
}

pub async fn generate_etching_transactions(
    fees: Fees,
    vins: Vec<Utxo>,
    args: &InternalEtchingArgs,
) -> anyhow::Result<BuildEtchingTxsResult> {
    let etching_account = init_etching_account_info().await;
    let sender = Address::from_str(etching_account.address.as_str())
        .unwrap()
        .assume_checked();

    let mut builder = OrdTransactionBuilder::p2tr(
        PublicKey::from_str(etching_account.pubkey.as_str()).unwrap(),
        sender.clone(),
    );
    let space_rune = SpacedRune::from_str(&args.rune_name).unwrap();
    let symbol = match args.symbol.clone() {
        None => None,
        Some(s) => {
            let cs: Vec<char> = s.chars().collect();
            cs.first().cloned()
        }
    };

    let terms = args.terms.map(|t| Terms {
                    amount: Some(t.amount),
                    cap: Some(t.cap),
                    height: t.height,
                    offset: t.offset,
                });
    let etching = Etching {
        rune: Some(space_rune.rune),
        divisibility: args.divisibility,
        premine: args.premine,
        spacers: Some(space_rune.spacers),
        symbol,
        terms,
        turbo: args.turbo,
    };

    let mut inscription = Nft::new(None, None, args.logo.clone());
    inscription.pointer = Some(vec![]);
    inscription.rune = Some(
        etching
            .rune
            .ok_or(anyhow::anyhow!("Invalid etching data; rune is missing"))?
            .commitment(),
    );
    let commit_tx = builder
        .build_commit_transaction_with_fixed_fees(CreateCommitTransactionArgsV2 {
            inputs: vins.clone(),
            inscription,
            txin_script_pubkey: sender.script_pubkey(),
            fees,
        })
        .await?;
    let signed_commit_tx = builder
        .sign_commit_transaction(
            commit_tx.unsigned_tx,
            SignCommitTransactionArgs {
                inputs: vins,
                txin_script_pubkey: sender.script_pubkey(),
            },
        )
        .await?;
    let pointer = if args.premine.is_some() {
        Some(1)
    } else {
        None
    };
    // make runestone
    let runestone = Runestone {
        etching: Some(etching),
        edicts: vec![],
        mint: None,
        pointer,
    };
    let  receipient = if let Ok(p) = Principal::from_text(args.premine_receiver_principal.as_str())  {
        let get_btc_address_args = GetBtcAddressArgs {
            target_chain_id: "eICP".to_string(),
            receiver: args.premine_receiver_principal.to_string(),
        };
        crate::updates::get_btc_address::get_btc_address(get_btc_address_args).await
    } else if let Ok(t) = Address::from_str(args.premine_receiver_principal.as_str()){
            args.premine_receiver_principal.clone()
    }else {
        return Err(anyhow!("unsupport receiver"));
    };
    let receipient = Address::from_str(receipient.as_str())
        .unwrap()
        .assume_checked();
    let reveal_transaction = builder
        .build_etching_transaction(EtchingTransactionArgs {
            input: Utxo {
                id: signed_commit_tx.txid(),
                index: 0,
                amount: commit_tx.reveal_balance,
            },
            recipient_address: receipient,
            redeem_script: commit_tx.redeem_script,
            runestone,
            derivation_path: None,
        })
        .await?;
    Ok(BuildEtchingTxsResult {
        txs: vec![signed_commit_tx, reveal_transaction],
        script_out_address: commit_tx.script_out_address,
    })
}

pub struct BuildEtchingTxsResult {
    pub txs: Vec<Transaction>,
    pub script_out_address: String,
}
pub async fn estimate_tx_vbytes(
    rune_name: &str,
    logo: Option<LogoParams>,
) -> OrdResult<(usize, usize)> {
    let fees = Fees {
        commit_fee: Amount::from_sat(1000),
        reveal_fee: Amount::from_sat(20000),
    };
    let sender = Address::from_str("bc1qyelgkxpfhfjrg6hg8hlr9t4dzn7n88eajxfy5c")
        .unwrap()
        .assume_checked();
    let vins = vec![Utxo {
        id: Txid::from_str("13a0ea6d76b710a1a9cdf2d8ce37c53feaaf985386f14ba3e65c544833c00a47")
            .unwrap(),
        index: 1,
        amount: Amount::from_sat(1122),
    }];
    let space_rune = SpacedRune::from_str(rune_name).unwrap();

    let etching = Etching {
        rune: Some(space_rune.rune),
        divisibility: Some(2),
        premine: Some(1000000),
        spacers: Some(space_rune.spacers),
        symbol: Some('$'),
        terms: Some(Terms {
            amount: Some(100000),
            cap: Some(10000),
            height: (None, None),
            offset: (None, None),
        }),
        turbo: true,
    };

    let mut inscription = Nft::new(
        Some("text/plain;charset=utf-8".as_bytes().to_vec()),
        Some(etching.rune.unwrap().to_string().as_bytes().to_vec()),
        logo,
    );
    inscription.pointer = Some(vec![]);
    inscription.rune = Some(
        etching
            .rune
            .ok_or(anyhow::anyhow!("Invalid etching data; rune is missing"))
            .unwrap()
            .commitment(),
    );
    let mut builder = OrdTransactionBuilder::p2tr(
        PublicKey::from_str("02eec672e95d002ac6d1e8ba97a2faa9d94c6162e2f20988984106ba6265020453")
            .unwrap(), //TODO
        sender.clone(),
    );
    let commit_tx = builder
        .estimate_commit_transaction(CreateCommitTransactionArgsV2 {
            inputs: vins.clone(),
            inscription,
            txin_script_pubkey: sender.script_pubkey(),
            fees,
        })
        .await?;
    let runestone = Runestone {
        etching: Some(etching),
        edicts: vec![],
        mint: None,
        pointer: Some(1),
    };
    let reveal_transaction = builder
        .build_etching_transaction(EtchingTransactionArgs {
            input: Utxo {
                id: commit_tx.unsigned_tx.txid(),
                index: 0,
                amount: commit_tx.reveal_balance,
            },
            recipient_address: sender,
            redeem_script: commit_tx.redeem_script,
            runestone,
            derivation_path: None,
        })
        .await?;
    Ok((commit_tx.unsigned_tx.vsize(), reveal_transaction.vsize()))
}


pub async fn stash_etching(fee_rate: u64, args: EtchingArgs) -> Result<String, String> {
    let space_rune = SpacedRune::from_str(args.rune_name.as_str()).map_err(|e| e.to_string())?;
    check_name_duplication(space_rune.rune)?;
    let caller = caller();
    args.check().map_err(|e| e.to_string())?;
    let (commit_tx_size, reveal_size) =
        estimate_tx_vbytes(args.rune_name.as_str(), args.logo.clone()).await.map_err(|e|e.to_string())?;
   let icp_fee_amt = estimate_etching_fee(fee_rate, (commit_tx_size + reveal_size) as u128)
        .await
        .map_err(|e| e.to_string())?;
    let allowance = check_allowance(icp_fee_amt as u64).await.map_err(|e|e.to_string())?;
    let _ = transfer_etching_fees(allowance as u128).await.map_err(|e|e.to_string())?;
    let r = topup(allowance).await;
    log!(INFO, "etching topup result:{:?}", r);
    let internal_args: InternalEtchingArgs = (args.clone(), caller, None).into();
    let etching_key = format!("Bitcoin-runes-{}", args.rune_name);
    mutate_state(|s|{
        s.stash_etchings.insert(etching_key.clone(), internal_args);
         let _ = s.stash_etching_ids.push(&EtchingKey::new(etching_key.clone()));
    });
    Ok(etching_key)
}

pub async fn internal_etching(fee_rate: u64, args: EtchingArgs, premine_address: Option<String>) -> Result<String, String> {
    let space_rune = SpacedRune::from_str(args.rune_name.as_str()).map_err(|e| e.to_string())?;
    check_name_duplication(space_rune.rune)?;
    let caller = caller();
    args.check().map_err(|e| e.to_string())?;
    let internal_args: InternalEtchingArgs = (args, caller, premine_address).into();
    let r = etching_rune(fee_rate, &internal_args).await;
    match r {
        Ok((sr, allowance)) => {
            if sr.status == SendCommitSuccess {
                let commit_tx_id = sr.txs[0].txid().to_string();
                mutate_state(|s| s.pending_etching_requests.insert(commit_tx_id.clone(), sr));
                if allowance > 0 {
                    let r = transfer_etching_fees(allowance as u128).await;
                    log!(INFO, "transfer etching fee result: {:?}", r);
                    let r = topup(allowance).await;
                    log!(INFO, "etching topup result {:?}", r);
                }
                Ok(commit_tx_id)
            } else {
                Err(sr
                    .err_info
                    .unwrap_or(CallError {
                        method: "send commit tx".to_string(),
                        reason: Reason::QueueIsFull,
                    })
                    .to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn check_name_duplication(rune: Rune) -> Result<(), String> {
    let mut kvs = read_state(|s| {
        s.pending_etching_requests
            .iter()
            .collect::<BTreeMap<String, SendEtchingRequest>>()
    });
    let mut kvs1 = read_state(|s| {
        s.finalized_etching_requests
            .iter()
            .collect::<BTreeMap<String, SendEtchingRequest>>()
    });
    kvs.append(&mut kvs1);
    for (_k, v) in kvs {
        let space_rune = SpacedRune::from_str(v.etching_args.rune_name.as_str()).unwrap();
        if space_rune.rune == rune {
            return Err("the rune name is already etching".to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::runes_etching::transactions::estimate_tx_vbytes;
    use crate::runes_etching::LogoParams;

    #[tokio::test]
    pub async fn test() {
        let rune_name = "WEFW•WEGEF•SDOSS";
        let logo = LogoParams {
            content_type: "image/png;base64".to_string(),
            content_base64:  r#"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"#.to_string(),
        };
        let r = estimate_tx_vbytes(rune_name, None).await.unwrap();
        println!("{} {}", r.0, r.1);
        let r = estimate_tx_vbytes(rune_name, Some(logo)).await.unwrap();
        println!("{} {}", r.0, r.1);
    }
}
