use std::{collections::HashMap, path::Path};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    router::{self},
    transaction::{
        Deposit, Dispute, Transactable, TransactionEvent, TransactionEventType, TransactionState,
        Withdrawal,
    },
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
                    TransactionEventType::Deposit => {
                        let tx = Box::new(TransactionState::<Deposit>::new(tx_event));
                        self.account_tx.send(tx).await.unwrap()
                    }
                    TransactionEventType::Withdrawal => {
                        let tx = Box::new(TransactionState::<Withdrawal>::new(tx_event));
                        let _ = self.account_tx.send(tx).await;
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
                                let dispute = TransactionState::<Dispute>::new(event_record);
                                if let Ok(dispute_tx) = dispute {
                                    let tx = Box::new(dispute_tx);
                                    pending.insert(tx.tx_event.id, tx.clone());
                                    self.account_tx.send(tx).await.unwrap();
                                }
                                break;
                            }
                        }
                    }
                    TransactionEventType::Resolve => {
                        if let Some(tx) = pending.remove(&tx_event.id) {
                            let tx = tx.resolve();
                            self.account_tx.send(Box::new(tx)).await.unwrap();
                        }
                    }
                    TransactionEventType::Chargeback => {
                        if let Some(tx) = pending.remove(&tx_event.id) {
                            let tx = tx.chargeback();
                            self.account_tx.send(Box::new(tx)).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
