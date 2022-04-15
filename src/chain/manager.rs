use std::{sync::Arc, time::Duration};

use crate::{
    config::SenseiConfig,
    node::{ChainMonitor, ChannelManager},
};
use bitcoin::BlockHash;
use lightning::chain::{BestBlock, Listen, chaininterface::{FeeEstimator, BroadcasterInterface}};
use lightning_block_sync::{poll::ValidatedBlockHeader, BlockSource};
use lightning_block_sync::SpvClient;
use lightning_block_sync::{init, poll, UnboundedCache};

use super::{listener::SenseiChainListener, listener_database::ListenerDatabase };

pub struct SenseiChainManager {
    config: SenseiConfig,
    pub listener: Arc<SenseiChainListener>,
    pub block_source: Arc<dyn BlockSource + Send + Sync>,
    pub fee_estimator: Arc<dyn FeeEstimator + Send + Sync>,
    pub broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>
}

impl SenseiChainManager {
    pub async fn new(
        config: SenseiConfig, 
        block_source: Arc<dyn BlockSource + Send + Sync>,
        fee_estimator: Arc<dyn FeeEstimator + Send + Sync>,
        broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>
    ) -> Result<Self, crate::error::Error> {
        let listener = Arc::new(SenseiChainListener::new());    
        let block_source_poller = block_source.clone();
        let listener_poller = listener.clone();
        tokio::spawn(async move {
            let mut cache = UnboundedCache::new();
            let chain_tip = init::validate_best_block_header(block_source_poller.clone()).await.unwrap();
            let chain_poller = poll::ChainPoller::new(block_source_poller, config.network);
            let mut spv_client =
                SpvClient::new(chain_tip, chain_poller, &mut cache, listener_poller);
            loop {
                spv_client.poll_best_tip().await.unwrap();
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        Ok(Self {
            config,
            listener,
            block_source,
            fee_estimator,
            broadcaster
        })
    }

    pub async fn synchronize_to_tip(
        &self,
        chain_listeners: Vec<(BlockHash, &(dyn Listen + Send + Sync))>,
    ) -> Result<ValidatedBlockHeader, crate::error::Error> {
        let chain_tip = init::synchronize_listeners(
            self.block_source.clone(),
            self.config.network,
            &mut UnboundedCache::new(),
            chain_listeners,
        )
        .await
        .unwrap();

        Ok(chain_tip)
    }

    pub async fn keep_in_sync(
        &self,
        channel_manager: Arc<ChannelManager>,
        chain_monitor: Arc<ChainMonitor>,
        listener_database: ListenerDatabase,
    ) -> Result<(), crate::error::Error> {
        let chain_listener = (chain_monitor, channel_manager, listener_database);
        self.listener.add_listener(chain_listener);
        Ok(())
    }

    pub async fn get_best_block(&self) -> Result<BestBlock, crate::error::Error> {
        let (latest_blockhash, latest_height) = self.block_source.get_best_block().await.unwrap();
        Ok(BestBlock::new(
            latest_blockhash,
            latest_height.unwrap(),
        ))
    }
}
