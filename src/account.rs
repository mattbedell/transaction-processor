use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{RwLock, RwLockWriteGuard};
use tokio::task::JoinHandle;

use crate::error::AccountTxError;
use crate::transaction::Transactable;

#[derive(Default, Clone, Copy)]
pub enum AccountStatus {
    #[default]
    Active,
    Locked,
}

#[derive(Default, Clone, Copy)]
pub struct AccountState {
    available: f64,
    held: f64,
    status: AccountStatus,
}

pub struct AccountTx<'a> {
    id: u16,
    lock: RwLockWriteGuard<'a, AccountState>,
    state: AccountState,
}

pub enum AccountOp {
    Add(f64),
    Sub(f64),
}

impl AccountOp {
    pub fn apply(self, num: f64) -> f64 {
        match self {
            AccountOp::Add(op_num) => num + op_num,
            AccountOp::Sub(op_num) => num - op_num,
        }
    }
}

impl<'a> AccountTx<'a> {
    pub fn available(&mut self, op: AccountOp) {
        self.state.available = op.apply(self.state.available);
    }

    pub fn held(&mut self, op: AccountOp) {
        self.state.held = op.apply(self.state.held);
    }

    pub fn lock(&mut self) {
        self.state.status = AccountStatus::Locked;
    }

    fn verify(&self) -> Result<(), AccountTxError> {
        if let AccountStatus::Locked = self.lock.status {
            return Err(AccountTxError::AccountFrozenError { id: self.id });
        }

        if self.state.available < 0.0 {
            return Err(AccountTxError::InsufficientAvailableBalanceError { id: self.id });
        }

        if self.state.held < 0.0 {
            return Err(AccountTxError::InsufficientHoldBalanceError { id: self.id });
        }

        Ok(())
    }

    pub fn apply(mut self) -> Result<(), AccountTxError> {
        self.verify()?;
        *self.lock = self.state;
        Ok(())
    }
}

#[derive(Default)]
pub struct Account {
    id: u16,
    state: RwLock<AccountState>,
}

impl Account {
    pub fn new(id: u16) -> Self {
        Account {
            id,
            ..Account::default()
        }
    }

    pub async fn tx(&self) -> AccountTx<'_> {
        let lock = self.state.write().await;
        let state = *lock;
        AccountTx {
            id: self.id,
            lock,
            state,
        }
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
            let state = account.state.try_read().unwrap();
            write!(
                f,
                "\n{},{},{},{},{}",
                account.id,
                format_f64(state.available),
                format_f64(state.held),
                format_f64(state.available + state.held),
                matches!(state.status, AccountStatus::Locked)
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
                let account_tx = account.tx().await;
                let _ = tx.apply(account_tx).inspect_err(|err| eprintln!("{err}"));
            }
            accounts
        })
    }
}
