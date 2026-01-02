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

impl TryFrom<TransactionEvent> for Box<dyn Transactable> {
    type Error = TxEventError;
    fn try_from(tx_event: TransactionEvent) -> Result<Self, Self::Error> {
        match tx_event.r#type {
            TransactionEventType::Deposit => Ok(Box::new(TransactionState {
                tx_event,
                transaction: Deposit {},
            })),
            TransactionEventType::Withdrawal => Ok(Box::new(TransactionState {
                tx_event,
                transaction: Withdrawal {},
            })),
            TransactionEventType::Dispute
            | TransactionEventType::Resolve
            | TransactionEventType::Chargeback => Err(TxEventError::UnexpectedTxType {
                actual: tx_event.r#type,
            }),
        }
    }
}

pub trait Disputable: Transactable {
    fn resolve(self: Box<Self>) -> Box<dyn Transactable>;
    fn chargeback(self: Box<Self>) -> Box<dyn Transactable>;
    fn cloned(&self) -> Box<dyn Disputable>;
}

pub trait Transactable: Send {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError>;
    fn client(&self) -> u16;
    fn try_dispute(self: Box<Self>) -> Result<Box<dyn Disputable>, TxEventError> {
        Err(TxEventError::NotDisputable)
    }
    fn id(&self) -> u32;
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

impl Transactable for TransactionState<Deposit> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.deposit(self)
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn try_dispute(self: Box<Self>) -> Result<Box<dyn Disputable>, TxEventError> {
        Ok(Box::new(TransactionState {
            tx_event: self.tx_event,
            transaction: Dispute {},
        }))
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}

impl Transactable for TransactionState<Withdrawal> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.debit(self)
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}

impl Disputable for TransactionState<Dispute> {
    fn resolve(self: Box<Self>) -> Box<dyn Transactable> {
        Box::new(TransactionState {
            tx_event: self.tx_event,
            transaction: Resolve {},
        })
    }

    fn chargeback(self: Box<Self>) -> Box<dyn Transactable> {
        Box::new(TransactionState {
            tx_event: self.tx_event,
            transaction: Chargeback {},
        })
    }
    fn cloned(&self) -> Box<dyn Disputable> {
        Box::new(self.clone())
    }
}

impl Transactable for TransactionState<Dispute> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.hold(self)
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}

impl Transactable for TransactionState<Resolve> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.free(self)
    }

    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}

impl Transactable for TransactionState<Chargeback> {
    fn apply(&self, account: &mut Account) -> Result<(), AccountError> {
        account.chargeback(self)
    }

    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}
