use candid::{decode_one, encode_one, types::principal, CandidType, Encode, Principal};
use cosmoswasm_route::{
    lifecycle::init::InitArgs,
    schnorr::{SchnorrKeyIds, SchnorrPublicKeyArgs, SchnorrPublicKeyResult},
    RouteError,
};
use ic_cdk::api::management_canister::http_request::HttpResponse;
// use ic_base_types::{CanisterId, PrincipalId};
// use ic_stable_structures::vec;
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};
use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::path::Path;

const SCHNORR_WASM: &[u8] = include_bytes!("../schnorr_canister.wasm");
const CW_ROUTE_WASM: &[u8] = include_bytes!("../cosmoswasm_route.wasm");

fn load_schnorr_canister_wasm() -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::prelude::*;

    // let wasm_path = Path::new("../schnorr_canister.wasm");
    dbg!(&SCHNORR_WASM.len());
    let wasm_bytes = SCHNORR_WASM.to_vec();
    // let wasm_bytes: Vec<u8> = vec![];

    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(wasm_bytes.as_slice()).unwrap();
    let zipped_bytes = e.finish().unwrap();

    zipped_bytes
}

fn load_cw_route_canister_wasm() -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::prelude::*;

    // let wasm_path = Path::new("../schnorr_canister.wasm");
    dbg!(&CW_ROUTE_WASM.len());
    let wasm_bytes = CW_ROUTE_WASM.to_vec();
    // let wasm_bytes: Vec<u8> = vec![];

    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(wasm_bytes.as_slice()).unwrap();
    let zipped_bytes = e.finish().unwrap();

    zipped_bytes
}

pub struct Setup {
    pub cosmoswasm_route: Principal,
    pub schnorr_canister: Principal,
    pub pic: PocketIc,
    pub caller: Principal,
}

// pub fn principal_to_canister_id(principal: Principal) -> CanisterId {
//     let principal_id: PrincipalId = principal.into();
//     CanisterId::unchecked_from_principal(principal_id)
// }

pub fn fast_forward(ic: &PocketIc, ticks: u64) {
    for _ in 0..ticks - 1 {
        ic.tick();
    }
}

pub fn update<T: CandidType + for<'de> Deserialize<'de>>(
    ic: &PocketIc,
    sender: Principal,
    receiver: Principal,
    method: &str,
    args: Vec<u8>,
) -> Result<T, String> {
    match ic.update_call(receiver, sender, method, args) {
        Ok(WasmResult::Reply(data)) => {
            dbg!(&data);
            Ok(decode_one(&data).unwrap())
        }
        Ok(WasmResult::Reject(error_message)) => Err(error_message.to_string()),
        Err(user_error) => Err(user_error.to_string()),
    }
}

const INIT_CYCLES: u128 = 2_000_000_000_000;

pub fn create_canister(ic: &PocketIc, env_key: &str) -> Principal {
    let canister_id = ic.create_canister();
    ic.add_cycles(canister_id, INIT_CYCLES);

    let wasm_path =
        std::env::var_os(env_key).unwrap_or_else(|| panic!("Missing `{}` env variable", env_key));

    // let wasm_module = std::fs::read(wasm_path).unwrap();
    let wasm_module = load_schnorr_canister_wasm();

    ic.install_canister(canister_id, wasm_module, vec![], None);
    canister_id
}

impl Setup {
    pub fn new() -> Self {
        let pic = PocketIcBuilder::new().with_application_subnet().build();

        let schnorr_canister_id = pic.create_canister();
        pic.add_cycles(schnorr_canister_id, 2_000_000_000_000);
        pic.install_canister(
            schnorr_canister_id.clone().into(),
            load_schnorr_canister_wasm(),
            vec![],
            None,
        );
        fast_forward(&pic, 5);

        dbg!("Schnorr canister installed");

        let cosmoswasm_route_principal = pic.create_canister();
        pic.add_cycles(cosmoswasm_route_principal, 2_000_000_000_000);
        let arg = InitArgs {
            schnorr_canister_principal: schnorr_canister_id,
            cosmoswasm_port_contract_address: "osmo1cyyzpxplxdzkeea7kwsydadg87357qnahakaks"
                .to_string(),
            chain_id: "localosmosis".to_string(),
            cw_rpc_url: "http://localhost:26657".to_string(),
        };
        pic.install_canister(
            cosmoswasm_route_principal.into(),
            load_cw_route_canister_wasm(),
            Encode!(&arg).unwrap(),
            None,
        );
        fast_forward(&pic, 5);

        dbg!("Cosmoswasm route canister installed");

        Self {
            cosmoswasm_route: cosmoswasm_route_principal,
            schnorr_canister: schnorr_canister_id.into(),
            pic,
            caller: Principal::anonymous(),
        }
    }

    fn schnorr_public_key(&self) {
        let derivation_path: Vec<ByteBuf> = [vec![1u8; 4]] // Example derivation path for signing
            .iter()
            .map(|v| ByteBuf::from(v.clone()))
            .collect();

        let public_arg = SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: derivation_path.clone(),
            key_id: SchnorrKeyIds::TestKey1.to_key_id(),
        };

        let res: Result<SchnorrPublicKeyResult, String> = update(
            &self.pic,
            self.caller,
            self.schnorr_canister,
            "schnorr_public_key",
            encode_one(public_arg).unwrap(),
        );

        dbg!(&res);
    }

    fn add_token(&self) {
        let res: Result<Result<HttpResponse, String>, String> = update(
            &self.pic,
            self.caller,
            self.cosmoswasm_route,
            "test_add_token",
            encode_one(()).unwrap(),
        );

        dbg!(&res);
    }

    fn cosmos_address(&self) {
        let res: Result<Result<String, String>, String> = update(
            &self.pic,
            self.caller,
            self.cosmoswasm_route,
            "cosmos_address",
            encode_one(()).unwrap(),
        );

        dbg!(&res);
    }
}

#[test]
fn test_schnorr_by_pocket_ic() {
    let setup = Setup::new();
    setup.schnorr_public_key();
}

#[test]
fn test_add_token() {
    let setup = Setup::new();
    setup.add_token();
}

#[test]
pub fn test_show_address() {
    let setup = Setup::new();
    let address = setup.cosmos_address();
    dbg!(&address);
}
