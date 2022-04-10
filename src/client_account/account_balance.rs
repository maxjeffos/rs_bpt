#[derive(Debug)]
pub struct AccountBalance {
    pub available: f64,
    pub held: f64,
}

impl AccountBalance {
    pub fn new() -> AccountBalance {
        AccountBalance {
            available: 0.0,
            held: 0.0,
        }
    }

    pub fn total(&self) -> f64 {
        self.available + self.held
    }
}
