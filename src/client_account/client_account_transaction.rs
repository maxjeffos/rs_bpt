use std::convert::From;

use crate::serializable_form;
use crate::TransactionId;
use crate::TransactionType;

#[derive(Debug)]
pub struct ClientAccountTransaction {
    pub transaction_type: TransactionType,
    pub transaction_id: TransactionId,
    pub amount: Option<f64>,
}

impl From<serializable_form::Transaction> for ClientAccountTransaction {
    fn from(transaction: serializable_form::Transaction) -> Self {
        ClientAccountTransaction {
            transaction_type: transaction.transaction_type,
            transaction_id: transaction.transaction_id,
            amount: transaction.amount,
        }
    }
}

impl From<&serializable_form::Transaction> for ClientAccountTransaction {
    fn from(transaction: &serializable_form::Transaction) -> Self {
        ClientAccountTransaction {
            transaction_type: transaction.transaction_type,
            transaction_id: transaction.transaction_id,
            amount: transaction.amount,
        }
    }
}
