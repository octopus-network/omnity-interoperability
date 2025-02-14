use crate::ic_sui::constants::*;
use ic_cdk::api::management_canister::http_request::CanisterHttpRequestArgument;

/// Calculates the baseline cost of sending a JSON-RPC request using HTTP outcalls.
pub fn http_request_required_cycles(
    arg: &CanisterHttpRequestArgument,
    nodes_in_subnet: u32,
) -> u128 {
    let max_response_bytes = match arg.max_response_bytes {
        Some(ref n) => *n as u128,
        None => 2 * 1024 * 1024, // default 2MiB
    };
    let nodes_in_subnet = nodes_in_subnet as u128;

    // The coefficients can be found in [this page](https://internetcomputer.org/docs/current/developer-docs/production/computation-and-storage-costs).
    // 12 is "http_request".len().

    let request_bytes = candid::utils::encode_args((arg,))
        .expect("Failed to encode arguments.")
        .len() as u128
        + 12;
    // header_len = header_1.name + header_1.value + ... + header_n.name + header_n.value
    // request_size = url.len + transform.name.len + transform.context.len + body.len + header_len
    // http_outcall_cost = per_call_cost + per_request_byte_cost * request_size + per_response_byte_cost * max_response_size
    // scaling_factor = subnet_size / 13
    // total_cost = scaling_factor * http_outcall_cost

    (HTTP_OUTCALL_REQUEST_BASE_COST
        + HTTP_OUTCALL_REQUEST_PER_NODE_COST * nodes_in_subnet
        + HTTP_OUTCALL_REQUEST_COST_PER_BYTE * request_bytes
        + HTTP_OUTCALL_RESPONSE_COST_PER_BYTE * max_response_bytes)
        * nodes_in_subnet

    // let payload_size_bytes = arg.body.as_ref().map_or(0, |v| v.len()) as u128;
    // let ingress_bytes = payload_size_bytes as u128
    //     + u32::max(RPC_URL_MIN_COST_BYTES, arg.url.len() as u32) as u128
    //     + INGRESS_OVERHEAD_BYTES;
    // let cost_per_node = INGRESS_MESSAGE_RECEIVED_COST
    //     + INGRESS_MESSAGE_BYTE_RECEIVED_COST * ingress_bytes
    //     + HTTP_OUTCALL_REQUEST_BASE_COST
    //     + HTTP_OUTCALL_REQUEST_PER_NODE_COST * nodes_in_subnet as u128
    //     + HTTP_OUTCALL_REQUEST_COST_PER_BYTE * payload_size_bytes as u128
    //     + HTTP_OUTCALL_RESPONSE_COST_PER_BYTE * max_response_bytes as u128
    //     + CANISTER_OVERHEAD;
    // cost_per_node * (nodes_in_subnet as u128)
}

pub fn get_http_request_cost(payload_size_bytes: u64, max_response_bytes: u64) -> u128 {
    let nodes_in_subnet = NODES_IN_SUBNET as u128;
    let ingress_bytes =
        payload_size_bytes as u128 + RPC_URL_COST_BYTES as u128 + INGRESS_OVERHEAD_BYTES;
    let cost_per_node = INGRESS_MESSAGE_RECEIVED_COST
        + INGRESS_MESSAGE_BYTE_RECEIVED_COST * ingress_bytes
        + HTTP_OUTCALL_REQUEST_BASE_COST
        + HTTP_OUTCALL_REQUEST_PER_NODE_COST * nodes_in_subnet
        + HTTP_OUTCALL_REQUEST_COST_PER_BYTE * payload_size_bytes as u128
        + HTTP_OUTCALL_RESPONSE_COST_PER_BYTE * max_response_bytes as u128
        + CANISTER_OVERHEAD;
    cost_per_node * nodes_in_subnet
}
