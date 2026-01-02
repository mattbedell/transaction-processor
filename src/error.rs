use std::error::Error;
use std::fmt::Display;

use crate::transaction::{TransactionEvent, TransactionEventType};

#[derive(Debug)]
pub enum AccountTxError {
    InsufficientAvailableBalanceError { id: u16 },
    InsufficientHoldBalanceError { id: u16 },
    AccountFrozenError { id: u16 },
}

impl Error for AccountTxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AccountTxError::InsufficientAvailableBalanceError { .. } => None,
            AccountTxError::InsufficientHoldBalanceError { .. } => None,
            AccountTxError::AccountFrozenError { .. } => None,
        }
    }
}

impl Display for AccountTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountTxError::InsufficientAvailableBalanceError { id } => write!(
                f,
                "InsufficientAvailableBalanceError: Insufficient funds to complete the transaction for account: '{id}'"
            ),
            AccountTxError::InsufficientHoldBalanceError { id } => write!(
                f,
                "InsufficientHeldBalanceError: Insufficient funds to complete the transaction for account '{id}'"
            ),
            AccountTxError::AccountFrozenError { id } => write!(
                f,
                "AccountFrozenError: Account frozen, cannot complete the transaction for account: {id}"
            ),
        }
    }
}

#[derive(Debug)]
pub enum EventTxError {
    UnexpectedTxTypeError { actual: TransactionEventType },
    NotDisputableError,
    ExpectedAmountError,
}

impl Error for EventTxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EventTxError::UnexpectedTxTypeError { .. } => None,
            EventTxError::NotDisputableError => None,
            EventTxError::ExpectedAmountError => None,
        }
    }
}

impl Display for EventTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventTxError::UnexpectedTxTypeError { actual } => write!(
                f,
                "UnexpectedTxTypeError: expected: '{:?}|{:?}', actual: '{actual:?}'",
                TransactionEventType::Deposit,
                TransactionEventType::Withdrawal,
            ),
            EventTxError::NotDisputableError => {
                write!(f, "NotDisputableError: cannot dispute transaction type")
            }
            EventTxError::ExpectedAmountError => {
                write!(
                    f,
                    "ExpectedAmountError: expected transaction amount field to be Some"
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum TransactionError {
    AccountTransactionError { tx: u32, source: AccountTxError },
    TransactionEventError { tx: u32, source: EventTxError },
}

impl TransactionError {
    pub fn from_event_error(tx: TransactionEvent, err: EventTxError) -> Self {
        TransactionError::TransactionEventError { tx: tx.id, source: err }
    }
}

impl Error for TransactionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TransactionError::AccountTransactionError { source, .. } => Some(source),
            TransactionError::TransactionEventError { source, .. } => Some(source),
        }
    }
}

impl Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::AccountTransactionError { tx, .. } => {
                write!(f, "AccountTransactionError: transaction: '{tx}'")
            }
            TransactionError::TransactionEventError { tx, .. } => {
                write!(f, "TransactionEventError: transaction: '{tx}'")
            }
        }
    }
}
