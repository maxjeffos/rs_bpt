#[derive(Debug, PartialEq)]
pub enum TransactionProcessingError {
    ReferencedTransactionNotFound,
    TransactionAlreadyHasPendingDisupte,
    TransactionDoesNotHavePendingDisupte,
    TransactionIDAlreadyExists,
}

impl std::error::Error for TransactionProcessingError {}

impl std::fmt::Display for TransactionProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionProcessingError::ReferencedTransactionNotFound => {
                write!(f, "ReferencedTransactionNotFound")
            }
            TransactionProcessingError::TransactionAlreadyHasPendingDisupte => {
                write!(f, "TransactionAlreadyHasPendingDisupte")
            }
            TransactionProcessingError::TransactionDoesNotHavePendingDisupte => {
                write!(f, "TransactionDoesNotHavePendingDisupte")
            }
            TransactionProcessingError::TransactionIDAlreadyExists => {
                write!(f, "TransactionIDAlreadyExists")
            }
        }
    }
}
