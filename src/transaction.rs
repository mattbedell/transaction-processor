use crate::{
    account::{AccountOp, AccountTx},
    error::{AccountTxError, EventTxError, TransactionError},
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
    type Error = TransactionError;
    fn try_from(tx_event: TransactionEvent) -> Result<Self, Self::Error> {
        match tx_event.r#type {
            TransactionEventType::Deposit => {
                let tx = TransactionState::<Deposit>::new(tx_event)?;
                Ok(Box::new(tx))
            }
            TransactionEventType::Withdrawal => {
                let tx = TransactionState::<Withdrawal>::new(tx_event)?;
                Ok(Box::new(tx))
            }
            TransactionEventType::Dispute
            | TransactionEventType::Resolve
            | TransactionEventType::Chargeback => {
                let actual = tx_event.r#type;
                Err(TransactionError::from_event_error(
                    tx_event,
                    EventTxError::UnexpectedTxTypeError { actual },
                ))
            }
        }
    }
}

pub trait Disputable: Transactable {
    fn resolve(self: Box<Self>) -> Box<dyn Transactable>;
    fn chargeback(self: Box<Self>) -> Box<dyn Transactable>;
    fn cloned(&self) -> Box<dyn Disputable>;
}

pub trait Transactable: Send {
    fn apply(&self, account: AccountTx) -> Result<(), AccountTxError>;
    fn client(&self) -> u16;
    fn try_dispute(self: Box<Self>) -> Result<Box<dyn Disputable>, EventTxError> {
        Err(EventTxError::NotDisputableError)
    }
    fn id(&self) -> u32;
}

#[derive(Copy, Clone, Debug)]
pub struct Deposit {
    amount: f64,
}
#[derive(Copy, Clone, Debug)]
pub struct Withdrawal {
    amount: f64,
}
#[derive(Copy, Clone, Debug)]
pub struct Dispute {
    amount: f64,
}
#[derive(Copy, Clone, Debug)]
pub struct Resolve {
    amount: f64,
}
#[derive(Copy, Clone, Debug)]
pub struct Chargeback {
    amount: f64,
}

#[derive(Clone, Debug)]
pub struct TransactionState<T: Clone> {
    pub tx_event: TransactionEvent,
    pub transaction: T,
}

impl Transactable for TransactionState<Deposit> {
    fn apply(&self, mut account: AccountTx) -> Result<(), AccountTxError> {
        account.available(AccountOp::Add(self.transaction.amount));
        account.apply()?;
        Ok(())
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn try_dispute(self: Box<Self>) -> Result<Box<dyn Disputable>, EventTxError> {
        Ok(Box::new(TransactionState {
            tx_event: self.tx_event,
            transaction: Dispute {
                amount: self.transaction.amount,
            },
        }))
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}

impl TransactionState<Deposit> {
    pub fn new(tx_event: TransactionEvent) -> Result<Self, TransactionError> {
        if let Some(amount) = tx_event.amount {
            Ok(TransactionState {
                tx_event,
                transaction: Deposit { amount },
            })
        } else {
            Err(TransactionError::from_event_error(
                tx_event,
                EventTxError::ExpectedAmountError,
            ))
        }
    }
}

impl Transactable for TransactionState<Withdrawal> {
    fn apply(&self, mut account: AccountTx) -> Result<(), AccountTxError> {
        account.available(AccountOp::Sub(self.transaction.amount));
        account.apply()?;
        Ok(())
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}

impl TransactionState<Withdrawal> {
    pub fn new(tx_event: TransactionEvent) -> Result<Self, TransactionError> {
        if let Some(amount) = tx_event.amount {
            Ok(TransactionState {
                tx_event,
                transaction: Withdrawal { amount },
            })
        } else {
            Err(TransactionError::from_event_error(
                tx_event,
                EventTxError::ExpectedAmountError,
            ))
        }
    }
}

impl Disputable for TransactionState<Dispute> {
    fn resolve(self: Box<Self>) -> Box<dyn Transactable> {
        Box::new(TransactionState {
            tx_event: self.tx_event,
            transaction: Resolve {
                amount: self.transaction.amount,
            },
        })
    }

    fn chargeback(self: Box<Self>) -> Box<dyn Transactable> {
        Box::new(TransactionState {
            tx_event: self.tx_event,
            transaction: Chargeback {
                amount: self.transaction.amount,
            },
        })
    }
    fn cloned(&self) -> Box<dyn Disputable> {
        Box::new(self.clone())
    }
}

impl Transactable for TransactionState<Dispute> {
    fn apply(&self, mut account: AccountTx) -> Result<(), AccountTxError> {
        account.held(AccountOp::Add(self.transaction.amount));
        account.available(AccountOp::Sub(self.transaction.amount));
        account.apply()?;
        Ok(())
    }
    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}

impl Transactable for TransactionState<Resolve> {
    fn apply(&self, mut account: AccountTx) -> Result<(), AccountTxError> {
        account.available(AccountOp::Add(self.transaction.amount));
        account.held(AccountOp::Sub(self.transaction.amount));
        account.apply()?;
        Ok(())
    }

    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}

impl Transactable for TransactionState<Chargeback> {
    fn apply(&self, mut account: AccountTx) -> Result<(), AccountTxError> {
        account.lock();
        account.held(AccountOp::Sub(self.transaction.amount));
        account.apply()?;
        Ok(())
    }

    fn client(&self) -> u16 {
        self.tx_event.client
    }

    fn id(&self) -> u32 {
        self.tx_event.id
    }
}
