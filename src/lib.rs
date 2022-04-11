use serde_derive::Deserialize;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

pub mod client_account;
use client_account::{
    client_account_transaction::ClientAccountTransaction, error::TransactionProcessingError,
    ClientAccount,
};
pub mod serializable_form;

pub type ClientId = u16;
pub type TransactionId = u32;

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,

    #[serde(rename = "withdrawal")]
    Withdrawal,

    #[serde(rename = "dispute")]
    Dispute,

    #[serde(rename = "resolve")]
    Resolve,

    #[serde(rename = "chargeback")]
    Chargeback,
}

fn process_transaction(
    accounts: &mut HashMap<ClientId, ClientAccount>,
    transaction: &serializable_form::Transaction,
    debug_logger: &mut dyn std::io::Write,
) -> Result<(), TransactionProcessingError> {
    let client_account = accounts
        .entry(transaction.client_id)
        .or_insert_with(|| ClientAccount::new(transaction.client_id));

    let client_account_transaction = ClientAccountTransaction::from(transaction);
    client_account.process_client_transaction(client_account_transaction, debug_logger)?;

    Ok(())
}

pub fn process_transactions_file(
    accounts: &mut HashMap<ClientId, ClientAccount>,
    input_transactions_file: PathBuf,
    debug_logger: &mut dyn std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(input_transactions_file)?;

    for transaction in reader.deserialize() {
        process_transaction(accounts, &transaction?, debug_logger)?;
    }

    Ok(())
}

pub fn write_output(
    output: &[serializable_form::Output],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout_writer = csv::Writer::from_writer(io::stdout());
    for output in output {
        stdout_writer
            .serialize(output)
            .expect("failed to write CSV record");
    }

    Ok(())
}

pub fn create_serializable_output_from_accounts(
    accounts: &HashMap<ClientId, ClientAccount>,
) -> Vec<serializable_form::Output> {
    accounts
        .iter()
        .map(|(client_id, account)| {
            serializable_form::Output::new(
                *client_id,
                account.balance.available,
                account.balance.held,
                account.balance.total(),
                account.locked,
            )
        })
        .collect()
}

pub fn cli(
    input_file: PathBuf,
    debug_logger: &mut dyn std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut accounts = HashMap::<ClientId, ClientAccount>::new();
    process_transactions_file(&mut accounts, input_file, debug_logger)?;

    let serializable_output = create_serializable_output_from_accounts(&accounts);
    write_output(&serializable_output)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_transaction_creates_a_new_client_as_required() {
        let mut accounts = HashMap::<ClientId, ClientAccount>::new();

        let transaction_1 = serializable_form::Transaction {
            client_id: 1,
            transaction_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(100.0),
        };
        process_transaction(&mut accounts, &transaction_1, &mut std::io::sink()).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[&1].balance.available, 100.0);

        let transaction_2 = serializable_form::Transaction {
            client_id: 2,
            transaction_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(1000.0),
        };
        process_transaction(&mut accounts, &transaction_2, &mut std::io::sink()).unwrap();
        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[&2].balance.available, 1000.0);
    }

    #[test]
    fn test_transactions_flow() {
        // init deposit to client 1
        // init deposit to client 2
        // a second deposit to client 1 - to dispute
        // dispute client 1 transaction 2
        // resolve client 1 transaction 2
        // a second deposit to client 2 - to dispute
        // dispute client 2 transaction 2
        // chargeback client 2 transaction 2

        let mut accounts = HashMap::<ClientId, ClientAccount>::new();

        let mut transactions = Vec::<serializable_form::Transaction>::new();

        let t_client_1_tx_1 = serializable_form::Transaction {
            client_id: 1,
            transaction_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(100.0),
        };
        let t_client_2_tx_1 = serializable_form::Transaction {
            client_id: 2,
            transaction_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(1000.0),
        };

        // Client 1 dispute-resolve flow
        let t_client_1_tx_2_to_dispute = serializable_form::Transaction {
            client_id: 1,
            transaction_id: 2,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.0),
        };
        let t_client_1_dispute_tx_2 = serializable_form::Transaction {
            client_id: 1,
            transaction_id: 2,
            transaction_type: TransactionType::Dispute,
            amount: None,
        };
        let t_client_1_resolve_tx_2 = serializable_form::Transaction {
            client_id: 1,
            transaction_id: 2,
            transaction_type: TransactionType::Resolve,
            amount: None,
        };

        // Client 2 dispute-chargeback flow
        let t_client_2_tx_2_to_dispute = serializable_form::Transaction {
            client_id: 2,
            transaction_id: 2,
            transaction_type: TransactionType::Deposit,
            amount: Some(100.0),
        };
        let t_client_2_dispute_tx_2 = serializable_form::Transaction {
            client_id: 2,
            transaction_id: 2,
            transaction_type: TransactionType::Dispute,
            amount: None,
        };
        let t_client_2_chargeback_tx_2 = serializable_form::Transaction {
            client_id: 2,
            transaction_id: 2,
            transaction_type: TransactionType::Chargeback,
            amount: None,
        };

        transactions.push(t_client_1_tx_1);
        transactions.push(t_client_2_tx_1);
        transactions.push(t_client_1_tx_2_to_dispute);
        transactions.push(t_client_1_dispute_tx_2);
        transactions.push(t_client_1_resolve_tx_2);
        transactions.push(t_client_2_tx_2_to_dispute);
        transactions.push(t_client_2_dispute_tx_2);
        transactions.push(t_client_2_chargeback_tx_2);

        for transaction in transactions {
            process_transaction(&mut accounts, &transaction, &mut std::io::sink()).unwrap();
        }

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[&1].balance.available, 110.0);
        assert_eq!(accounts[&1].balance.held, 0.0);
        assert_eq!(accounts[&1].balance.total(), 110.0);
        assert_eq!(accounts[&1].locked, false);

        assert_eq!(accounts[&2].balance.available, 1000.0);
        assert_eq!(accounts[&2].balance.held, 0.0);
        assert_eq!(accounts[&2].balance.total(), 1000.0);
        assert_eq!(accounts[&2].locked, true);

        let output = create_serializable_output_from_accounts(&accounts);

        assert_eq!(output.len(), 2);
        let client_1_output = output.iter().find(|output| output.client == 1).unwrap();
        let client_2_output = output.iter().find(|output| output.client == 2).unwrap();

        assert_eq!(client_1_output.available, "110.0000");
        assert_eq!(client_1_output.held, "0.0000");
        assert_eq!(client_1_output.total, "110.0000");
        assert_eq!(client_1_output.locked, false);

        assert_eq!(client_2_output.available, "1000.0000");
        assert_eq!(client_2_output.held, "0.0000");
        assert_eq!(client_2_output.total, "1000.0000");
        assert_eq!(client_2_output.locked, true);
    }
}
