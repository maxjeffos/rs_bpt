use crate::TransactionId;

#[derive(Debug, PartialEq)]
pub enum TransactionProcessingError {
    ReferencedTransactionNotFound(TransactionId),
    TransactionAlreadyHasPendingDisupte(TransactionId),
    TransactionDoesNotHavePendingDisupte(TransactionId),
    TransactionIDAlreadyExists(TransactionId),
    AmountNotPresentForDeposit(TransactionId),
    AmountNotPresentForWithdrawal(TransactionId),
}

impl std::error::Error for TransactionProcessingError {}

impl std::fmt::Display for TransactionProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionProcessingError::ReferencedTransactionNotFound(t) => {
                write!(f, "ReferencedTransactionNotFound: {}", t)
            }
            TransactionProcessingError::TransactionAlreadyHasPendingDisupte(t) => {
                write!(f, "TransactionAlreadyHasPendingDisupte: {}", t)
            }
            TransactionProcessingError::TransactionDoesNotHavePendingDisupte(t) => {
                write!(f, "TransactionDoesNotHavePendingDisupte: {}", t)
            }
            TransactionProcessingError::TransactionIDAlreadyExists(t) => {
                write!(f, "TransactionIDAlreadyExists: {}", t)
            }
            TransactionProcessingError::AmountNotPresentForDeposit(t) => {
                write!(f, "AmountNotPresentForDeposit: {}", t)
            }
            TransactionProcessingError::AmountNotPresentForWithdrawal(t) => {
                write!(f, "AmountNotPresentForWithdrawal: {}", t)
            }
        }
    }
}
