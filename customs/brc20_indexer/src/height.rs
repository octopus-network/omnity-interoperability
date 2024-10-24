use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpMethod, TransformContext, TransformFunc,
};
use std::str::FromStr;

pub async fn get_block_height() -> u64 {
    let url = "https://mempool.space/api/blocks/tip/height";

    const MAX_CYCLES: u128 = 200_000_000;
    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(1000),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        }),
        headers: vec![],
    };

    match http_request(request, MAX_CYCLES).await {
        Ok((response,)) => {
            let status = response.status;
            if status == 200_u32 {
                u64::from_str(
                    String::from_utf8(response.body)
                        .unwrap_or_default()
                        .as_str(),
                )
                .unwrap_or_default()
            } else {
                0
            }
        }
        Err((_, _m)) => 0,
    }
}
