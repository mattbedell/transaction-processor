use std::{error::Error, fs::File, path::Path};

use csv::{Reader, ReaderBuilder};
use tokio::sync::mpsc::Sender;

use crate::transaction::TransactionEvent;

pub fn reader<P: AsRef<Path>>(path: P) -> Result<Reader<File>, Box<dyn Error>> {
    Ok(ReaderBuilder::new().has_headers(true).from_path(path)?)
}

pub struct TransactionRouter<'a> {
    shards: Vec<Sender<TransactionEvent>>,
    path: &'a Path,
}

impl<'a> TransactionRouter<'a> {
    pub fn new(shards: Vec<Sender<TransactionEvent>>, path: &'a Path) -> Self {
        TransactionRouter { shards, path }
    }
    pub async fn route_tx(&self, event: TransactionEvent) {
        let shard_id = (event.client as usize) % self.shards.len();
        let _ = self.shards[shard_id].send(event).await.inspect_err(|err| eprintln!("{err}"));
    }

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let mut reader = reader(self.path)?;
        for result in reader.deserialize() {
            let event: TransactionEvent = result?;
            self.route_tx(event).await;
        }
        Ok(())
    }
}
