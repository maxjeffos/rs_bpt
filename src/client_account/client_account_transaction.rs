use std::convert::From;

use crate::ser_form;
use crate::TransactionId;
use crate::TransactionType;

pub struct ClientAccountTransaction {
    pub transaction_type: TransactionType,
    pub transaction_id: TransactionId,
    pub amount: Option<f64>, // TODO: make this a decimal
}

impl From<ser_form::Transaction> for ClientAccountTransaction {
    fn from(transaction: ser_form::Transaction) -> Self {
        ClientAccountTransaction {
            transaction_type: transaction.transaction_type,
            transaction_id: transaction.transaction_id,
            amount: transaction.amount,
        }
    }
}
