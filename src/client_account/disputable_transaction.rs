use crate::TransactionId;

// Encodes a deposit as a positive amount and a withdrawal as a negative amount.
#[derive(Debug)]
pub struct DisputableTransaction {
    pub transaction_id: TransactionId,
    pub amount: f64,
    pub is_under_dispute: bool,
}

impl DisputableTransaction {
    pub fn new_deposit_transaction(transaction_id: TransactionId, amount: f64) -> Self {
        Self {
            transaction_id,
            amount,
            is_under_dispute: false,
        }
    }

    pub fn new_withdrawal_transaction(transaction_id: TransactionId, amount: f64) -> Self {
        Self {
            transaction_id,
            amount: -amount,
            is_under_dispute: false,
        }
    }
}
