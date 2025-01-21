use ic_crypto_sha2::Sha256;
use ic_ic00_types::DerivationPath;
use serde_bytes::ByteBuf;

use crate::address::BitcoinAddress;
use crate::management;
use crate::state::{mutate_state, read_state, EtchingAccountInfo};

pub async fn init_etching_account_info() -> EtchingAccountInfo {
    let account_info = read_state(|s| s.etching_acount_info.clone());
    if account_info.is_inited() {
        return account_info;
    }
    let btc_network = read_state(|s| s.btc_network.clone());
    let key_name = read_state(|s| s.ecdsa_key_name.clone());
    let derive_path_str = "etching_address";
    let dp = DerivationPath::new(vec![ByteBuf::from(derive_path_str.as_bytes())]);
    let pub_key = management::ecdsa_public_key(key_name, dp.clone())
        .await
        .unwrap_or_else(|e| ic_cdk::trap(&format!("failed to retrieve ECDSA public key: {e}")));

    use ripemd::{Digest, Ripemd160};
    let address =
        BitcoinAddress::P2wpkhV0(Ripemd160::digest(Sha256::hash(&pub_key.public_key)).into());
    let deposit_addr = address.display(btc_network);
    let account_info = EtchingAccountInfo {
        pubkey: hex::encode(pub_key.public_key),
        address: deposit_addr,
        derive_path: derive_path_str.to_string(),
    };
    mutate_state(|s| {
        s.etching_acount_info = account_info.clone();
    });
    account_info
}
