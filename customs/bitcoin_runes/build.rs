fn main() {
    let did_path = std::path::PathBuf::from("bitcoin_customs.did")
        .canonicalize()
        .unwrap();

    println!(
        "cargo:rustc-env=BITCOIN_CUSTOMS_DID_PATH={}",
        did_path.display()
    );
}
