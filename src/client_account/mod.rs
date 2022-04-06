use std::collections::HashMap;

use crate::{ser_form, ClientId, TransactionId, TransactionType};

mod disputable_transaction;
use disputable_transaction::DisputableTransaction;

mod dispute_related_transaction;
use dispute_related_transaction::DisputeRelatedTransaction;

pub mod error;
use error::TransactionProcessingError;
#[derive(Debug)]
pub struct ClientAccount {
    pub client: ClientId,
    disputable_transactions: HashMap<TransactionId, DisputableTransaction>,
    pub balance: AccountBalance,
    pub locked: bool,
}

impl ClientAccount {
    pub fn new(client: ClientId) -> Self {
        Self {
            client,
            disputable_transactions: HashMap::new(),
            balance: AccountBalance::new(),
            locked: false,
        }
    }

    fn process_disputable_transaction(
        &mut self,
        disputable_transaction: DisputableTransaction,
    ) -> Result<(), TransactionProcessingError> {
        if self
            .disputable_transactions
            .contains_key(&disputable_transaction.transaction_id)
        {
            Err(TransactionProcessingError::TransactionIDAlreadyExists)
        } else {
            self.balance.available += disputable_transaction.amount;
            self.disputable_transactions.insert(
                disputable_transaction.transaction_id,
                disputable_transaction,
            );
            Ok(())
        }
    }

    fn process_dispute(
        &mut self,
        transaction: DisputeRelatedTransaction,
    ) -> Result<(), TransactionProcessingError> {
        let maybe_referenced_transaction = self
            .disputable_transactions
            .get_mut(&transaction.referenced_transaction_id);

        if let Some(mut referenced_transaction) = maybe_referenced_transaction {
            if referenced_transaction.is_under_dispute {
                Err(TransactionProcessingError::TransactionAlreadyHasPendingDisupte)
            } else {
                let amount = referenced_transaction.amount;
                self.balance.available -= amount;
                self.balance.held += amount;
                referenced_transaction.is_under_dispute = true;
                Ok(())
            }
        } else {
            Err(TransactionProcessingError::ReferencedTransactionNotFound)
        }
    }

    fn process_resolve(
        &mut self,
        transaction: DisputeRelatedTransaction,
    ) -> Result<(), TransactionProcessingError> {
        let maybe_referenced_transaction = self
            .disputable_transactions
            .get_mut(&transaction.referenced_transaction_id);

        if let Some(mut referenced_transaction) = maybe_referenced_transaction {
            if referenced_transaction.is_under_dispute {
                let amount = referenced_transaction.amount;
                self.balance.available += amount;
                self.balance.held -= amount;
                referenced_transaction.is_under_dispute = false;
                Ok(())
            } else {
                Err(TransactionProcessingError::TransactionDoesNotHavePendingDisupte)
            }
        } else {
            Err(TransactionProcessingError::ReferencedTransactionNotFound)
        }
    }

    fn process_chargeback(
        &mut self,
        transaction: DisputeRelatedTransaction,
    ) -> Result<(), TransactionProcessingError> {
        let maybe_referenced_transaction = self
            .disputable_transactions
            .get_mut(&transaction.referenced_transaction_id);

        if let Some(mut referenced_transaction) = maybe_referenced_transaction {
            if referenced_transaction.is_under_dispute {
                self.balance.held -= referenced_transaction.amount;
                referenced_transaction.is_under_dispute = false;
                self.locked = true;
                Ok(())
            } else {
                Err(TransactionProcessingError::TransactionDoesNotHavePendingDisupte)
            }
        } else {
            Err(TransactionProcessingError::ReferencedTransactionNotFound)
        }
    }

    pub fn process_transaction(
        &mut self,
        transaction: ser_form::Transaction,
    ) -> Result<(), TransactionProcessingError> {
        match transaction.transaction_type {
            TransactionType::Deposit => {
                let deposit_transaction = DisputableTransaction::new_deposit_transaction(
                    transaction.transaction_id,
                    transaction
                        .amount
                        .expect("amount is required for a deposit"),
                );
                self.process_disputable_transaction(deposit_transaction)?;
            }
            TransactionType::Withdrawal => {
                let deposit_transaction = DisputableTransaction::new_withdrawal_transaction(
                    transaction.transaction_id,
                    transaction
                        .amount
                        .expect("amount is required for a deposit"),
                );
                self.process_disputable_transaction(deposit_transaction)?;
            }
            TransactionType::Dispute => {
                let dispute_transaction =
                    DisputeRelatedTransaction::new_dispute_transaction(transaction.transaction_id);
                self.process_dispute(dispute_transaction)?;
            }
            TransactionType::Resolve => {
                let resolve_transaction =
                    DisputeRelatedTransaction::new_resolve_transaction(transaction.transaction_id);
                self.process_resolve(resolve_transaction)?;
            }
            TransactionType::Chargeback => {
                let resolve_transaction = DisputeRelatedTransaction::new_chargeback_transaction(
                    transaction.transaction_id,
                );
                self.process_chargeback(resolve_transaction)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_deposit() {
        let mut account = ClientAccount::new(1);

        account
            .process_disputable_transaction(DisputableTransaction::new_deposit_transaction(
                1, 100.0,
            ))
            .unwrap();

        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_process_withdrawal() {
        let mut account = ClientAccount::new(1);

        account
            .process_disputable_transaction(DisputableTransaction::new_withdrawal_transaction(
                1, 100.0,
            ))
            .unwrap();

        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, -100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), -100.0);
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_process_dispute_and_resolve() {
        let mut account = ClientAccount::new(1);

        let initial_tranaction = DisputableTransaction::new_deposit_transaction(1, 100.0);
        account
            .process_disputable_transaction(initial_tranaction)
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);

        let transaction_to_dispute = DisputableTransaction::new_deposit_transaction(2, 10.0);
        account
            .process_disputable_transaction(transaction_to_dispute)
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 110.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 110.0);
        assert_eq!(account.locked, false);

        let dispute_transaction = DisputeRelatedTransaction::new_dispute_transaction(2);
        account.process_dispute(dispute_transaction).unwrap();

        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 10.0);
        assert_eq!(account.balance.total(), 110.0);
        assert_eq!(account.locked, false);

        // get the referenced transaction and make sure it's under dispute
        let referenced_transaction = account.disputable_transactions.get(&2).unwrap();
        assert_eq!(referenced_transaction.is_under_dispute, true);

        // now resolve
        let resolve_transaction = DisputeRelatedTransaction::new_resolve_transaction(2);
        account.process_resolve(resolve_transaction).unwrap();

        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 110.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 110.0);
        assert_eq!(account.locked, false);
        let referenced_transaction = account.disputable_transactions.get(&2).unwrap();
        assert_eq!(referenced_transaction.is_under_dispute, false);
    }

    #[test]
    fn test_process_dispute_and_chargeback() {
        let mut account = ClientAccount::new(1);

        let initial_tranaction = DisputableTransaction::new_deposit_transaction(1, 100.0);
        account
            .process_disputable_transaction(initial_tranaction)
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);

        let transaction_to_dispute = DisputableTransaction::new_deposit_transaction(2, 10.0);
        account
            .process_disputable_transaction(transaction_to_dispute)
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 110.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 110.0);
        assert_eq!(account.locked, false);

        let dispute_transaction = DisputeRelatedTransaction::new_dispute_transaction(2);
        account.process_dispute(dispute_transaction).unwrap();

        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 10.0);
        assert_eq!(account.balance.total(), 110.0);
        assert_eq!(account.locked, false);

        // get the referenced transaction and make sure it's under dispute
        let referenced_transaction = account.disputable_transactions.get(&2).unwrap();
        assert_eq!(referenced_transaction.is_under_dispute, true);

        // now chargeback
        let chargeback_transaction = DisputeRelatedTransaction::new_chargeback_transaction(2);
        account.process_chargeback(chargeback_transaction).unwrap();

        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, true);
        let referenced_transaction = account.disputable_transactions.get(&2).unwrap();
        assert_eq!(referenced_transaction.is_under_dispute, false);
    }

    #[test]
    fn test_process_dispute_and_chargeback_with_withdrawal() {
        let mut account = ClientAccount::new(1);

        let initial_tranaction = DisputableTransaction::new_deposit_transaction(1, 100.0);
        account
            .process_disputable_transaction(initial_tranaction)
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);

        let transaction_to_dispute = DisputableTransaction::new_withdrawal_transaction(2, 10.0);
        account
            .process_disputable_transaction(transaction_to_dispute)
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 90.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 90.0);
        assert_eq!(account.locked, false);

        let dispute_transaction = DisputeRelatedTransaction::new_dispute_transaction(2);
        account.process_dispute(dispute_transaction).unwrap();

        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, -10.0);
        assert_eq!(account.balance.total(), 90.0);
        assert_eq!(account.locked, false);

        // get the referenced transaction and make sure it's under dispute
        let referenced_transaction = account.disputable_transactions.get(&2).unwrap();
        assert_eq!(referenced_transaction.is_under_dispute, true);

        // now chargeback
        let chargeback_transaction = DisputeRelatedTransaction::new_chargeback_transaction(2);
        account.process_chargeback(chargeback_transaction).unwrap();

        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, true);
        let referenced_transaction = account.disputable_transactions.get(&2).unwrap();
        assert_eq!(referenced_transaction.is_under_dispute, false);
    }

    #[test]
    fn test_process_disputable_transaction_returns_error_if_duplicate_tx_id() {
        let mut account = ClientAccount::new(1);
        account
            .process_disputable_transaction(DisputableTransaction::new_deposit_transaction(
                1, 100.0,
            ))
            .unwrap();
        let res = account.process_disputable_transaction(
            DisputableTransaction::new_deposit_transaction(1, 200.0),
        );
        if let Err(the_error) = res {
            assert_eq!(
                the_error,
                TransactionProcessingError::TransactionIDAlreadyExists
            );
        } else {
            panic!("Should have returned an error");
        }
    }

    #[test]
    fn test_process_dispute_resolve_or_chargeback_with_no_matching_transaction_id_returns_error() {
        let mut account = ClientAccount::new(1);

        assert_eq!(
            account.process_dispute(DisputeRelatedTransaction::new_dispute_transaction(1)),
            Err(TransactionProcessingError::ReferencedTransactionNotFound)
        );

        assert_eq!(
            account.process_resolve(DisputeRelatedTransaction::new_resolve_transaction(1)),
            Err(TransactionProcessingError::ReferencedTransactionNotFound)
        );

        assert_eq!(
            account.process_chargeback(DisputeRelatedTransaction::new_chargeback_transaction(1)),
            Err(TransactionProcessingError::ReferencedTransactionNotFound)
        );
    }

    #[test]
    fn test_process_resolve_returns_error_if_referenced_tx_is_already_under_dispute() {
        let mut account = ClientAccount::new(1);

        let initial_tranaction = DisputableTransaction::new_deposit_transaction(1, 100.0);
        account
            .process_disputable_transaction(initial_tranaction)
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);

        let transaction_to_dispute = DisputableTransaction::new_deposit_transaction(2, 10.0);
        account
            .process_disputable_transaction(transaction_to_dispute)
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 110.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 110.0);
        assert_eq!(account.locked, false);

        let dispute_transaction = DisputeRelatedTransaction::new_dispute_transaction(2);
        account.process_dispute(dispute_transaction).unwrap();

        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 10.0);
        assert_eq!(account.balance.total(), 110.0);
        assert_eq!(account.locked, false);

        let dispute_it_again_transaction = DisputeRelatedTransaction::new_dispute_transaction(2);
        let res = account.process_dispute(dispute_it_again_transaction);
        if let Err(the_error) = res {
            assert_eq!(
                the_error,
                TransactionProcessingError::TransactionAlreadyHasPendingDisupte
            );
        } else {
            panic!("Should have returned an error");
        }
    }

    #[test]
    fn test_process_resolve_returns_error_if_referenced_tx_is_not_under_dispute() {
        let mut account = ClientAccount::new(1);

        account
            .process_disputable_transaction(DisputableTransaction::new_deposit_transaction(
                1, 100.0,
            ))
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);

        let res = account.process_resolve(DisputeRelatedTransaction::new_resolve_transaction(1));
        if let Err(the_error) = res {
            assert_eq!(
                the_error,
                TransactionProcessingError::TransactionDoesNotHavePendingDisupte
            );
        } else {
            panic!("Should have returned an error");
        }

        // account balance is unaffected
        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_process_chargeback_returns_error_if_referenced_tx_is_not_under_dispute() {
        let mut account = ClientAccount::new(1);

        account
            .process_disputable_transaction(DisputableTransaction::new_deposit_transaction(
                1, 100.0,
            ))
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);

        let res =
            account.process_chargeback(DisputeRelatedTransaction::new_chargeback_transaction(1));
        if let Err(the_error) = res {
            assert_eq!(
                the_error,
                TransactionProcessingError::TransactionDoesNotHavePendingDisupte
            );
        } else {
            panic!("Should have returned an error");
        }

        // account balance is unaffected
        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_process_transaction_deposit_withdrawal() {
        let mut account = ClientAccount::new(1);

        account
            .process_disputable_transaction(DisputableTransaction::new_deposit_transaction(
                1, 100.0,
            ))
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 1);
        assert_eq!(account.balance.available, 100.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 100.0);
        assert_eq!(account.locked, false);

        account
            .process_disputable_transaction(DisputableTransaction::new_withdrawal_transaction(
                2, 25.0,
            ))
            .unwrap();
        assert_eq!(account.disputable_transactions.len(), 2);
        assert_eq!(account.balance.available, 75.0);
        assert_eq!(account.balance.held, 0.0);
        assert_eq!(account.balance.total(), 75.0);
        assert_eq!(account.locked, false);
    }
}
