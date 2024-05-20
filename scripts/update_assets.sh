# cargo clean
dfx build omnity_hub --check
dfx build bitcoin_customs --check
dfx build icp_route --check
candid-extractor target/wasm32-unknown-unknown/release/icp_route.wasm > route/icp/icp_route.did 
candid-extractor target/wasm32-unknown-unknown/release/omnity_hub.wasm > hub/omnity_hub.did 
candid-extractor target/wasm32-unknown-unknown/release/bitcoin_customs.wasm > customs/bitcoin/bitcoin_customs.did 
cp .dfx/local/canisters/omnity_hub/service.did.d.ts assets/omnity_hub/
cp .dfx/local/canisters/omnity_hub/service.did.js assets/omnity_hub/
cp .dfx/local/canisters/bitcoin_customs/service.did.d.ts assets/bitcoin_customs/
cp .dfx/local/canisters/bitcoin_customs/service.did.js assets/bitcoin_customs/
cp .dfx/local/canisters/icp_route/service.did.d.ts assets/icp_route/
cp .dfx/local/canisters/icp_route/service.did.js assets/icp_route/
