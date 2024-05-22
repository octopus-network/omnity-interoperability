dfx stop
dfx start --clean --background
#deploy evm_rpc
dfx deploy evm_rpc --argument '(record { nodesInSubnet = 28 })'

#deploy cdk route
dfx deploy evm_route --argument '(record { fee_token_id = "BTC" network = variant { local }; omnity_port_contract = "0x765F2c1F334E6479Be5D5F8f2E12128612f47CE3"; scan_start_height = 200000; evm_rpc_canister_addr = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai";  evm_chain_id = 11155111; admin = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; hub_principal = principal "be2us-64aaa-aaaaa-qaabq-cai"; chain_id = "cdk_sepolia"; rpc_url = "https://rpc-sepolia.rockx.com";})'
