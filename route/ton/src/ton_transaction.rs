use std::str::FromStr;

use anyhow::anyhow;
use base64::Engine;
use num_bigint::BigUint;
use tonlib_core::cell::{BagOfCells, CellBuilder};
use tonlib_core::message::JETTON_INTERNAL_TRANSFER;
use tonlib_core::mnemonic::KeyPair;
use tonlib_core::wallet::{TonWallet, WalletVersion};
use tonlib_core::{TonAddress, TonAddressParseError};

use omnity_types::{Seq, Ticket};

use crate::chainkey::sign_external_body;
use crate::state::public_key;
use crate::ton_common::transfer::TransferMessage;

pub async fn build_jetton_mint(
    jetton_master: &str,
    ticket: &Ticket,
    seq: Seq,
    wallet_seqno: i32,
) -> anyhow::Result<String> {
    let pubkey = public_key();
    let fake_keypair = KeyPair {
        public_key: pubkey,
        secret_key: vec![],
    };
    let wallet = TonWallet::derive_default(WalletVersion::V4R2, &fake_keypair)
        .map_err(|e| anyhow!(e.to_string()))?;
    let self_addr: TonAddress = wallet.address.clone();
    let destination: TonAddress = ticket
        .receiver
        .as_str()
        .parse()
        .map_err(|e: TonAddressParseError| anyhow!(e.to_string()))?;
    let mut cb = CellBuilder::new();
    cb.store_u32(32, 21).map_err(|e| anyhow!(e.to_string()))?;
    cb.store_u64(64, seq).map_err(|e| anyhow!(e.to_string()))?;
    cb.store_address(&destination)
        .map_err(|e| anyhow!(e.to_string()))?;
    cb.store_coins(&BigUint::from(100000000u64))
        .map_err(|e| anyhow!(e.to_string()))?;
    let mut transfer_body = CellBuilder::new();
    transfer_body
        .store_u32(32, JETTON_INTERNAL_TRANSFER)
        .map_err(|e| anyhow!(e.to_string()))?;
    transfer_body
        .store_u64(64, seq)
        .map_err(|e| anyhow!(e.to_string()))?;
    transfer_body
        .store_coins(&BigUint::from(
            u128::from_str(&ticket.amount).map_err(|e| anyhow!(e.to_string()))?,
        ))
        .map_err(|e| anyhow!(e.to_string()))?;
    transfer_body
        .store_address(&self_addr)
        .map_err(|e| anyhow!(e.to_string()))?;
    let jetton_master_addr: TonAddress = jetton_master
        .parse()
        .map_err(|e: TonAddressParseError| anyhow!(e.to_string()))?;
    transfer_body
        .store_address(&self_addr)
        .map_err(|e| anyhow!(e.to_string()))?;
    transfer_body
        .store_coins(&BigUint::from(1000000u32))
        .map_err(|e| anyhow!(e.to_string()))?;
    transfer_body.store_bit(true).unwrap();
    let pc = CellBuilder::new()
        .store_string("")
        .unwrap()
        .build()
        .unwrap();
    transfer_body.store_reference(&pc.to_arc()).unwrap();
    cb.store_reference(&transfer_body.build().unwrap().to_arc())?;
    let ton_amount = BigUint::from(100000000u64);
    let transfer = TransferMessage::new(&jetton_master_addr, &ton_amount)
        .with_data(cb.build().unwrap())
        .build()?;
    let now = (ic_cdk::api::time() / 1000000000) as u32;
    let body = wallet
        .create_external_body(
            now + 10000,
            wallet_seqno.try_into().unwrap(),
            vec![transfer.to_arc()],
        )
        .map_err(|e| anyhow!(e.to_string()))?;
    let signed_msg = sign_external_body(&body).await?;
    let sig = wallet
        .wrap_signed_body(signed_msg, false)
        .map_err(|e| anyhow!(e.to_string()))?;
    let boc = BagOfCells::from_root(sig)
        .serialize(true)
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(boc))
}
