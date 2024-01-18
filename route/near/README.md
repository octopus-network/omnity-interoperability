# NEAR Route

The NEAR Route is responsible for verifying transport transactions executed on NEAR Protocol. 

## Management

The NEAR Route is under management of the Hub which accepts 4 kinds of directives:

- Initialization: requires a NEAR block header to activate this component
- Pause: requires nothing
- Resume: requires nothing
- Register Token: TODO

## NEAR Light Client

The NEAR Route internally maintains a serious of block headers of NEAR Protocol. For each transport relay, a user must submit 3 parts:

- Header
- Proofs of state(fetched from height-1)
- Token burnt

After the light client validates the request, it accepts the header and updates the height to `H` if the header > max(accepted_headers).

** Question: What if two users submit 2 transport transactions on NEAR at `H` and `H+1` respectively, then the light client handle the `H+1` first.

** Assume the light client already maintains a header `X` where `X` < `H` < `H+1`. After the newly `H+1` handled, the highest height becomes `H+1` but the light client still has `X`, so we could validate the `H` in terms of `X`.
