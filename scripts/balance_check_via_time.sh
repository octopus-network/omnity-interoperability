#!/bin/bash

set -euo pipefail
trap "echo 'error: Script failed: see failed command above'" ERR

export DFX_WARNING="-mainnet_plaintext_identity"
ID=vp-test
TIME=600 
con_record="consumption.txt"
# hub_id=r6kfs-wqaaa-aaaak-akviq-cai

echo "deploy bitcoin_mock and icp_mock to query omnity_hub from stable momery " >> $con_record
dfx deploy bitcoin_mock --mode reinstall -y --argument '(record {hub_principal = principal "r6kfs-wqaaa-aaaak-akviq-cai"; directive_method="query_directives"; ticket_method="query_tickets"; network=null})' --ic --identity $ID -q
dfx deploy icp_mock --mode reinstall -y --argument '(record {hub_principal = principal "r6kfs-wqaaa-aaaak-akviq-cai"; directive_method="query_directives"; ticket_method="query_tickets"})' --ic --identity $ID -q
PRE_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') omnity_hub current balance: $PRE_BALANCE cycles" >> $con_record
sleep $TIME
CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') after $TIME seconds,omnity_hub current balance: $CURRENT_BALANCE cycles" >> $con_record
CONSUMPTION=$(bc <<< "$PRE_BALANCE - $CURRENT_BALANCE")
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') the consumption that query omnity_hub from stable momery is: $CONSUMPTION cycles" >> $con_record

echo "deploy bitcoin_mock and icp_mock to query omnity_hub from momery " >> $con_record
dfx deploy bitcoin_mock --mode reinstall -y --argument '(record {hub_principal = principal "r6kfs-wqaaa-aaaak-akviq-cai"; directive_method="query_directives_from_map"; ticket_method="query_tickets_from_map"; network=null})' --ic --identity $ID -q
dfx deploy icp_mock --mode reinstall -y --argument '(record {hub_principal = principal "r6kfs-wqaaa-aaaak-akviq-cai"; directive_method="query_directives_from_map"; ticket_method="query_tickets_from_map"})' --ic --identity $ID -q
PRE_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') omnity_hub current balance: $PRE_BALANCE cycles" >> $con_record
sleep $TIME
CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') after $TIME seconds,omnity_hub current balance: $CURRENT_BALANCE cycles" >> $con_record
CONSUMPTION=$(bc <<< "$PRE_BALANCE - $CURRENT_BALANCE")
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') the consumption that query omnity_hub from momery is: $CONSUMPTION cycles" >> $con_record


echo "deploy bitcoin_mock and icp_mock to query omnity_hub from mix momery " >> $con_record
dfx deploy bitcoin_mock --mode reinstall -y --argument '(record {hub_principal = principal "r6kfs-wqaaa-aaaak-akviq-cai"; directive_method="query_directives_from_mix"; ticket_method="query_tickets_from_mix"; network=null})' --ic --identity $ID -q 
dfx deploy icp_mock --mode reinstall -y --argument '(record {hub_principal = principal "r6kfs-wqaaa-aaaak-akviq-cai"; directive_method="query_directives_from_mix"; ticket_method="query_tickets_from_mix"})' --ic --identity $ID -q
PRE_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') omnity_hub current balance: $PRE_BALANCE cycles" >> $con_record
sleep $TIME
CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') after $TIME seconds,omnity_hub current balance: $CURRENT_BALANCE cycles" >> $con_record
CONSUMPTION=$(bc <<< "$PRE_BALANCE - $CURRENT_BALANCE")
echo "$(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S') the consumption that query omnity_hub from mix momery is: $CONSUMPTION cycles" >> $con_record

