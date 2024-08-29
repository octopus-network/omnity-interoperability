#!/usr/bin/env bash


# disable warning
export DFX_WARNING="-mainnet_plaintext_identity"

total_calls=5
success_count=0
failure_count=0
output_file="uc_summary_report.md"
echo "# Feature Summary Report" > $output_file
echo "" >> $output_file
# Canister ID
SOLANA_ROUTE_CANISTER_ID=lvinw-hiaaa-aaaar-qahoa-cai

TOKEN_ID_PRE="Bitcoin-runes-HOPE•YOU•GET•FUNNY"
TOKEN_NAME_PRE="HOPE•YOU•GET•FUNNY"
TOKEN_SYMBOL_PRE="FUNNY"
DECIMALS=2
ICON="https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json"

echo "test create_mint_account ...."
log=create_mint_account.log
echo "" > $log
for i in $(seq 1 $total_calls); do
  echo "Executing create_mint_account request $i..."
  TOKEN_ID=${TOKEN_ID_PRE}$i
  TOKEN_NAME=${TOKEN_NAME_PRE}$i
  TOKEN_SYMBOL=${TOKEN_SYMBOL_PRE}$i
  echo ${TOKEN_ID}
  echo ${TOKEN_NAME}
  echo ${TOKEN_SYMBOL}

  # Capture start time
  start_time=$(date +%s)
  response=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID create_mint_account "(record {
        token_id=\"${TOKEN_ID}\";
        name=\"${TOKEN_NAME}\";
        symbol=\"${TOKEN_SYMBOL}\";
        decimals=${DECIMALS}:nat8;
        uri=\"${ICON}\";
        })" \
    --ic \
    --candid ./assets/solana_route.did)
  # Capture end time
  end_time=$(date +%s)
  # Calculate duration in seconds
  duration=$(( end_time - start_time ))
  echo "The No.$i execution time: ${duration} seconds" >> $log
  echo "The No.$i response:$response" >> $log
  if [[ $response == *"Ok"* ]]; then
    ((success_count++)) 
  else
    ((failure_count++))
  fi
done

success_rate=$(echo "scale=2; $success_count/$total_calls*100" | bc)
failure_rate=$(echo "scale=2; $failure_count/$total_calls*100" | bc)

echo "## create_mint_account" >> $output_file
echo "- ***Total Calls**: $total_calls" >> $output_file
echo "- ***Successful Calls**: $success_count" >> $output_file
echo "- ***Failed Calls**: $failure_count" >> $output_file
echo "- ***Success Rate**: $success_rate%" >> $output_file
echo "- ***Failure Rate**: $failure_rate%" >> $output_file


echo "test create_aossicated_account ...."
log=create_aossicated_account.log
echo "" > $log
success_count=0
failure_count=0
SOL_RECEIVER="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
for i in $(seq 1 $total_calls); do
  echo "Executing create_aossicated_account request $i..."
  TOKEN_ID=${TOKEN_ID_PRE}$i
  TOKEN_MINT=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_address "(\"${TOKEN_ID}\")" --ic)
  TOKEN_MINT=$(echo "$TOKEN_MINT" | awk -F'"' '{print $2}')
  echo "token mint address: $TOKEN_MINT"
  echo "solana wallet address: $SOL_RECEIVER"

  if [[ -n "$TOKEN_MINT" ]]; then
    start_time=$(date +%s)
    response=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID create_aossicated_account "(\"${SOL_RECEIVER}\",
          \"${TOKEN_MINT}\")" \
        --ic \
        --candid ./assets/solana_route.did)
    end_time=$(date +%s)
    duration=$(( end_time - start_time ))
    echo "The No.$i execution time: ${duration} seconds" >> $log
    echo "The No.$i response:$response" >> $log
    if [[ $response == *"Ok"* ]]; then
      ((success_count++))
      ATA=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account_address "(\"${SOL_RECEIVER}\",
          \"${TOKEN_MINT}\")" \
        --ic \
        --candid ./assets/solana_route.did)
      ATA=$(echo "$ATA" | awk -F'"' '{print $2}')
      echo "aossicated account address: $ATA"
    else
      ((failure_count++))
    fi
  else
    ((success_count++))
    echo "Not found the $TOKEN_ID mint account" >> $log
  fi

done

success_rate=$(echo "scale=2; $success_count/$total_calls*100" | bc)
failure_rate=$(echo "scale=2; $failure_count/$total_calls*100" | bc)

echo "## create_aossicated_account" >> $output_file
echo "- ***Total Calls**: $total_calls" >> $output_file
echo "- ***Successful Calls**: $success_count" >> $output_file
echo "- ***Failed Calls**: $failure_count" >> $output_file
echo "- ***Success Rate**: $success_rate%" >> $output_file
echo "- ***Failure Rate**: $failure_rate%" >> $output_file

echo "test mint_token ...."
log=mint_token.log
echo "" > $log
success_count=0
failure_count=0
SOL_RECEIVER="3gghk7mHWtFsJcg6EZGK7sbHj3qW6ExUdZLs9q8GRjia"
TID_PRE=28b47548-55dc-4e89-b41d-76bc0247828fa
MINT_AMOUNT=222222
for i in $(seq 1 $total_calls); do
  echo "Executing mint_token request $i..."
  TOKEN_ID=${TOKEN_ID_PRE}$i
  TID=${TID_PRE}$i
  TOKEN_MINT=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_mint_address "(\"${TOKEN_ID}\")" --ic)
  TOKEN_MINT=$(echo "$TOKEN_MINT" | awk -F'"' '{print $2}')
  echo "token mint address: $TOKEN_MINT"
  echo "solana wallet address: $SOL_RECEIVER"

  if [[ -n "$TOKEN_MINT" ]]; then
    ATA=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID query_aossicated_account_address "(\"${SOL_RECEIVER}\",
          \"${TOKEN_MINT}\")" \
        --ic \
        --candid ./assets/solana_route.did)
    ATA=$(echo "$ATA" | awk -F'"' '{print $2}')
    echo "aossicated account address: $ATA"

    if [[ -n "$ATA" ]]; then
      start_time=$(date +%s)
      response=$(dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token "(record{
        ticket_id=\"${TID}\";
        associated_account=\"${ATA}\";
        amount=${MINT_AMOUNT}:nat64;
        token_mint=\"${TOKEN_MINT}\";
        status=variant { Unknown };
        signature=null;})" --ic)
      end_time=$(date +%s)
      duration=$(( end_time - start_time ))
      echo "The No.$i execution time: ${duration} seconds" >> $log
      echo "The No.$i response:$response" >> $log

      if [[ $response == *"Ok"* ]]; then
         ((success_count++))
         dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_req "(\"${TID}\")" --ic
         dfx canister call $SOLANA_ROUTE_CANISTER_ID mint_token_status "(\"${TID}\")" --ic
      else
         ((failure_count++))
      fi
    else
      ((success_count++))
      echo "Not found the $SOL_RECEIVER ata account" >> $log
    fi
  else
    ((success_count++))
    echo "Not found the $TOKEN_ID mint account" >> $log
  fi

done

success_rate=$(echo "scale=2; $success_count/$total_calls*100" | bc)
failure_rate=$(echo "scale=2; $failure_count/$total_calls*100" | bc)

echo "## mint_token" >> $output_file
echo "- ***Total Calls**: $total_calls" >> $output_file
echo "- ***Successful Calls**: $success_count" >> $output_file
echo "- ***Failed Calls**: $failure_count" >> $output_file
echo "- ***Success Rate**: $success_rate%" >> $output_file
echo "- ***Failure Rate**: $failure_rate%" >> $output_file

echo "Report generated: $output_file"
