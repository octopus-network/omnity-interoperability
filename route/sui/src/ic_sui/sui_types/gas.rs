pub use checked::*;

pub mod checked {
    use crate::ic_sui::sui_types::{
        effects::{TransactionEffects, TransactionEffectsAPI},
        error::{UserInputError, UserInputResult},
        // gas_model::{gas_v2::SuiGasStatus as SuiGasStatusV2, tables::GasStatus},
        object::Object,
        sui_serde::{BigInt, Readable},
        // transaction::ObjectReadResult,
        // ObjectID,
    };
    // use enum_dispatch::enum_dispatch;
    use itertools::MultiUnzip;
    // use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;
    // use sui_protocol_config::ProtocolConfig;

    #[serde_as]
    #[derive(Eq, PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GasCostSummary {
        /// Cost of computation/execution
        // #[schemars(with = "BigInt<u64>")]
        #[serde_as(as = "Readable<BigInt<u64>, _>")]
        pub computation_cost: u64,
        /// Storage cost, it's the sum of all storage cost for all objects created or mutated.
        // #[schemars(with = "BigInt<u64>")]
        #[serde_as(as = "Readable<BigInt<u64>, _>")]
        pub storage_cost: u64,
        /// The amount of storage cost refunded to the user for all objects deleted or mutated in the
        /// transaction.
        // #[schemars(with = "BigInt<u64>")]
        #[serde_as(as = "Readable<BigInt<u64>, _>")]
        pub storage_rebate: u64,
        /// The fee for the rebate. The portion of the storage rebate kept by the system.
        // #[schemars(with = "BigInt<u64>")]
        #[serde_as(as = "Readable<BigInt<u64>, _>")]
        pub non_refundable_storage_fee: u64,
    }

    impl GasCostSummary {
        pub fn new(
            computation_cost: u64,
            storage_cost: u64,
            storage_rebate: u64,
            non_refundable_storage_fee: u64,
        ) -> GasCostSummary {
            GasCostSummary {
                computation_cost,
                storage_cost,
                storage_rebate,
                non_refundable_storage_fee,
            }
        }

        pub fn gas_used(&self) -> u64 {
            self.computation_cost + self.storage_cost
        }

        /// Portion of the storage rebate that gets passed on to the transaction sender. The remainder
        /// will be burned, then re-minted + added to the storage fund at the next epoch change
        pub fn sender_rebate(&self, storage_rebate_rate: u64) -> u64 {
            // we round storage rebate such that `>= x.5` goes to x+1 (rounds up) and
            // `< x.5` goes to x (truncates). We replicate `f32/64::round()`
            const BASIS_POINTS: u128 = 10000;
            (((self.storage_rebate as u128 * storage_rebate_rate as u128)
            + (BASIS_POINTS / 2)) // integer rounding adds half of the BASIS_POINTS (denominator)
            / BASIS_POINTS) as u64
        }

        /// Get net gas usage, positive number means used gas; negative number means refund.
        pub fn net_gas_usage(&self) -> i64 {
            self.gas_used() as i64 - self.storage_rebate as i64
        }

        pub fn new_from_txn_effects<'a>(
            transactions: impl Iterator<Item = &'a TransactionEffects>,
        ) -> GasCostSummary {
            let (storage_costs, computation_costs, storage_rebates, non_refundable_storage_fee): (
                Vec<u64>,
                Vec<u64>,
                Vec<u64>,
                Vec<u64>,
            ) = transactions
                .map(|e| {
                    (
                        e.gas_cost_summary().storage_cost,
                        e.gas_cost_summary().computation_cost,
                        e.gas_cost_summary().storage_rebate,
                        e.gas_cost_summary().non_refundable_storage_fee,
                    )
                })
                .multiunzip();

            GasCostSummary {
                storage_cost: storage_costs.iter().sum(),
                computation_cost: computation_costs.iter().sum(),
                storage_rebate: storage_rebates.iter().sum(),
                non_refundable_storage_fee: non_refundable_storage_fee.iter().sum(),
            }
        }
    }

    impl std::fmt::Display for GasCostSummary {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "computation_cost: {}, storage_cost: {},  storage_rebate: {}, non_refundable_storage_fee: {}",
                self.computation_cost, self.storage_cost, self.storage_rebate, self.non_refundable_storage_fee,
            )
        }
    }

    impl std::ops::AddAssign<&Self> for GasCostSummary {
        fn add_assign(&mut self, other: &Self) {
            self.computation_cost += other.computation_cost;
            self.storage_cost += other.storage_cost;
            self.storage_rebate += other.storage_rebate;
            self.non_refundable_storage_fee += other.non_refundable_storage_fee;
        }
    }

    impl std::ops::AddAssign<Self> for GasCostSummary {
        fn add_assign(&mut self, other: Self) {
            self.add_assign(&other)
        }
    }

    //
    // Helper functions to deal with gas coins operations.
    //

    pub fn deduct_gas(gas_object: &mut Object, charge_or_rebate: i64) {
        // The object must be a gas coin as we have checked in transaction handle phase.
        let gas_coin = gas_object.data.try_as_move_mut().unwrap();
        let balance = gas_coin.get_coin_value_unsafe();
        let new_balance = if charge_or_rebate < 0 {
            balance + (-charge_or_rebate as u64)
        } else {
            assert!(balance >= charge_or_rebate as u64);
            balance - charge_or_rebate as u64
        };
        gas_coin.set_coin_value_unsafe(new_balance)
    }

    pub fn get_gas_balance(gas_object: &Object) -> UserInputResult<u64> {
        if let Some(move_obj) = gas_object.data.try_as_move() {
            if !move_obj.type_().is_gas_coin() {
                return Err(UserInputError::InvalidGasObject {
                    object_id: gas_object.id(),
                });
            }
            Ok(move_obj.get_coin_value_unsafe())
        } else {
            Err(UserInputError::InvalidGasObject {
                object_id: gas_object.id(),
            })
        }
    }
}
