use crate::state::read_state;
use ic_solana::types::Pubkey;
use serde::{Deserialize, Deserializer};

pub fn port_address() -> (Pubkey, u8) {
    let program_id = read_state(|s| s.port_program_id);
    Pubkey::find_program_address(&[&b"port"[..]], &program_id)
}

pub fn vault_address() -> (Pubkey, u8) {
    let program_id = read_state(|s| s.port_program_id);
    let (port, _) = port_address();
    Pubkey::find_program_address(&[&b"vault"[..], port.as_ref()], &program_id)
}

#[derive(Deserialize)]
pub struct ParsedValue {
    pub accounts: Vec<String>,
    #[serde(deserialize_with = "decode_base64")]
    pub data: Vec<u8>,
}

fn decode_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    bs58::decode(&s)
        .into_vec()
        .map_err(serde::de::Error::custom)
}

pub mod instruction {
    use borsh::BorshSerialize;
    use borsh_derive::{BorshDeserialize, BorshSerialize};
    use sha2::{Digest, Sha256};

    #[derive(BorshSerialize)]
    pub struct Initialize {
        pub vault_bump: u8,
    }

    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct Transport {
        pub target_chain: String,
        pub recipient: String,
        pub amount: u64,
    }

    #[derive(BorshSerialize)]
    pub struct Redeem {
        pub ticket_id: String,
        pub amount: u64,
    }

    impl InstSerialize for Initialize {
        fn method() -> String {
            "initialize".into()
        }
    }

    impl InstSerialize for Transport {
        fn method() -> String {
            "transport".into()
        }
    }

    impl InstSerialize for Redeem {
        fn method() -> String {
            "redeem".into()
        }
    }

    pub trait InstSerialize: BorshSerialize {
        fn method() -> String;

        fn data(&self) -> Vec<u8> {
            let mut data = Vec::with_capacity(256);
            data.extend_from_slice(&Self::discriminator());
            self.serialize(&mut data).unwrap();
            data
        }

        fn discriminator() -> [u8; 8] {
            let mut hasher = Sha256::new();
            hasher.update(format!("global:{}", Self::method()));
            let result = hasher.finalize();
            let mut discriminator = [0u8; 8];
            discriminator.copy_from_slice(&result[..8]);
            discriminator
        }
    }
}
