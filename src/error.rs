use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum TransactionError {
    InsufficientFunds { account: u16, tx: u32 },
}

impl Error for TransactionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TransactionError::InsufficientFunds { .. } => None,
        }
    }
}

impl Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::InsufficientFunds { account, tx } => {
                write!(f, "InsufficientFunds: account: {account}, tx: {tx}")
            }
        }
    }
}
