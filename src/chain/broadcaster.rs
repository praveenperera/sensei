use std::sync::Arc;

use bitcoin::Transaction;
use lightning::chain::chaininterface::BroadcasterInterface;

use super::listener_database::ListenerDatabase;

pub struct SenseiBroadcaster {
    pub broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>,
    pub listener_database: ListenerDatabase,
}

impl BroadcasterInterface for SenseiBroadcaster {
    fn broadcast_transaction(&self, tx: &Transaction) {
        self.broadcaster.broadcast_transaction(tx);

        // TODO: there's a bug here if the broadcast fails
        //       best solution is to probably setup a zmq listener
        self.listener_database.process_mempool_tx(tx);
    }
}
