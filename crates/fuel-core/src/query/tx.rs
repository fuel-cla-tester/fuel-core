use crate::graphql_api::ports::DatabasePort;
use fuel_core_storage::{
    iter::{
        BoxedIter,
        IntoBoxedIter,
        IterDirection,
    },
    not_found,
    tables::{
        Receipts,
        Transactions,
    },
    Result as StorageResult,
    StorageAsRef,
};
use fuel_core_txpool::types::TxId;
use fuel_core_types::{
    fuel_tx::{
        Receipt,
        Transaction,
        TxPointer,
    },
    fuel_types::Address,
    services::txpool::TransactionStatus,
};

pub trait SimpleTransactionData: Send + Sync {
    /// Return all receipts in the given transaction.
    fn receipts(&self, transaction_id: &TxId) -> StorageResult<Vec<Receipt>>;

    /// Get the transaction.
    fn transaction(&self, transaction_id: &TxId) -> StorageResult<Transaction>;
}

impl<D: DatabasePort + ?Sized> SimpleTransactionData for D {
    fn transaction(&self, tx_id: &TxId) -> StorageResult<Transaction> {
        self.storage::<Transactions>()
            .get(tx_id)
            .and_then(|v| v.ok_or(not_found!(Transactions)))
    }

    fn receipts(&self, tx_id: &TxId) -> StorageResult<Vec<Receipt>> {
        self.storage::<Receipts>()
            .get(tx_id)
            .and_then(|v| v.ok_or(not_found!(Transactions)))
    }
}

pub trait TransactionQueryData: Send + Sync + SimpleTransactionData {
    fn status(&self, tx_id: &TxId) -> StorageResult<TransactionStatus>;

    fn owned_transactions(
        &self,
        owner: Address,
        start: Option<TxPointer>,
        direction: IterDirection,
    ) -> BoxedIter<StorageResult<(TxPointer, Transaction)>>;
}

impl<D: DatabasePort + ?Sized> TransactionQueryData for D {
    fn status(&self, tx_id: &TxId) -> StorageResult<TransactionStatus> {
        self.tx_status(tx_id)
    }

    fn owned_transactions(
        &self,
        owner: Address,
        start: Option<TxPointer>,
        direction: IterDirection,
    ) -> BoxedIter<StorageResult<(TxPointer, Transaction)>> {
        self.owned_transactions_ids(owner, start, direction)
            .map(|result| {
                result.and_then(|(tx_pointer, tx_id)| {
                    let tx = self.transaction(&tx_id)?;

                    Ok((tx_pointer, tx))
                })
            })
            .into_boxed()
    }
}
