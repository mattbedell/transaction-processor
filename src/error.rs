use std::error::Error;
use std::fmt::Display;

use crate::transaction::TransactionEventType;

#[derive(Debug)]
pub enum AccountError {
    InsufficientFunds { id: u16, tx: u32 },
    InsufficientHold { id: u16, tx: u32 },
    ExpectedAmountError { id: u16, tx: u32 },
    AccountFrozen { id: u16, tx: u32 },
}

impl Error for AccountError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AccountError::InsufficientFunds { .. } => None,
            AccountError::InsufficientHold { .. } => None,
            AccountError::ExpectedAmountError { .. } => None,
            AccountError::AccountFrozen { .. } => None,
        }
    }
}

impl Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountError::InsufficientFunds { id, tx } => {
                write!(f, "InsufficientFunds in account: '{id}', tx: '{tx}'")
            }
            AccountError::InsufficientHold { id, tx } => {
                write!(f, "InsufficientHold in account: '{id}', tx: '{tx}'")
            }
            AccountError::ExpectedAmountError { id, tx } => {
                write!(
                    f,
                    "ExpectedAmountError: '{id}', tx: '{tx}', expected tx amount"
                )
            }
            AccountError::AccountFrozen { id, tx } => {
                write!(
                    f,
                    "AccountFrozen: '{id}', tx: '{tx}', expected tx amount"
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum TxEventError {
    UnexpectedTxType {
        actual: TransactionEventType,
        expected: TransactionEventType,
    },
}

impl Error for TxEventError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TxEventError::UnexpectedTxType { .. } => None,
        }
    }
}

impl Display for TxEventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxEventError::UnexpectedTxType { actual, expected } => write!(
                f,
                "UnexpectedTxType: expected: '{expected:?}', actual: '{actual:?}'"
            ),
        }
    }
}
