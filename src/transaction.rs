use crate::{account::Account, error::TransactionError};

pub enum TransactionEventType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

pub struct TransactionEvent {
    id: u32,
    r#type: TransactionEventType,
    client: u16,
    amount: Option<f64>,
}

trait Transactable {
    fn apply(&self, account: &mut Account) -> Result<(), TransactionError>;
}

struct Deposit {}
struct Withdrawal {}
struct Dispute {
    id: u32,
}
struct Resolve {
    id: u32,
}
struct Chargeback {
    id: u32,
}

pub struct TransactionState<T> {
    tx_event: TransactionEvent,
    transaction: T,
}

impl<T: Transactable> Transactable for TransactionState<T> {
    fn apply(&self, account: &mut Account) -> Result<(), TransactionError> {
        self.transaction.apply(account)
    }
}

impl TransactionState<Deposit> {
    pub fn new(tx_event: TransactionEvent) -> Self {
        TransactionState {
            tx_event,
            transaction: Deposit {},
        }
    }

    pub fn dispute(self, id: u32) -> TransactionState<Dispute> {
        TransactionState {
            tx_event: self.tx_event,
            transaction: Dispute { id },
        }
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

impl TransactionState<Dispute> {
    pub fn resolve(self, id: u32) -> TransactionState<Resolve> {
        TransactionState {
            tx_event: self.tx_event,
            transaction: Resolve { id },
        }
    }

    pub fn chargeback(self, id: u32) -> TransactionState<Chargeback> {
        TransactionState {
            tx_event: self.tx_event,
            transaction: Chargeback { id },
        }
    }
}
