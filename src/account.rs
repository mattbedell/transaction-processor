use crate::error::{AccountError, TransactionError};

#[derive(Default, Clone, Copy)]
pub enum AccountStatus {
    #[default]
    Active,
    Locked,
}

#[derive(Default)]
pub struct Account {
    id: u16,
    available: f64,
    held: f64,
    status: AccountStatus,
}

impl Account {
    pub fn new(id: u16) -> Self {
        Account {
            id,
            ..Account::default()
        }
    }

    pub fn hold(&mut self, amount: f64) -> Result<(), AccountError> {
        if 0.0 < self.available - amount {
            self.available -= amount;
            self.held += amount;
            Ok(())
        } else {
            Err(AccountError::InsufficientFunds { id: self.id })
        }
    }

    // @todo: duped and dirty, clean this up
    pub fn free(&mut self, amount: f64) -> Result<(), AccountError> {
        if 0.0 < self.held - amount {
            self.held -= amount;
            self.available += amount;
            Ok(())
        } else {
            Err(AccountError::InsufficientHold { id: self.id })
        }
    }

    fn set_status(&mut self, status: AccountStatus) {
        self.status = status;
    }

    pub fn lock(&mut self) {
        self.set_status(AccountStatus::Locked)
    }

    pub fn set_active(&mut self) {
        self.set_status(AccountStatus::Active)
    }
}
