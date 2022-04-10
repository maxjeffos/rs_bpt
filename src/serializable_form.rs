use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde_derive::{Deserialize, Serialize};

use crate::ClientId;
use crate::TransactionId;
use crate::TransactionType;

#[derive(Debug, Serialize)]
pub struct Output {
    pub client: ClientId,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}

impl Output {
    pub fn new(client: ClientId, available: f64, held: f64, total: f64, locked: bool) -> Self {
        Self {
            client,
            available: round_f64_4dp_string(available).expect("failed to represent f64 as decimal"),
            held: round_f64_4dp_string(held).expect("failed to represent f64 as decimal"),
            total: round_f64_4dp_string(total).expect("failed to represent f64 as decimal"),
            locked,
        }
    }
}

#[derive(Debug, Deserialize)]
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
fn round_f64_4dp_string(x: f64) -> Result<String, &'static str> {
    let d = Decimal::from_f64(x).ok_or("Error converting f64 to Decimal")?;
    let rounded_decimal = d.round_dp(4);
    Ok(format!("{:.4}", rounded_decimal))
}
