use crate::{
    account::Account,
    error::{AccountError, TxEventError},
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionEventType {
    Deposit,
    Withdrawal,
    Resolve,
    Chargeback,
    Dispute,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TransactionEvent {
    #[serde(rename = "tx")]
    pub id: u32,
    pub r#type: TransactionEventType,
    pub client: u16,
    pub amount: Option<f64>,
}

pub trait Transactable: Send {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError>;
    fn client(&self) -> u16;
}

#[derive(Copy, Clone, Debug)]
pub struct Deposit {}
#[derive(Copy, Clone, Debug)]
pub struct Withdrawal {}
#[derive(Copy, Clone, Debug)]
pub struct Dispute {}
#[derive(Copy, Clone, Debug)]
pub struct Resolve {}
#[derive(Copy, Clone, Debug)]
pub struct Chargeback {}

#[derive(Clone, Debug)]
pub struct TransactionState<T: Clone> {
    pub tx_event: TransactionEvent,
    pub transaction: T,
}

impl<T: Transactable + Send + Clone> Transactable for TransactionState<T> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        self.transaction.apply(account)
    }

    fn client(&self) -> u16 {
        self.tx_event.client
    }
}

impl TransactionState<Deposit> {
    pub fn new(tx_event: TransactionEvent) -> Self {
        TransactionState {
            tx_event,
            transaction: Deposit {},
        }
    }
}

impl Transactable for TransactionState<Deposit> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.deposit(self)
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }
}

impl TransactionState<Withdrawal> {
    pub fn new(tx_event: TransactionEvent) -> Self {
        TransactionState {
            tx_event,
            transaction: Withdrawal {},
        }
    }
}

impl Transactable for TransactionState<Withdrawal> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.debit(self)
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }
}

impl TransactionState<Dispute> {
    pub fn new(tx_event: TransactionEvent) -> Result<Self, TxEventError> {
        if let TransactionEventType::Deposit = tx_event.r#type {
            Ok(TransactionState {
                tx_event,
                transaction: Dispute {},
            })
        } else {
            Err(TxEventError::UnexpectedTxType {
                expected: TransactionEventType::Dispute,
                actual: tx_event.r#type,
            })
        }
    }
    pub fn resolve(self) -> TransactionState<Resolve> {
        TransactionState {
            tx_event: self.tx_event,
            transaction: Resolve {},
        }
    }

    pub fn chargeback(self) -> TransactionState<Chargeback> {
        TransactionState {
            tx_event: self.tx_event,
            transaction: Chargeback {},
        }
    }
}

impl Transactable for TransactionState<Dispute> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.hold(self)
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }
}

impl Transactable for TransactionState<Resolve> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.free(self)
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }
}

impl Transactable for TransactionState<Chargeback> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.chargeback(self)
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }
}
