dfx canister create schnorr_canister
dfx canister install --wasm schnorr_canister.wasm schnorr_canister
dfx canister create cosmoswasm_route
dfx canister install --wasm cosmoswasm_route.wasm cosmoswasm_route --argument '( record { schnorr_canister_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; cosmoswasm_port_contract_address = "osmo1ywdhdslsvnr7udqr50up3qtg94lslcnrn99unxt986jdh9ws4raskd9ewp"; chain_id = "osmo-test-5"; cw_rpc_url= "https://rpc.testnet.osmosis.zone:443"; cw_rest_url= "https://lcd.testnet.osmosis.zone" } )'
# dfx canister call cosmoswasm_route test_add_token '()'