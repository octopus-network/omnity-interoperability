#deploy evm_rpc
dfx deploy evm_rpc --argument '(record { nodesInSubnet = 28 })'

#deploy cdk route
dfx deploy cdk_route --argument '(record { network = variant { local }; omnity_port_contract = "0x544F52f459a42E098775118e0A1880f1FA3eb9a9"; scan_start_height = 200000; evm_rpc_canister_addr = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai";  evm_chain_id = 686868; admin = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; hub_principal = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; chain_id = "cdk_merlin_test"; rpc_url = "https://testnet-rpc.merlinchain.io";})'


#[derive(CandidType, Deserialize)]
#pub struct InitArgs {
#    pub admin: Principal,
#    pub chain_id: String,
#    pub hub_principal: Principal,
#    pub evm_chain_id: u64,
#    pub evm_rpc_canister_addr: Principal,
#    pub omnity_port_contract: Vec<u8>,
#    pub scan_start_height: u64,
#    pub network: Network,
#}