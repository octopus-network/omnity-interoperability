use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ExecuteTransactionRequestType {
    WaitForEffectsCert,
    WaitForLocalExecution,
}

#[derive(Debug)]
pub enum TransactionType {
    SingleWriter, // Txes that only use owned objects and/or immutable objects
    SharedObject, // Txes that use at least one shared object
}
