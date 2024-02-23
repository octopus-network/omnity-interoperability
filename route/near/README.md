# NEAR Route

The NEAR Route is responsible for verifying transport transactions executed on NEAR Protocol. 

## Management

The NEAR Route is under management of the Hub which accepts 4 kinds of directives:

- Initialization: requires a NEAR block header and an RPC url to activate this component
- Pause: requires nothing
- Resume: requires a new RPC url
- Register Token: TODO

## NEAR Light Client

The NEAR Route internally maintains a serious of block headers(at least 1 block header for each epoch) of NEAR Protocol. For each transport request, it must satisfies 3 conditions:

1. Pass the light block header validation
2. Pass the transaction validation 
3. The transaction is not yet accepted by NEAR route

### Light Block Header Validation

To validate a header, the NEAR light client must know either an ancestor header of this epoch or an arbitrary header of previous epoch. The NEAR route will periodically fetch a new block header from RPC and try to accept it if `epoch(new_header) = epoch(highest_header) + 1`. That will ensure the light client keeps 1 block header at every epoch.
If a user submit a transport request to the NEAR route at epoch `X` while the light client still in `X-1`, the light client will try to validate the user-requested header and accept it.
If a user submit a transport request with a header in epoch `X` and the light client got `X-2` at most, e.g. some IO errors occured during the timer logic, the route will reject the request. Thus, an off-chain watch dog should monitor the NEAR protocol and NEAR route and fire an alarm if the NEAR route falls behind 2 or more epochs.

### RPC Cycles

```
cycles = (3_000_000 + 60_000 * n) * n + 400 * n * req_bytes + 800 * n + rsp_bytes

```
for a 12-nodes subnet, a single RPC request might cost around 0.16B cycles(0.02 cents).
