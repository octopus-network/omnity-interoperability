use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::call_error::{CallError, Reason};
use crate::state::read_state;

#[derive(Serialize, Deserialize, CandidType, Default, Clone)]
pub struct TokenPrice {
    pub name: String,
    #[serde(rename = "priceUSD")]
    pub price_usd: f64,
    pub standard: String,
    pub symbol: String,
}

pub async fn estimate_etching_fee(_fee_rate: u64, _etching_size: u128) -> Result<u128, String> {
    /*let satoshi = fee_rate as u128 * etching_size;
    let (btc, icp) = get_token_price().await.map_err(|e| e.to_string())?;
    if btc.is_none() || icp.is_none() {
        return Err("estimate etching fees failed, please try agin later, code: 0".to_string());
    }
    let btcprice = Decimal::from_f64(btc.unwrap().price_usd).unwrap();
    let icpprice = Decimal::from_f64(icp.unwrap().price_usd).unwrap();
    if icpprice <= Decimal::ZERO || btcprice <= Decimal::ZERO {
        return Err("estimate etching fees failed, please try agin later, code: 1".to_string());
    }
    let fee_usd = Decimal::from(satoshi).mul(btcprice);
    let icp_amt = fee_usd
        .div(icpprice)
        .to_u128()
        .ok_or("estimate etching fees failed, please try agin later, code:3".to_string())?;
    Ok(icp_amt)*/
    Ok(100000000)
}

pub async fn get_token_price() -> Result<(Option<TokenPrice>, Option<TokenPrice>), CallError> {
    let method = "getAllTokens";
    let ord_principal = read_state(|s| s.icpswap_principal.clone().unwrap());
    let resp: (Vec<TokenPrice>,) = ic_cdk::api::call::call(ord_principal, method, ())
        .await
        .map_err(|(code, message)| CallError {
            method: method.to_string(),
            reason: Reason::from_reject(code, message),
        })?;
    let mut btc_token_price = None;
    let mut icp_token_price = None;
    for t in resp.0 {
        if t.name == "ckBTC" {
            btc_token_price = Some(t.clone());
        }
        if t.name == "ICP" {
            icp_token_price = Some(t.clone())
        }
    }

    Ok((btc_token_price, icp_token_price))
}
