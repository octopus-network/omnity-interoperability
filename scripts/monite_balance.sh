#!/bin/bash

export DFX_WARNING="-mainnet_plaintext_identity"
# Initialize the cumulative balance variable.
ID=vp-test
SIZE=17280
# get init balance
# TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S'
PREV_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') omnity_hub current balance: $PREV_BALANCE cycles"


# call query_directives for SIZE
echo "begin to call query_directives for $SIZE times"

for i in $(seq 1 $SIZE)
do
  dfx canister call omnity_hub query_directives '(opt "eICP",null,12:nat64,24:nat64)' --ic --identity $ID > /dev/null
  sleep 5
done

echo "end to call query_directives"

CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') omnity_hub current balance: $CURRENT_BALANCE cycles"

# Calculate the average expense balance.
EXPENSE=$(bc <<< "$PREV_BALANCE - $CURRENT_BALANCE")
# Printing the cumulative and average balances.
# 1 TC= 10^12 cycles
TC=1000000000000
echo "Total calling expense include storage for $SIZE times: $EXPENSE cycles,$(printf "%.6f" $(echo "scale=6;$EXPENSE/$TC"|bc)) TC"

