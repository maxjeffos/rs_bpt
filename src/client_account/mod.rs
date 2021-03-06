use std::collections::{hash_map, HashMap};

use crate::{ClientId, TransactionId, TransactionType};

mod disputable_transaction;
use disputable_transaction::DisputableTransaction;

mod dispute_related_transaction;
use dispute_related_transaction::DisputeRelatedTransaction;

pub mod error;
use error::TransactionProcessingError;

pub mod client_account_transaction;
use client_account_transaction::ClientAccountTransaction;

pub mod account_balance;
use account_balance::AccountBalance;

#[derive(Debug)]
pub struct ClientAccount {
    pub client_id: ClientId,
    disputable_transactions: HashMap<TransactionId, DisputableTransaction>,
    pub balance: AccountBalance,
    pub locked: bool,
}

impl ClientAccount {
    pub fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            disputable_transactions: HashMap::new(),
            balance: AccountBalance::default(),
            locked: false,
        }
    }

    fn process_disputable_transaction(
        &mut self,
        disputable_transaction: DisputableTransaction,
    ) -> Result<(), TransactionProcessingError> {
        if let hash_map::Entry::Vacant(e) = self
            .disputable_transactions
            .entry(disputable_transaction.transaction_id)
        {
            self.balance.available += disputable_transaction.amount;
            e.insert(disputable_transaction);
            Ok(())
        } else {
            Err(TransactionProcessingError::TransactionIDAlreadyExists(
                disputable_transaction.transaction_id,
            ))
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
                Err(
                    TransactionProcessingError::TransactionAlreadyHasPendingDisupte(
                        transaction.referenced_transaction_id,
                    ),
                )
            } else {
                let amount = referenced_transaction.amount;
                self.balance.available -= amount;
                self.balance.held += amount;
                referenced_transaction.is_under_dispute = true;
                Ok(())
            }
        } else {
            Err(TransactionProcessingError::ReferencedTransactionNotFound(
                transaction.referenced_transaction_id,
            ))
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
                Err(
                    TransactionProcessingError::TransactionDoesNotHavePendingDisupte(
                        transaction.referenced_transaction_id,
                    ),
                )
            }
        } else {
            Err(TransactionProcessingError::ReferencedTransactionNotFound(
                transaction.referenced_transaction_id,
            ))
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
                Err(
                    TransactionProcessingError::TransactionDoesNotHavePendingDisupte(
                        transaction.referenced_transaction_id,
                    ),
                )
            }
        } else {
            Err(TransactionProcessingError::ReferencedTransactionNotFound(
                transaction.referenced_transaction_id,
            ))
        }
    }

    fn log_error(
        &self,
        debug_logger: &mut dyn std::io::Write,
        transaction: &ClientAccountTransaction,
        error: TransactionProcessingError,
    ) {
        writeln!(debug_logger, "error processing transaction - {}", error)
            .expect("error writing to debug stream");
        writeln!(debug_logger, "{:?}", transaction).expect("error writing to debug stream");
    }

    pub fn process_client_transaction(
        &mut self,
        transaction: ClientAccountTransaction,
        debug_logger: &mut dyn std::io::Write,
    ) {
        let res: Result<(), TransactionProcessingError> = match transaction.transaction_type {
            TransactionType::Deposit => {
                if let Some(amount) = transaction.amount {
                    self.process_disputable_transaction(
                        DisputableTransaction::new_deposit_transaction(
                            transaction.transaction_id,
                            amount,
                        ),
                    )
                } else {
                    Err(TransactionProcessingError::AmountNotPresentForDeposit(
                        transaction.transaction_id,
                    ))
                }
            }
            TransactionType::Withdrawal => {
                if let Some(amount) = transaction.amount {
                    self.process_disputable_transaction(
                        DisputableTransaction::new_withdrawal_transaction(
                            transaction.transaction_id,
                            amount,
                        ),
                    )
                } else {
                    Err(TransactionProcessingError::AmountNotPresentForWithdrawal(
                        transaction.transaction_id,
                    ))
                }
            }
            TransactionType::Dispute => self.process_dispute(
                DisputeRelatedTransaction::new_dispute_transaction(transaction.transaction_id),
            ),
            TransactionType::Resolve => self.process_resolve(
                DisputeRelatedTransaction::new_resolve_transaction(transaction.transaction_id),
            ),
            TransactionType::Chargeback => self.process_chargeback(
                DisputeRelatedTransaction::new_chargeback_transaction(transaction.transaction_id),
            ),
        };

        if let Err(e) = res {
            self.log_error(debug_logger, &transaction, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod process_disputable_transaction {
        use super::*;

        #[test]
        fn it_returns_error_transaction_id_already_exists() {
            let mut account = ClientAccount::new(1);

            account
                .process_disputable_transaction(DisputableTransaction::new_deposit_transaction(
                    1, 100.0,
                ))
                .unwrap();

            assert_eq!(
                account.process_disputable_transaction(
                    DisputableTransaction::new_deposit_transaction(1, 200.0),
                ),
                Err(TransactionProcessingError::TransactionIDAlreadyExists(1)),
            );
        }

        #[test]
        fn works_for_deposit() {
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
        fn works_for_withdrawal() {
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
    }

    // edge cases for various process_xyz scenarios

    #[test]
    fn test_process_dispute_resolve_or_chargeback_with_no_matching_transaction_id_returns_error() {
        let mut account = ClientAccount::new(1);

        assert_eq!(
            account.process_dispute(DisputeRelatedTransaction::new_dispute_transaction(1)),
            Err(TransactionProcessingError::ReferencedTransactionNotFound(1))
        );

        assert_eq!(
            account.process_resolve(DisputeRelatedTransaction::new_resolve_transaction(1)),
            Err(TransactionProcessingError::ReferencedTransactionNotFound(1))
        );

        assert_eq!(
            account.process_chargeback(DisputeRelatedTransaction::new_chargeback_transaction(1)),
            Err(TransactionProcessingError::ReferencedTransactionNotFound(1))
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
                TransactionProcessingError::TransactionAlreadyHasPendingDisupte(2)
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
                TransactionProcessingError::TransactionDoesNotHavePendingDisupte(1)
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
                TransactionProcessingError::TransactionDoesNotHavePendingDisupte(1)
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

    // flows. maybe these should use process_client_transaction instead?

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

    #[test]
    fn test_deposit_dispute_and_resolve() {
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

    #[cfg(test)]
    mod process_client_transaction {
        use super::*;

        #[test]
        fn it_should_ignore_errors_generated_from_process_disputable_transaction_when_transaction_id_already_exists(
        ) {
            let mut account = ClientAccount::new(1);
            let mut debug_logger = Vec::<u8>::new();

            account.process_client_transaction(
                ClientAccountTransaction {
                    transaction_type: TransactionType::Deposit,
                    transaction_id: 1,
                    amount: Some(100.0),
                },
                &mut debug_logger,
            );
            assert_eq!(account.balance.available, 100.0);
            assert_eq!(account.balance.held, 0.0);
            assert_eq!(account.balance.total(), 100.0);
            assert_eq!(account.locked, false);
            let error_log_str = std::str::from_utf8(&debug_logger).unwrap();
            assert_eq!(error_log_str, "",);

            // another transaction (deposit) with the same transaction id
            account.process_client_transaction(
                ClientAccountTransaction {
                    transaction_type: TransactionType::Deposit,
                    transaction_id: 1,
                    amount: Some(200.0),
                },
                &mut debug_logger,
            );
            assert_eq!(account.balance.available, 100.0);
            assert_eq!(account.balance.held, 0.0);
            assert_eq!(account.balance.total(), 100.0);
            assert_eq!(account.locked, false);
            let error_log_str = std::str::from_utf8(&debug_logger).unwrap();
            assert!(
                error_log_str.contains("error processing transaction - TransactionIDAlreadyExists")
            );
            assert!(error_log_str.contains("Deposit"));
            assert!(error_log_str.contains("transaction_id: 1"));
            debug_logger.clear();

            // another transaction (withdrawal) with the same transaction id

            account.process_client_transaction(
                ClientAccountTransaction {
                    transaction_type: TransactionType::Withdrawal,
                    transaction_id: 1,
                    amount: Some(50.0),
                },
                &mut debug_logger,
            );
            assert_eq!(account.balance.available, 100.0);
            assert_eq!(account.balance.held, 0.0);
            assert_eq!(account.balance.total(), 100.0);
            assert_eq!(account.locked, false);
            let error_log_str = std::str::from_utf8(&debug_logger).unwrap();
            assert!(
                error_log_str.contains("error processing transaction - TransactionIDAlreadyExists")
            );
            assert!(error_log_str.contains("Withdrawal"));
            assert!(error_log_str.contains("transaction_id: 1"));
            debug_logger.clear();
        }

        #[test]
        fn it_should_ignore_deposit_and_withdrawal_transactions_with_no_amount() {
            let mut account = ClientAccount::new(1);
            let mut debug_logger = Vec::<u8>::new();

            // deposit
            account.process_client_transaction(
                ClientAccountTransaction {
                    transaction_type: TransactionType::Deposit,
                    transaction_id: 1,
                    amount: None,
                },
                &mut debug_logger,
            );
            assert_eq!(account.balance.available, 0.0);
            assert_eq!(account.balance.held, 0.0);
            assert_eq!(account.balance.total(), 0.0);
            assert_eq!(account.locked, false);

            let error_log_str = std::str::from_utf8(&debug_logger).unwrap();
            assert!(
                error_log_str.contains("error processing transaction - AmountNotPresentForDeposit")
            );
            assert!(error_log_str.contains("Deposit"));
            assert!(error_log_str.contains("transaction_id: 1"));
            debug_logger.clear();

            // same for a withdrawal
            account.process_client_transaction(
                ClientAccountTransaction {
                    transaction_type: TransactionType::Withdrawal,
                    transaction_id: 1,
                    amount: None,
                },
                &mut debug_logger,
            );
            assert_eq!(account.balance.available, 0.0);
            assert_eq!(account.balance.held, 0.0);
            assert_eq!(account.balance.total(), 0.0);
            assert_eq!(account.locked, false);

            let error_log_str = std::str::from_utf8(&debug_logger).unwrap();
            assert!(error_log_str
                .contains("error processing transaction - AmountNotPresentForWithdrawal"));
            assert!(error_log_str.contains("Withdrawal"));
            assert!(error_log_str.contains("transaction_id: 1"));
            debug_logger.clear();
        }

        // This test makes sure that errors generated from the process_dispute, process_resolve, and process_chargeback
        // are ignored. Why not just not have them return an error and ignore the conditions that generate the error?
        // Because this way, we can better test that the process_xyz functions are working properly and because
        // it gives the option of (maybe in the future) logging those errors in some way.
        #[test]
        fn it_should_handle_errors_when_dispute_resolve_or_chargeback_transactions_refer_to_a_non_existing_transaction(
        ) {
            let mut account = ClientAccount::new(1);
            let mut debug_logger = Vec::<u8>::new();

            account.process_client_transaction(
                ClientAccountTransaction {
                    transaction_type: TransactionType::Dispute,
                    transaction_id: 1,
                    amount: None,
                },
                &mut debug_logger,
            );
            let error_log_str = std::str::from_utf8(&debug_logger).unwrap();
            assert!(error_log_str
                .contains("error processing transaction - ReferencedTransactionNotFound"));
            assert!(error_log_str.contains("Dispute"));
            assert!(error_log_str.contains("transaction_id: 1"));
            debug_logger.clear();

            account.process_client_transaction(
                ClientAccountTransaction {
                    transaction_type: TransactionType::Resolve,
                    transaction_id: 1,
                    amount: None,
                },
                &mut debug_logger,
            );
            let error_log_str = std::str::from_utf8(&debug_logger).unwrap();
            assert!(error_log_str
                .contains("error processing transaction - ReferencedTransactionNotFound"));
            assert!(error_log_str.contains("Resolve"));
            assert!(error_log_str.contains("transaction_id: 1"));
            debug_logger.clear();

            account.process_client_transaction(
                ClientAccountTransaction {
                    transaction_type: TransactionType::Chargeback,
                    transaction_id: 1,
                    amount: None,
                },
                &mut debug_logger,
            );
            let error_log_str = std::str::from_utf8(&debug_logger).unwrap();
            assert!(error_log_str
                .contains("error processing transaction - ReferencedTransactionNotFound"));
            assert!(error_log_str.contains("Chargeback"));
            assert!(error_log_str.contains("transaction_id: 1"));
            debug_logger.clear();
        }

        // this test is similar to the one with the same name above, but exercises process_client_transaction
        // for each step.
        #[test]
        fn test_deposit_dispute_and_resolve() {
            let mut account = ClientAccount::new(1);
            let mut debug_logger = Vec::<u8>::new();

            let deposit = ClientAccountTransaction {
                transaction_type: TransactionType::Deposit,
                transaction_id: 1,
                amount: Some(100.0),
            };
            account.process_client_transaction(deposit, &mut debug_logger);
            assert_eq!(account.disputable_transactions.len(), 1);
            assert_eq!(account.balance.available, 100.0);
            assert_eq!(account.balance.held, 0.0);
            assert_eq!(account.balance.total(), 100.0);
            assert_eq!(account.locked, false);
            assert_eq!(debug_logger.len(), 0);

            let transaction_to_dispute = ClientAccountTransaction {
                transaction_type: TransactionType::Deposit,
                transaction_id: 2,
                amount: Some(10.0),
            };
            account.process_client_transaction(transaction_to_dispute, &mut debug_logger);
            assert_eq!(account.disputable_transactions.len(), 2);
            assert_eq!(account.balance.available, 110.0);
            assert_eq!(account.balance.held, 0.0);
            assert_eq!(account.balance.total(), 110.0);
            assert_eq!(account.locked, false);
            assert_eq!(debug_logger.len(), 0);

            let dispute = ClientAccountTransaction {
                transaction_type: TransactionType::Dispute,
                transaction_id: 2,
                amount: None,
            };
            account.process_client_transaction(dispute, &mut debug_logger);
            assert_eq!(account.disputable_transactions.len(), 2);
            assert_eq!(account.balance.available, 100.0);
            assert_eq!(account.balance.held, 10.0);
            assert_eq!(account.balance.total(), 110.0);
            assert_eq!(account.locked, false);
            assert_eq!(debug_logger.len(), 0);

            // get the referenced transaction and make sure it's under dispute
            let referenced_transaction = account.disputable_transactions.get(&2).unwrap();
            assert_eq!(referenced_transaction.is_under_dispute, true);

            // now resolve
            let resolve = ClientAccountTransaction {
                transaction_type: TransactionType::Resolve,
                transaction_id: 2,
                amount: None,
            };
            account.process_client_transaction(resolve, &mut debug_logger);

            assert_eq!(account.disputable_transactions.len(), 2);
            assert_eq!(account.balance.available, 110.0);
            assert_eq!(account.balance.held, 0.0);
            assert_eq!(account.balance.total(), 110.0);
            assert_eq!(account.locked, false);
            let referenced_transaction = account.disputable_transactions.get(&2).unwrap();
            assert_eq!(referenced_transaction.is_under_dispute, false);
            assert_eq!(debug_logger.len(), 0);
        }
    }
}
