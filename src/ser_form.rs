use serde_derive::{Deserialize, Serialize};

use crate::ClientId;
use crate::TransactionId;
use crate::TransactionType;

#[derive(Debug, Serialize)]
pub struct Output {
    pub client: ClientId,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,

    #[serde(rename = "client")]
    pub client_id: ClientId,

    #[serde(rename = "tx")]
    pub transaction_id: TransactionId,

    pub amount: Option<f64>, // TODO: make this a decimal
}
