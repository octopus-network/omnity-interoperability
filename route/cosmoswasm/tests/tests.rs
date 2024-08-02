use candid::{Decode, Encode, Principal};
use cosmoswasm_route::{
    lifecycle::init::InitArgs,
    schnorr::{SchnorrKeyIds, SchnorrPublicKeyArgs, SchnorrPublicKeyResult},
};
use cosmrs::tendermint::serializers::bytes;
use ic_base_types::{CanisterId, PrincipalId};
use ic_state_machine_tests::{StateMachine, WasmResult};
use ic_test_utilities_load_wasm::load_wasm;
use omnity_types::Token;
use pocket_ic::PocketIc;
use serde_bytes::ByteBuf;

const SCHNORR_WASM: &[u8] = include_bytes!("../schnorr_canister.wasm");

// fn install
fn cw_wasm() -> Vec<u8> {
    load_wasm(
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        "cosmoswasm_route",
        &[],
    )
}

fn install_cw_route_canister(env: &StateMachine, schnorr: Principal) -> CanisterId {
    let payload = InitArgs {
        schnorr_canister_principal: schnorr,
        cosmoswasm_port_contract_address: "osmo1cyyzpxplxdzkeea7kwsydadg87357qnahakaks".to_string(),
        chain_id: "localosmosis".to_string(),
    };
    env.install_canister(cw_wasm(), Encode!(&payload).unwrap(), None)
        .unwrap()
}

fn install_schnorr_canister(env: &StateMachine) -> CanisterId {
    env.install_canister(SCHNORR_WASM.to_vec(), vec![], None)
        .unwrap()
}

struct Setup {
    pub env: StateMachine,
    pub schnorr: CanisterId,
    pub cw_route: CanisterId,
    pub caller: PrincipalId,
    // pub hub
}

fn canister_id_to_principal(canister_id: &CanisterId) -> Principal {
    Principal::from_slice(&canister_id.get().into_vec())
}

fn set_up() -> Setup {
    let env = StateMachine::new();
    let schnorr = install_schnorr_canister(&env);
    let cw_route = install_cw_route_canister(&env, canister_id_to_principal(&schnorr));
    let caller = PrincipalId::new_user_test_id(2);
    Setup {
        env,
        schnorr,
        cw_route,
        caller,
    }
}

fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => {
            dbg!(&bytes);
            bytes
        }
        WasmResult::Reject(reject) => {
            panic!("Expected a successful reply, got a reject: {}", reject)
        }
    }
}

impl Setup {
    pub fn add_token(&self) {
        let _ = Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.caller,
                        self.cw_route,
                        "test_add_token",
                        Encode!().unwrap()
                    )
                    .unwrap()
            ),
            ()
        );
    }
}

#[test]
pub fn test_add_token() {
    let setup = set_up();
    // let token = Token
}

#[test]
pub fn test_schnorr() {
    let setup = set_up();

    let derivation_path: Vec<ByteBuf> = [vec![1u8; 4]] // Example derivation path for signing
        .iter()
        .map(|v| ByteBuf::from(v.clone()))
        .collect();

    let key_id = SchnorrKeyIds::TestKey1.to_key_id();

    let payload = SchnorrPublicKeyArgs {
        // canister_id: Some(canister_id_to_principal(&setup.cw_route)),
        canister_id: None,
        derivation_path: derivation_path.clone(),
        key_id: key_id.clone(),
    };
    let r = Decode!(
        &assert_reply(
            setup.env
                .execute_ingress_as(
                    setup.caller,
                    setup.schnorr,
                    "schnorr_public_key",
                    Encode!(&payload).unwrap()
                )
                .unwrap()
        ),
        Result<SchnorrPublicKeyResult, String>
    )
    .unwrap();

    dbg!(&r);
}
