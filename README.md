# Omnity

## Draft design
The `BroadingPass`(or `Cargo` for alternatives) issued by `x_route`:

``` rust
pub struct BroadingPass {
    pub id: H256, // id of this pass, generated by the some messages below using deterministic hash
    pub from: Chain,
    pub to: Chain,
    pub initiator: Account,
    pub receiver: Account,
    pub face_value: u128,
    pub token: Token,
    pub universal_time: u64,
}
```

``` rust
enum Account {
    {name}(BYTES),
    // e.g.
    Ed25519(Bytes32),
    H160(Bytes20),
}
```

``` rust
enum Chain {
    NEAR,
    COSMOS(String),
    BTC,
    EVM(String),
}
```

``` rust
enum Token {

}
```