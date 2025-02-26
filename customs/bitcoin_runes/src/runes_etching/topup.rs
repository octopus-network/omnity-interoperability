use candid::{CandidType, Principal};
use ic_cdk::{call};
use ic_ledger_types::{AccountIdentifier, BlockIndex, Memo, Subaccount, Tokens, TransferArgs};
use serde::{Deserialize, Serialize};


const MEMO_TOP_UP_CANISTER: Memo = Memo(0x50555054);
const LEDGER_CANISTER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const CMC: &str = "rkp4c-7iaaa-aaaaa-aaaca-cai";
const ICP_ROUTE_CANISTER_ID: &str = "7ywcn-nyaaa-aaaar-qaeza-cai";
const ICP_TRANSFER_FEE: u64 = 10000;

#[derive(CandidType, Serialize)]
pub struct NotifyTopUp {
    pub block_index: u64,
    pub canister_id: Principal
}
#[derive(Clone, Eq, PartialEq, Hash, Debug, CandidType, Deserialize, Serialize)]
pub struct Cycles(pub u128);

#[derive(Clone, Eq, PartialEq, Hash, Debug, CandidType, Deserialize, Serialize)]
pub enum NotifyError {
    Refunded {
        reason: String,
        block_index: Option<BlockIndex>,
    },
    InvalidTransaction(String),
    TransactionTooOld(BlockIndex),
    Processing,
    Other {
        error_code: u64,
        error_message: String,
    },
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct TopUpArgs {
    amount: Tokens, // icp的e8s
    canister_id: Principal // 要冲给哪个canister就写哪个canister
}

pub async fn topup(amt: u64,) -> Result<u128, String> {
    if amt <= ICP_TRANSFER_FEE {
        return Ok(0);
    }
    let icp_route_canister = Principal::from_text(ICP_ROUTE_CANISTER_ID).unwrap();
    let args = TopUpArgs {
        amount: Tokens::from_e8s(amt),
        canister_id: icp_route_canister,
    };
    // cycle minting canister id
    let cmc = Principal::from_text(CMC).unwrap();
    // icp ledger canister id
    let ledger = Principal::from_text(LEDGER_CANISTER).unwrap();
    let subaccount = Subaccount::from(args.canister_id);
    let to = AccountIdentifier::new(&cmc, &subaccount);

    // 1. transfer icp to cmc
    let transfer_args = ic_ledger_types::TransferArgs {
        memo: MEMO_TOP_UP_CANISTER,
        amount: args.amount,
        fee: Tokens::from_e8s(10_000),
        from_subaccount: None,
        to,
        created_at_time: None,
    };


    let block_idx = ic_ledger_types::transfer(ledger, transfer_args)
        .await
        .map_err(|e| format!("failed to call ledger: {:?}", e))?
        .map_err(|e| format!("ledger transfer error {:?}", e))?;
    // 2. notify cmc topup cycles
    let notify_arg = NotifyTopUp{
        block_index: block_idx,
        canister_id: args.canister_id
    };
    if let Ok((res, )) = call::<_, (Result<Cycles, NotifyError>, )>(cmc, "notify_top_up", (notify_arg, )).await{
        Ok(res.map_err(|e|format!("{:?}",e))?.0)
    }else {
        Err("send cycles error".to_string())
    }

}
