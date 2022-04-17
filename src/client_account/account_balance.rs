#[derive(Debug)]
pub struct AccountBalance {
    pub available: f64,
    pub held: f64,
}

impl Default for AccountBalance {
    fn default() -> Self {
        Self {
            available: 0.0,
            held: 0.0,
        }
    }
}

impl AccountBalance {
    pub fn total(&self) -> f64 {
        self.available + self.held
    }
}
