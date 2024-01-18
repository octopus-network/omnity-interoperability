use std::{cell::RefCell, collections::HashMap};

// TODO
thread_local! {
    static TOKENS: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(Default::default());
    static RECEIPTS: RefCell<HashMap<[u8; 32], Vec<u8>>> = RefCell::new(Default::default());
}
