# Transaction Processor

This is a transaction processor that consumes a CSV containing transactions on client accounts.

The head line of the CSV reader is treated as a Transaction Event, which is routed to a processing shard based on the transaction's client ID. The shard then either forwards simple Deposit or Withdrawal types to the Account Processor, or in the event of a Dispute, finds the previous transaction under dispute, caches the transaction until a Resolve or Chargeback event occurs.

### Usage
`cargo run -- path/to.csv > output.csv`


## Transaction Router
Reads raw transaction events (which are really just a line in a csv), and forwards the event to a shard based on the transaction's client ID

## Transaction Shard
- Looks up and caches transactions under Dispute (which is the tail in a CSV file), converts a cached disputed transaction into a Resolve or Chargeback, and forwards to the Account Processor.
- Forwards a Deposit or Withdrawal event to the Account Processor

## Account Processor
Holds all account data, and executes operations on accounts via Transactions it receives from one or more Transaction Shards.


