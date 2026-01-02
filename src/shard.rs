use std::{collections::HashMap, path::Path};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    router::{self},
    transaction::{Transactable, TransactionEvent, TransactionEventType},
};

pub struct TransactionShard<'a> {
    path: &'a Path,
    account_tx: Sender<Box<dyn Transactable>>,
    rx: Receiver<TransactionEvent>,
    pub tx: Sender<TransactionEvent>,
}

impl<'a> TransactionShard<'a> {
    pub fn new(account_processor_tx: Sender<Box<dyn Transactable>>, path: &'a Path) -> Self {
        let (tx, rx) = mpsc::channel(10_000);
        TransactionShard {
            path,
            account_tx: account_processor_tx,
            rx,
            tx,
        }
    }

    pub fn run(mut self) {
        let path = self.path.to_owned();
        tokio::task::spawn(async move {
            let mut pending = HashMap::new();
            while let Some(tx_event) = self.rx.recv().await {
                match tx_event.r#type {
                    TransactionEventType::Deposit | TransactionEventType::Withdrawal => {
                        if let Ok(tx) = Box::<dyn Transactable>::try_from(tx_event) {
                            self.account_tx.send(tx).await.unwrap();
                        }
                    }
                    TransactionEventType::Dispute => {
                        let tx_id = tx_event.id;
                        if pending.contains_key(&tx_id) {
                            continue;
                        }
                        let mut reader = router::reader(&path).unwrap();
                        for result in reader.deserialize() {
                            let event_record: TransactionEvent = result.unwrap();
                            if event_record.id == tx_id {
                                if let Ok(tx) = Box::<dyn Transactable>::try_from(event_record)
                                    && let Ok(tx) = tx.try_dispute()
                                {
                                    pending.insert(tx.id(), tx.cloned());
                                    self.account_tx.send(tx).await.unwrap();
                                }
                                break;
                            }
                        }
                    }
                    TransactionEventType::Resolve => {
                        if let Some(tx) = pending.remove(&tx_event.id) {
                            let tx = tx.resolve();
                            self.account_tx.send(tx).await.unwrap();
                        }
                    }
                    TransactionEventType::Chargeback => {
                        if let Some(tx) = pending.remove(&tx_event.id) {
                            let tx = tx.chargeback();
                            self.account_tx.send(tx).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
