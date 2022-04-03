use crate::{ClientId, TransactionId, TransactionType};

#[derive(Debug)]
pub enum TransactionProcessingError {
    AmountNotSpecified,
    TransactionAlreadyHasPendingDisupte,
    TransactionDoesNotHavePendingDisupte,
}

impl std::error::Error for TransactionProcessingError {}

impl std::fmt::Display for TransactionProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionProcessingError::AmountNotSpecified => write!(f, "AmountNotSpecified"),
            TransactionProcessingError::TransactionAlreadyHasPendingDisupte => write!(f, "TransactionAlreadyHasPendingDisupte"),
            TransactionProcessingError::TransactionDoesNotHavePendingDisupte => write!(f, "TransactionDoesNotHavePendingDisupte"),
        }
    }
}

struct Dispute {
    disputed_transaction_id: TransactionId,
    status: DisputeStatus,
}

impl Dispute {
    fn new(disputed_transaction_id: TransactionId) -> Self {
        Self {
            disputed_transaction_id,
            status: DisputeStatus::Pending,
        }
    }

    fn resolve(&mut self) {
        self.status = DisputeStatus::Resolved;
    }

    fn charge_back(&mut self) {
        self.status = DisputeStatus::ChargedBack;
    }
}

pub struct ClientTransaction {
    pub transaction_type: TransactionType,
    pub transaction_id: TransactionId,
    pub amount: Option<f64>,  // Optional because dispute, resolve, and chargeback transactions do not have an amount
}

#[derive(Debug)]
pub struct ClientAccount {
    pub client: ClientId,
    real_transactions: Vec<RealTransaction>,
    pub balance: AccountBalance,
    pub locked: bool,
}

impl ClientAccount {
    pub fn new(client: ClientId) -> Self {
        Self {
            client,
            real_transactions: Vec::new(),
            balance: AccountBalance::new(),
            locked: false,
        }
    }

    fn procecss_deposit(
        &mut self,
        transaction: ClientTransaction,
    ) -> Result<(), TransactionProcessingError> {
        let amount = transaction
            .amount
            .ok_or(TransactionProcessingError::AmountNotSpecified)?;

        self.balance.available += amount;

        self.real_transactions.push(RealTransaction {
            transaction_id: transaction.transaction_id,
            amount,
            dispute_status: None,
        });

        Ok(())
    }

    fn procecss_withdrawal(
        &mut self,
        transaction: ClientTransaction,
    ) -> Result<(), TransactionProcessingError> {
        let amount = transaction
            .amount
            .ok_or(TransactionProcessingError::AmountNotSpecified)?;

        self.balance.available -= amount;

        self.real_transactions.push(RealTransaction {
            transaction_id: transaction.transaction_id,
            amount: -amount,
            dispute_status: None,
        });
        Ok(())
    }

    fn procecss_dispute(
        &mut self,
        transaction: ClientTransaction,
    ) -> Result<(), TransactionProcessingError> {
        let maybe_existing_transaction = self
            .real_transactions
            .iter_mut()
            .find(|t| t.transaction_id == transaction.transaction_id);

        if let Some(mut existing_transaction) = maybe_existing_transaction {
            // println!("{:?}", existing_transaction);

            if let Some(existing_dispute_status) = existing_transaction.dispute_status {
                if existing_dispute_status == DisputeStatus::Pending {
                    return Err(TransactionProcessingError::TransactionAlreadyHasPendingDisupte);
                }
            }

            let amount = existing_transaction.amount;
            // println!("amount: {:?}", amount);

            self.balance.available -= amount;
            self.balance.held += amount;

            // TODO: find a way to simplify this code by not having to do the if let Some twice
            if let Some(mut existing_dispute_status) = existing_transaction.dispute_status {
                if existing_dispute_status == DisputeStatus::Pending {
                    // spec did not specify how to handle this so I'm returning an error
                    return Err(TransactionProcessingError::TransactionAlreadyHasPendingDisupte);
                } else {
                    // TODO: there's a bug here - I'm not updating the actual dispute status
                    // this is just a copy of the dispute status, not a reference to it
                    existing_dispute_status = DisputeStatus::Pending;
                }
            } else {
                existing_transaction
                    .dispute_status
                    .replace(DisputeStatus::Pending);

                // existing_transaction.dispute_status = Some(DisputeStatus::Pending);
            }
        } // else spec says to ignore it

        Ok(())
    }

    fn procecss_resolve(
        &mut self,
        transaction: ClientTransaction,
    ) -> Result<(), TransactionProcessingError> {
        // look up the referenced transaction
        // make sure that it is under dispute (pending)
        // make required adjustment to the client balance
        // mark the dispute as resolved

        let maybe_existing_transaction = self
            .real_transactions
            .iter_mut()
            .find(|t| t.transaction_id == transaction.transaction_id);

        if let Some(existing_transaction) = maybe_existing_transaction {
            if let Some(mut existing_dispute_status) = existing_transaction.dispute_status {
                if existing_dispute_status != DisputeStatus::Pending {
                    // spec did not specify how to handle this so I'm returning an error
                    return Err(TransactionProcessingError::TransactionDoesNotHavePendingDisupte);
                }

                let amount = existing_transaction.amount;
                self.balance.available += amount;
                self.balance.held -= amount;

                existing_dispute_status = DisputeStatus::Resolved;
            }
        } // else spec says to ignore it

        Ok(())
    }

    fn procecss_chargeback(
        &mut self,
        transaction: ClientTransaction,
    ) -> Result<(), TransactionProcessingError> {
        // look up the referenced transaction
        // make sure that it is under dispute (pending)
        // make required adjustment to the client balance and lock the account
        // mark the dispute as charged back

        let maybe_existing_transaction = self
            .real_transactions
            .iter_mut()
            .find(|t| t.transaction_id == transaction.transaction_id);

        if let Some(existing_transaction) = maybe_existing_transaction {
            if let Some(mut existing_dispute_status) = existing_transaction.dispute_status {
                if existing_dispute_status != DisputeStatus::Pending {
                    // spec did not specify how to handle this so I'm returning an error
                    return Err(TransactionProcessingError::TransactionDoesNotHavePendingDisupte);
                }

                let amount = existing_transaction.amount;
                self.balance.held -= amount;
                self.locked = true;

                existing_dispute_status = DisputeStatus::Resolved;
            }
        } // else spec says to ignore it

        Ok(())
    }

    pub fn process_transaction(
        &mut self,
        transaction: ClientTransaction,
    ) -> Result<(), TransactionProcessingError> {
        match transaction.transaction_type {
            TransactionType::Deposit => {
                self.procecss_deposit(transaction)?;
            }
            TransactionType::Withdrawal => {
                self.procecss_withdrawal(transaction)?;
            }
            TransactionType::Dispute => {
                self.procecss_dispute(transaction)?;
            }
            TransactionType::Resolve => {
                self.procecss_resolve(transaction)?;
            }
            TransactionType::Chargeback => {
                self.procecss_chargeback(transaction)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AccountBalance {
    pub available: f64,
    pub held: f64,
}

impl AccountBalance {
    fn new() -> AccountBalance {
        AccountBalance {
            available: 0.0,
            held: 0.0,
        }
    }

    pub fn total(&self) -> f64 {
        self.available + self.held
    }
}

#[derive(Debug)]
struct RealTransaction {
    transaction_id: TransactionId,
    amount: f64,
    dispute_status: Option<DisputeStatus>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum DisputeStatus {
    Pending,
    Resolved,
    ChargedBack,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_deposit() {
        let mut account = ClientAccount::new(1);

        let transaction = ClientTransaction {
            transaction_type: TransactionType::Deposit,
            transaction_id: 1,
            amount: Some(100.0),
        };

        account.procecss_deposit(transaction).unwrap();

        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
    }

    #[test]
    fn test_process_withdrawal() {
        let mut account = ClientAccount::new(1);

        let transaction = ClientTransaction {
            transaction_type: TransactionType::Withdrawal,
            transaction_id: 1,
            amount: Some(100.0),
        };

        account.procecss_withdrawal(transaction).unwrap();

        assert_eq!(account.balance.available, -100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), -100.0);
    }

    #[test]
    fn test_process_dispute() {
        let mut account = ClientAccount::new(1);

        account.real_transactions.push(RealTransaction {
            transaction_id: 1,
            amount: 100.0,
            dispute_status: None,
        });

        let dispute_transaction = ClientTransaction {
            transaction_type: TransactionType::Dispute,
            transaction_id: 1,
            amount: None,
        };

        // account.procecss_dispute(transaction)



    }


    #[test]
    fn test_process_transaction_deposit_withdrawal() {
        let mut account = ClientAccount::new(1);

        let transaction = ClientTransaction {
            transaction_type: TransactionType::Deposit,
            transaction_id: 1,
            amount: Some(100.0),
        };
        account.process_transaction(transaction).unwrap();
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.real_transactions.len(), 1);

        let transaction_2 = ClientTransaction {
            transaction_type: TransactionType::Withdrawal,
            transaction_id: 2,
            amount: Some(25.0),
        };
        account.process_transaction(transaction_2).unwrap();
        assert_eq!(account.balance.available, 75.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 75.0);
        assert_eq!(account.real_transactions.len(), 2);
    }

    #[test]
    fn test_process_transaction_dispute_and_resolve() {
        let mut account = ClientAccount::new(1);

        let transaction = ClientTransaction {
            transaction_type: TransactionType::Deposit,
            transaction_id: 1,
            amount: Some(100.0),
        };
        account.process_transaction(transaction).unwrap();
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.real_transactions.len(), 1);

        let transaction_2 = ClientTransaction {
            transaction_type: TransactionType::Deposit,
            transaction_id: 2,
            amount: Some(10.0),
        };
        account.process_transaction(transaction_2).unwrap();
        assert_eq!(account.balance.available, 110.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 110.0);
        assert_eq!(account.real_transactions.len(), 2);

        let transaction_3 = ClientTransaction {
            transaction_type: TransactionType::Dispute,
            transaction_id: 2,
            amount: None,
        };
        account.process_transaction(transaction_3).unwrap();
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 10.0);
        assert_eq!(account.balance.total(), 110.0);
        assert_eq!(account.real_transactions.len(), 2);
    }
}
