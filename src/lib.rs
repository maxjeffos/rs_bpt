use serde_derive::Deserialize;
use std::collections::HashMap;
use std::io;

pub mod client_account;
use client_account::{
    client_account_transaction::ClientAccountTransaction, error::TransactionProcessingError,
    ClientAccount,
};
pub mod serializable_form;

pub type ClientId = u16;
pub type TransactionId = u32;

#[derive(Debug, Deserialize, PartialEq)]
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
    transaction: serializable_form::Transaction,
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
    input_transactions_file: String,
    debug_logger: &mut dyn std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut accounts = HashMap::<ClientId, ClientAccount>::new();
    let mut reader = csv::Reader::from_path(input_transactions_file)?;

    for result in reader.deserialize() {
        let transation: serializable_form::Transaction = result.unwrap();
        process_transaction(&mut accounts, transation, debug_logger)?;
    }

    let output_format_outputs: Vec<serializable_form::Output> = accounts
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
        .collect();

    // write CSV output
    let mut stdout_writer = csv::Writer::from_writer(io::stdout());
    for output in output_format_outputs {
        stdout_writer
            .serialize(output)
            .expect("failed to write CSV record");
    }

    Ok(())
}
