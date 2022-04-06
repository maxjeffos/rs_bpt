use crate::TransactionId;

#[derive(Debug, PartialEq)]
pub enum DisputeRelatedTransactionType {
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
pub struct DisputeRelatedTransaction {
    pub referenced_transaction_id: TransactionId,
    pub dispute_related_transaction_type: DisputeRelatedTransactionType,
}

impl DisputeRelatedTransaction {
    pub fn new_dispute_transaction(referenced_transaction_id: TransactionId) -> Self {
        Self {
            referenced_transaction_id,
            dispute_related_transaction_type: DisputeRelatedTransactionType::Dispute,
        }
    }

    pub fn new_resolve_transaction(referenced_transaction_id: TransactionId) -> Self {
        Self {
            referenced_transaction_id,
            dispute_related_transaction_type: DisputeRelatedTransactionType::Resolve,
        }
    }

    pub fn new_chargeback_transaction(referenced_transaction_id: TransactionId) -> Self {
        Self {
            referenced_transaction_id,
            dispute_related_transaction_type: DisputeRelatedTransactionType::Chargeback,
        }
    }
}
