#!/bin/bash

set -euo pipefail
trap "echo 'error: Script failed: see failed command above'" ERR

export DFX_WARNING="-mainnet_plaintext_identity"
ID=vp-test
TIME=3600
TC=1000000000000
log_file="balance.log"
# hub_id=r6kfs-wqaaa-aaaak-akviq-cai
SIZE=1

for i in $(seq 1 $SIZE)
do
    PRE_BALANCE="$(dfx canister --ic call e3mmv-5qaaa-aaaah-aadma-cai canister_status '(record {canister_id = (principal "7wupf-wiaaa-aaaar-qaeya-cai");})' --ic --identity $ID 2>&1 | grep -i cycles | cut -d'=' -f 2 | cut -d' ' -f 2 | sed 's/_//g')"
    echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') omnity_hub current balance: $PRE_BALANCE cycles" >> $log_file
    # echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') omnity_hub current balance: $PRE_BALANCE cycles" 
    sleep $TIME
    CURRENT_BALANCE="$(dfx canister --ic call e3mmv-5qaaa-aaaah-aadma-cai canister_status '(record {canister_id = (principal "7wupf-wiaaa-aaaar-qaeya-cai");})' --ic --identity $ID 2>&1 | grep -i cycles | cut -d'=' -f 2 | cut -d' ' -f 2 | sed 's/_//g')"
    echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') after $TIME seconds,omnity_hub current balance: $CURRENT_BALANCE cycles" >> $log_file
    # echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') after $TIME seconds,omnity_hub current balance: $CURRENT_BALANCE cycles" 

    CONSUMPTION=$(bc <<< "$PRE_BALANCE - $CURRENT_BALANCE")
    echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') the omnity_hub consumption is: $CONSUMPTION cycles, $(printf "%.6f" $(echo "scale=6;$CONSUMPTION/$TC"|bc)) TC" >> $log_file
    # echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') the omnity_hub consumption is: $CONSUMPTION cycles, $(printf "%.6f" $(echo "scale=6;$CONSUMPTION/$TC"|bc)) TC"

done