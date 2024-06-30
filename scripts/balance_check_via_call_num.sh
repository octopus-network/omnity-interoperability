#!/bin/bash

export DFX_WARNING="-mainnet_plaintext_identity"
# Initialize the cumulative balance variable.
CUMULATIVE_STORAGE_EXPENSE=0
ID=vp-test
# get init balance
INIT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "omnity_hub current initial balance: $INIT_BALANCE cycles"
# get previous balance
PREV_BALANCE=$INIT_BALANCE
sleep 5
# Run the loop 12 times.
for i in {1..12}
do
  # Fetch the current balance.
  CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
  echo "omnity_hub current balance: $CURRENT_BALANCE cycles"
   # expense
  STORAGE_EXPENSE=$(bc <<< "$PREV_BALANCE - $CURRENT_BALANCE")
  echo "Storage expense for this iteration: $STORAGE_EXPENSE cycles"
  # cumulative EXPENSE
  CUMULATIVE_STORAGE_EXPENSE=$(bc <<< "$CUMULATIVE_STORAGE_EXPENSE + $STORAGE_EXPENSE")
  # update previous balance
  PREV_BALANCE=$CURRENT_BALANCE
  # Wait for 5 seconds before the next iteration.
  sleep 5
done
# Calculate the average expense balance.
AVERAGE_STORAGE_EXPENSE=$(bc <<< "scale=2; $CUMULATIVE_STORAGE_EXPENSE / 12")
# Printing the cumulative and average balances.
echo "Total storage expense after 1 minute: $CUMULATIVE_STORAGE_EXPENSE cycles"
echo "Average storgae expense: $AVERAGE_STORAGE_EXPENSE cycles"

CUMULATIVE_CALL_EXPENSE=0
PREV_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "omnity_hub before query_directives current balance: $PREV_BALANCE cycles"
sleep 5
for i in {1..12}
do
  # Fetch the current balance,before calling
  # CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
  # echo "omnity_hub current balance: $CURRENT_BALANCE cycles"
  # call query_directives
  # dfx canister call omnity_hub query_directives '(opt "eICP",null,12:nat64,24:nat64)' --ic --identity $ID
  dfx canister call omnity_hub query_directives_from_map '(opt "eICP",null,0:nat64,12:nat64)' --ic --identity $ID
  # Fetch the current balance,after calling
  CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
  echo "omnity_hub after query_directives current balance: $CURRENT_BALANCE cycles"
   # expense
  CALL_EXPENSE=$(bc <<< "$PREV_BALANCE - $CURRENT_BALANCE - $AVERAGE_STORAGE_EXPENSE")
  echo "Calling expense for this query_directives: $CALL_EXPENSE cycles"
  CUMULATIVE_CALL_EXPENSE=$(bc <<< "$CUMULATIVE_CALL_EXPENSE + $CALL_EXPENSE")
  # update previous balance
  PREV_BALANCE=$CURRENT_BALANCE
  # Wait for 5 seconds before the next iteration.
  sleep 5
done

# Calculate the average expense balance.
AVERAGE_CALL_EXPENSE=$(bc <<< "scale=2; $CUMULATIVE_CALL_EXPENSE / 12")
# Printing the cumulative and average balances.
echo "Total calling expense after 1 minute: $CUMULATIVE_CALL_EXPENSE cycles"
echo "Average calling expense: $AVERAGE_CALL_EXPENSE cycles"


CUMULATIVE_CALL_EXPENSE=0
PREV_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "omnity_hub before query_tickets current balance: $PREV_BALANCE cycles"
sleep 5
for i in {1..12}
do
  # Fetch the current balance,before calling
  # CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
  # echo "omnity_hub current balance: $CURRENT_BALANCE cycles"
  # call query_directives
  # dfx canister call omnity_hub query_tickets '(opt "Bitcoin",6:nat64,12:nat64)' --ic --identity $ID
    dfx canister call omnity_hub query_tickets_from_map '(opt "Bitcoin",0:nat64,6:nat64)' --ic --identity $ID
  # Fetch the current balance,after calling
  CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
  echo "omnity_hub after query_tickets current balance: $CURRENT_BALANCE cycles"
   # expense
  CALL_EXPENSE=$(bc <<< "$PREV_BALANCE - $CURRENT_BALANCE - $AVERAGE_STORAGE_EXPENSE")
  echo "Calling expense for this query_tickets: $CALL_EXPENSE cycles"
  CUMULATIVE_CALL_EXPENSE=$(bc <<< "$CUMULATIVE_CALL_EXPENSE + $CALL_EXPENSE")
  # update previous balance
  PREV_BALANCE=$CURRENT_BALANCE
  # Wait for 5 seconds before the next iteration.
  sleep 5
done

# Calculate the average expense balance.
AVERAGE_CALL_EXPENSE=$(bc <<< "scale=2; $CUMULATIVE_CALL_EXPENSE / 12")
# Printing the cumulative and average balances.
echo "Total calling expense after 1 minute: $CUMULATIVE_CALL_EXPENSE cycles"
echo "Average calling expense: $AVERAGE_CALL_EXPENSE cycles"
