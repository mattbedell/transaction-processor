use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum AccountError {
    InsufficientFunds { id: u16 },
}

impl Error for AccountError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AccountError::InsufficientFunds { .. } => None,
        }
    }
}

impl Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountError::InsufficientFunds { id } => {
                write!(f, "InsufficientFunds in account: '{id}'")
            }
        }
    }
}

#[derive(Debug)]
pub enum TransactionError {
    InsufficientFunds { tx: u32, source: AccountError },
}

impl TransactionError {
    pub fn from_account_error(err: AccountError, transaction_id: u32) -> Self {
        match err {
            AccountError::InsufficientFunds { .. } => TransactionError::InsufficientFunds {
                tx: transaction_id,
                source: err,
            },
        }
    }
}

impl Error for TransactionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TransactionError::InsufficientFunds { source, .. } => Some(source),
        }
    }
}

impl Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::InsufficientFunds { tx, .. } => {
                write!(f, "InsufficientFunds tx: '{tx}'")
            }
        }
    }
}
