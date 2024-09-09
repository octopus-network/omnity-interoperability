#!/usr/bin/env bash


# disable warning
export DFX_WARNING="-mainnet_plaintext_identity"

total_calls=100
success_count=0
failure_count=0
output_file="block_hash_summary_report.md"
# Canister ID
SOLANA_ROUTE_CANISTER_ID=lvinw-hiaaa-aaaar-qahoa-cai

log=get_latest_blockhash.log
echo "" > $log
echo "test get_latest_blockhash ...."
for i in $(seq 1 $total_calls); do
  echo "Executing get_latest_blockhash request $i..."
  response=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID get_latest_blockhash '()' --ic --candid ./assets/solana_route.did)
  echo "The No.$i response:$response" >> $log
  if [[ $response == *"Ok"* ]]; then
    ((success_count++))
  else
    ((failure_count++))
  fi
done

success_rate=$(echo "scale=2; $success_count/$total_calls*100" | bc)
failure_rate=$(echo "scale=2; $failure_count/$total_calls*100" | bc)

echo "# Block Hash Summary Report" > $output_file
echo "" >> $output_file
echo "## get_latest_blockhash" >> $output_file
echo "- ***Total Calls**: $total_calls" >> $output_file
echo "- ***Successful Calls**: $success_count" >> $output_file
echo "- ***Failed Calls**: $failure_count" >> $output_file
echo "- ***Success Rate**: $success_rate%" >> $output_file
echo "- ***Failure Rate**: $failure_rate%" >> $output_file

echo "Report generated: $output_file"
