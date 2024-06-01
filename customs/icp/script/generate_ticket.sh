# Refer to this document to deploy ledger canister.
# https://internetcomputer.org/docs/current/developer-docs/defi/icrc-1/icrc1-ledger-setup
$ dfx deploy icrc1_ledger_canister --argument "(variant {Init =
record {
     token_symbol = \"<0001f9e7>\";
     token_name = \"TEST•RICH\";
     minting_account = record { owner = principal \"onpkv-r64im-go7um-v7jnc-i33lg-ubem4-sxbyk-megsv-ww7fz-yzee7-zae\" };
     transfer_fee = 0;
     decimals = opt 2;
     metadata = vec {
        record {
      \"icrc1:logo\";
      variant {
        Text = \"https://github.com/ordinals/ord/assets/14307069/f1307be5-84fb-4b58-81d0-6521196a2406\"
      };
    }
     };
     feature_flags = opt record{icrc2 = true};
     initial_balances = vec {};
     archive_options = record {
         num_blocks_to_archive = 1000;
         trigger_threshold = 1000;
         controller_id = principal \"onpkv-r64im-go7um-v7jnc-i33lg-ubem4-sxbyk-megsv-ww7fz-yzee7-zae\";
         cycles_for_archive_creation = null;
     };
 }
})"
$ icrc1_ledger_canister: http://127.0.0.1:4943/?canisterId=b77ix-eeaaa-aaaaa-qaada-cai&id=bw4dl-smaaa-aaaaa-qaacq-cai

# Use mint account to call ledger, mint token to the receiving account.
dfx canister call icrc1_ledger_canister icrc1_transfer 'record {
    from_subaccount = null;
    to = record {owner = principal "cu4zh-2c4it-54irp-xgtxc-gajvr-h6gle-c5n7r-hwpeg-spkye-z4ta7-iae"; subaccount = null};
    amount = 1000000000;
    fee = null;
    memo = null;
    created_at_time = null
}'

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc1_balance_of "(record {owner = principal \"cu4zh-2c4it-54irp-xgtxc-gajvr-h6gle-c5n7r-hwpeg-spkye-z4ta7-iae\"; })"

$ dfx deploy omnity_hub --argument '(variant { Init = record { admin = principal "onpkv-r64im-go7um-v7jnc-i33lg-ubem4-sxbyk-megsv-ww7fz-yzee7-zae"} })'
$ dfx deploy icp_route --argument '(variant { Init = record { chain_state = variant { Active }; hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; chain_id = "eICP" } })'
$ dfx deploy icp_customs --argument '(variant { Init = record { hub_principal = principal "bkyz2-fmaaa-aaaaa-qaaaq-cai"; chain_id = "ICP" } })'

icp_customs: http://0.0.0.0:4943/?canisterId=bd3sg-teaaa-aaaaa-qaaba-cai&id=br5f7-7uaaa-aaaaa-qaaca-cai
icp_route: http://0.0.0.0:4943/?canisterId=bd3sg-teaaa-aaaaa-qaaba-cai&id=be2us-64aaa-aaaaa-qaabq-cai
omnity_hub: http://0.0.0.0:4943/?canisterId=bd3sg-teaaa-aaaaa-qaaba-cai&id=bkyz2-fmaaa-aaaaa-qaaaq-cai

$ dfx canister call omnity_hub sub_directives '(opt "ICP", vec {variant {AddChain};variant {AddToken};variant {UpdateFee};variant {ToggleChainState}})'
$ dfx canister call omnity_hub sub_directives '(opt "eICP", vec {variant {AddChain};variant {AddToken};variant {UpdateFee};variant {ToggleChainState}})'

$ dfx canister call omnity_hub execute_proposal '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "ICP"; chain_type=variant { SettlementChain };canister_id="br5f7-7uaaa-aaaaa-qaaca-cai"; contract_address=null;counterparties=opt vec {"eICP"}; fee_token=null}}})'
$ dfx canister call omnity_hub execute_proposal  '(vec {variant { AddChain = record { chain_state=variant { Active };chain_id = "eICP"; chain_type=variant { ExecutionChain };canister_id="be2us-64aaa-aaaaa-qaabq-cai";  contract_address=null; counterparties= opt vec {"ICP"}; fee_token=opt "LICP"}}})'
$ dfx canister call omnity_hub execute_proposal '( vec {variant { AddToken = record { decimals = 2 : nat8; icon = opt "rune.logo.url"; token_id = "ICP-icrc-TEST•RICH"; name = "TEST•RICH";issue_chain = "ICP"; symbol = "TEST•RICH"; metadata =  vec{ record {"ledger_id"; "bw4dl-smaaa-aaaaa-qaacq-cai"}}; dst_chains = vec {"ICP";"eICP";}}}})'
$ dfx canister call omnity_hub update_fee 'vec {variant { UpdateTargetChainFactor = record {target_chain_id="ICP"; target_chain_factor=0 : nat}}; variant { UpdateFeeTokenFactor = record { fee_token="LICP"; fee_token_factor=1 : nat}}}'

# Use `dfx identity use xxx` switch to the receiving account.

$ dfx canister call bw4dl-smaaa-aaaaa-qaacq-cai icrc2_approve "(record { amount = 2000000; spender = record{owner = principal \"br5f7-7uaaa-aaaaa-qaaca-cai\";} })"
$ dfx canister call icp_customs generate_ticket '(record {target_chain_id = "eICP"; receiver = "cu4zh-2c4it-54irp-xgtxc-gajvr-h6gle-c5n7r-hwpeg-spkye-z4ta7-iae"; token_id = "ICP-icrc-TEST•RICH"; amount = 2000000})'
