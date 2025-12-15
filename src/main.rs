use std::{error::Error, path::Path};

use account::AccountProcessor;
use shard::TransactionShard;

mod account;
mod error;
mod router;
mod shard;
mod transaction;

const NUM_SHARDS: u8 = 4;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).unwrap();
    let path = Path::new(path);

    let account_processor = AccountProcessor::new();
    let shards: Vec<_> = (0..NUM_SHARDS)
        .map(|_| TransactionShard::new(account_processor.tx.clone(), path))
        .collect();
    let shards_tx: Vec<_> = shards.iter().map(|shard| shard.tx.clone()).collect();
    let router = router::TransactionRouter::new(shards_tx, path);

    for shard in shards {
        shard.run();
    }

    let account_processor_handle = account_processor.run();
    router.run().await?;

    let accounts = account_processor_handle.await?;

    println!("{accounts}");

    Ok(())
}
