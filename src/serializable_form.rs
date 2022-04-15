use anyhow::anyhow;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde_derive::{Deserialize, Serialize};

use crate::{ClientAccount, ClientId, TransactionId, TransactionType};

#[derive(Debug, Serialize)]
pub struct Output {
    pub client: ClientId,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}

impl Output {
    pub fn from_client_account(client_account: &ClientAccount) -> anyhow::Result<Self> {
        Ok(Self {
            client: client_account.client_id,
            available: round_f64_4dp_string(client_account.balance.available)?,
            held: round_f64_4dp_string(client_account.balance.held)?,
            total: round_f64_4dp_string(client_account.balance.total())?,
            locked: client_account.locked,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,

    #[serde(rename = "client")]
    pub client_id: ClientId,

    #[serde(rename = "tx")]
    pub transaction_id: TransactionId,

    pub amount: Option<f64>,
}

/// Round an f64 to a Decimal using "Banker's Rounding" with max 4 decimal places and represent it as a String
fn round_f64_4dp_string(x: f64) -> anyhow::Result<String> {
    let d = Decimal::from_f64(x).ok_or(anyhow!("Failed to represent f64 as Decimal: {}", x))?;
    let rounded_decimal = d.round_dp(4);
    Ok(format!("{:.4}", rounded_decimal))
}
