#!/bin/bash

# Initialize the cumulative balance variable.
CUMULATIVE_STORAGE_EXPENSE=0
ID=vp-test
# get init balance
INIT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
echo "omnity_hub current initial balance: $INIT_BALANCE cycles"
# get previous balance
PREV_BALANCE=$INIT_BALANCE
# Run the loop 12 times.
for i in {1..12}
do
  # Fetch the current balance.
  CURRENT_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
  echo "omnity_hub current balance: $CURRENT_BALANCE cycles"
   # expense
  EXPENSE=$(bc <<< "$PREV_BALANCE - $CURRENT_BALANCE")
  echo "Expense for this iteration: $EXPENSE cycles"
  # cumulative EXPENSE
  CUMULATIVE_STORAGE_EXPENSE=$(bc <<< "$CUMULATIVE_STORAGE_EXPENSE + $EXPENSE")
  # update previous balance
  PREV_BALANCE=$CURRENT_BALANCE
  # Wait for 5 seconds before the next iteration.
  sleep 5
done
# Calculate the average expense balance.
AVERAGE_EXPENSE=$(bc <<< "scale=2; $CUMULATIVE_STORAGE_EXPENSE / 12")
# Printing the cumulative and average balances.
echo "Total expense after 1 minute: $CUMULATIVE_STORAGE_EXPENSE cycles"
echo "Average expense: $AVERAGE_EXPENSE cycles"

CUMULATIVE_CALL_EXPENSE=0
for i in {1..12}
do
  # Fetch the current balance.
  BEFORE_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
  echo "omnity_hub before query_directives current balance: $BEFORE_BALANCE cycles"
  dfx canister call omnity_hub query_directives '(opt "eICP",null,0:nat64,12:nat64)' --ic --identity $ID
  # Fetch the current balance.
  AFTER_BALANCE="$(dfx canister status omnity_hub --ic --identity $ID 2>&1 | grep -i balance | cut -d' ' -f 2 | sed 's/_//g')"
  echo "omnity_hub after query_directives current balance: $AFTER_BALANCE cycles"
   # expense
  EXPENSE=$(bc <<< "$BEFORE_BALANCE - $AFTER_BALANCE - $AVERAGE_EXPENSE")
  echo "Expense for this query_directives: $EXPENSE cycles"
  CUMULATIVE_CALL_EXPENSE=$(bc <<< "$CUMULATIVE_CALL_EXPENSE + $EXPENSE")

  sleep 5
done

# Calculate the average expense balance.
AVERAGE_EXPENSE=$(bc <<< "scale=2; $CUMULATIVE_CALL_EXPENSE / 12")
# Printing the cumulative and average balances.
echo "Total call expense after 1 minute: $CUMULATIVE_CALL_EXPENSE cycles"
echo "Average call expense: $AVERAGE_EXPENSE cycles"
