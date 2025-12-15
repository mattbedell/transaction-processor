use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::{
    error::AccountError,
    transaction::{
        Chargeback, Deposit, Dispute, Resolve, Transactable, TransactionState, Withdrawal,
    },
};

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

    pub fn deposit(&mut self, tx: &TransactionState<Deposit>) -> Result<(), AccountError> {
        if let AccountStatus::Locked = self.status {
            return Err(AccountError::AccountFrozen {
                id: self.id,
                tx: tx.tx_event.id,
            });
        }
        let amount = tx
            .tx_event
            .amount
            .ok_or(AccountError::ExpectedAmountError {
                id: self.id,
                tx: tx.tx_event.id,
            })?;

        self.available += amount;
        Ok(())
    }

    pub fn debit(&mut self, tx: &TransactionState<Withdrawal>) -> Result<(), AccountError> {
        if let AccountStatus::Locked = self.status {
            return Err(AccountError::AccountFrozen {
                id: self.id,
                tx: tx.tx_event.id,
            });
        }
        let amount = tx
            .tx_event
            .amount
            .ok_or(AccountError::ExpectedAmountError {
                id: self.id,
                tx: tx.tx_event.id,
            })?;

        if 0.0 <= self.available - amount {
            self.available -= amount;
            Ok(())
        } else {
            Err(AccountError::InsufficientFunds {
                id: self.id,
                tx: tx.tx_event.id,
            })
        }
    }

    pub fn hold(&mut self, tx: &TransactionState<Dispute>) -> Result<(), AccountError> {
        if let AccountStatus::Locked = self.status {
            return Err(AccountError::AccountFrozen {
                id: self.id,
                tx: tx.tx_event.id,
            });
        }
        let amount = tx
            .tx_event
            .amount
            .ok_or(AccountError::ExpectedAmountError {
                id: self.id,
                tx: tx.tx_event.id,
            })?;

        if 0.0 <= self.available - amount {
            self.available -= amount;
            self.held += amount;
            Ok(())
        } else {
            Err(AccountError::InsufficientFunds {
                id: self.id,
                tx: tx.tx_event.id,
            })
        }
    }

    // @todo: duped and dirty, clean this up
    pub fn free(&mut self, tx: &TransactionState<Resolve>) -> Result<(), AccountError> {
        if let AccountStatus::Locked = self.status {
            return Err(AccountError::AccountFrozen {
                id: self.id,
                tx: tx.tx_event.id,
            });
        }
        let amount = tx
            .tx_event
            .amount
            .ok_or(AccountError::ExpectedAmountError {
                id: self.id,
                tx: tx.tx_event.id,
            })?;

        if 0.0 <= self.held - amount {
            self.held -= amount;
            self.available += amount;
            Ok(())
        } else {
            Err(AccountError::InsufficientHold {
                id: self.id,
                tx: tx.tx_event.id,
            })
        }
    }

    pub fn chargeback(&mut self, tx: &TransactionState<Chargeback>) -> Result<(), AccountError> {
        if let AccountStatus::Locked = self.status {
            return Err(AccountError::AccountFrozen {
                id: self.id,
                tx: tx.tx_event.id,
            });
        }
        let amount = tx
            .tx_event
            .amount
            .ok_or(AccountError::ExpectedAmountError {
                id: self.id,
                tx: tx.tx_event.id,
            })?;

        if 0.0 <= self.held - amount {
            self.held -= amount;
            self.lock();
            Ok(())
        } else {
            Err(AccountError::InsufficientHold {
                id: self.id,
                tx: tx.tx_event.id,
            })
        }
    }

    fn set_status(&mut self, status: AccountStatus) {
        self.status = status;
    }

    pub fn lock(&mut self) {
        self.set_status(AccountStatus::Locked)
    }
}

#[derive(Default)]
pub struct Accounts(HashMap<u16, Account>);

impl Deref for Accounts {
    type Target = HashMap<u16, Account>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Accounts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// @todo: this is bad
fn format_f64(num: f64) -> String {
    let fract = num.fract();
    let precision = if fract == 0.0 {
        1
    } else {
        let str = format!("{fract}");
        str[2..].trim_end_matches('0').len().min(4)
    };

    let fmt = format!("{:.*}", precision, num);
    if fmt.ends_with(".0") {
        fmt
    } else {
        fmt.trim_end_matches('0').to_owned()
    }
}

impl Display for Accounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client,available,held,total,locked")?;

        self.0.values().try_for_each(|account| {
            write!(
                f,
                "\n{},{},{},{},{}",
                account.id,
                format_f64(account.available),
                format_f64(account.held),
                format_f64(account.available + account.held),
                matches!(account.status, AccountStatus::Locked)
            )
        })
    }
}

pub struct AccountProcessor {
    rx: Receiver<Box<dyn Transactable>>,
    pub tx: Sender<Box<dyn Transactable>>,
    accounts: Accounts,
}

impl AccountProcessor {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(10_000);
        AccountProcessor {
            rx,
            tx,
            accounts: Default::default(),
        }
    }

    pub fn run(mut self) -> JoinHandle<Accounts> {
        tokio::task::spawn(async move {
            let mut accounts = self.accounts;
            while let Some(tx) = self.rx.recv().await {
                let account = accounts
                    .entry(tx.client())
                    .or_insert_with(|| Account::new(tx.client()));
                let _ = tx.apply(account).inspect_err(|err| eprintln!("{err}"));
            }
            accounts
        })
    }
}
