use std::collections::HashMap;
use std::io;

use serde_derive::Deserialize;

pub mod client_account;
use client_account::{ClientAccount, ClientTransaction, TransactionProcessingError};
pub mod ser_form;

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
    transaction: ser_form::Transaction,
) -> Result<(), TransactionProcessingError> {
    println!("{:?}", transaction);

    let client_account = accounts
        .entry(transaction.client_id)
        .or_insert(ClientAccount::new(transaction.client_id));

    client_account.process_transaction(ClientTransaction {
        transaction_type: transaction.transaction_type,
        transaction_id: transaction.transaction_id,
        amount: transaction.amount,
    })?;

    Ok(())
}

pub fn process_transactions_file(
    input_transactions_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut accounts = HashMap::<ClientId, ClientAccount>::new();
    let mut reader = csv::Reader::from_path(input_transactions_file)?;

    for result in reader.deserialize() {
        let transation: ser_form::Transaction = result.unwrap();
        process_transaction(&mut accounts, transation)?;
    }

    let output_format_outputs: Vec<ser_form::Output> = accounts
        .iter()
        .map(|(client_id, account)| ser_form::Output {
            client: *client_id,
            available: account.balance.available,
            held: account.balance.held,
            total: account.balance.total(),
            locked: account.locked,
        })
        .collect();

    // write CSV output
    let mut stdout_writer = csv::Writer::from_writer(io::stdout());
    // stdout_writer.write_record(&["client", "available", "held", "total", "locked"]).expect("failed to write CSV header");
    for output in output_format_outputs {
        stdout_writer
            .serialize(output)
            .expect("failed to write CSV record");
    }

    Ok(())
}
