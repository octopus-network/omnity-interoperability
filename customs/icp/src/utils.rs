use std::str::FromStr;

pub fn convert_u128_u64(n: u128) -> u64 {
    if n > u64::MAX as u128 {
        panic!("u128 value is too large to convert to u64");
    }
    n as u64
}

pub fn nat_to_u64(nat: candid::Nat) -> u64 {
    u64::from_str(&nat.0.to_string()).unwrap()
}