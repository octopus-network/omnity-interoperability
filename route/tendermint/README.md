# Tendermint Route Canister


Build & Deploy & Activate
``` 4d
dfx build
dfx deploy tendermint_lightclient_canister --argument '("otto",1000,60,60,"https://gateway.mainnet.octopus.network/rpc/otto/andk2nmw198f7on2")'
dfx canister call tendermint_lightclient_canister activate
```

