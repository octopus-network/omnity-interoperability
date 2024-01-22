# NEAR Route

The NEAR Route is responsible for verifying transport transactions executed on NEAR Protocol. 

## Management

The NEAR Route is under management of the Hub which accepts 4 kinds of directives:

- Initialization: requires a NEAR block header to activate this component
- Pause: requires nothing
- Resume: requires nothing
- Register Token: TODO

## NEAR Light Client

The NEAR Route internally maintains a serious of block headers(at least 1 block header for each epoch) of NEAR Protocol. For each transport relay, a user must submit 3 parts:

- Header
- Proof of exists
- Origin transaction

For each transport request, the light client must find the nearest ancestor either from the internal storage or RPC. The nearest ancestor's epoch must satisfy `epoch(ancestor) == epoch(header)` or `epoch(ancestor) + 1 == epoch(header)`.

TODO
