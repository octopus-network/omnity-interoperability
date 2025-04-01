pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

pub use icp::post;

mod icp {
    use ic_cdk::api::management_canister::http_request::{
        http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
        HttpResponse as CanisterHttpResponse, TransformContext, TransformFunc,
    };

    use crate::lightclient::rpc_types::errors::HttpError;

    use super::HttpResponse;

    pub async fn post(
        url: &str,
        body: Vec<u8>,
        max_respone_size: u64,
        max_cost_cycles: u128,
    ) -> Result<HttpResponse, HttpError> {
        let headers = vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }];
        let req = CanisterHttpRequestArgument {
            url: url.to_string(),
            method: HttpMethod::POST,
            headers,
            body: Some(body),
            transform: Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform".to_string(),
                }),
                context: vec![],
            }),
            max_response_bytes: Some(max_respone_size),
        };
        let resp = http_request(req, max_cost_cycles)
            .await
            .map_err(|(_code, msg)| HttpError::Other(msg))?;
        resp.0.try_into()
    }

    impl TryFrom<CanisterHttpResponse> for HttpResponse {
        type Error = HttpError;

        fn try_from(value: CanisterHttpResponse) -> Result<Self, Self::Error> {
            Ok(Self {
                status: http_status_from_nat(&value.status)?,
                body: value.body,
            })
        }
    }

    fn http_status_from_nat(status: &candid::Nat) -> Result<u16, HttpError> {
        (&status.0).try_into().map_err(|err| {
            HttpError::Other(format!("Status should be a 3 digit number, got: {err}"))
        })
    }
}
