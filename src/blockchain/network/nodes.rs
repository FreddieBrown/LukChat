//! Node structure containing all information for application

use crate::blockchain::{
    config::Profile,
    network::{
        accounts::{Account, Role},
        JobSync,
    },
    Block, BlockChain, BlockChainBase, Event, UserPair,
};

use crate::blockchain::userpair::PersistentInformation;

use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

/// Struct to contain all information about [`Node`]
#[derive(Debug)]
pub struct Node<T: BlockChainBase> {
    pub account: Account,
    pub blockchain: RwLock<BlockChain<T>>,
    pub loose_events: RwLock<Vec<Event<T>>>,
}

impl<T: BlockChainBase> Node<T> {
    /// Creats a new [`Node`] instance
    pub fn new(role: Role, profile: Profile, persistent: PersistentInformation) -> Self {
        Self {
            account: Account::new(
                role,
                profile,
                persistent.pub_key,
                persistent.priv_key,
                persistent.id,
            ),
            blockchain: RwLock::new(BlockChain::new()),
            loose_events: RwLock::new(Vec::new()),
        }
    }

    /// Function to create a [`Blockchain`] with a genesis [`Block`]
    pub async fn genesis(
        profile: Profile,
        sync: &JobSync<T>,
        persistent: PersistentInformation,
    ) -> Result<Self> {
        let account = Account::new(
            Role::Miner,
            profile,
            persistent.pub_key,
            persistent.priv_key,
            persistent.id,
        );
        let mut initial_bc: BlockChain<T> = BlockChain::new();
        let genesis: Block<T> = Block::new(None);
        sync.write_block(genesis.clone()).await?;
        initial_bc.chain.push(genesis);
        Ok(Self {
            account,
            blockchain: RwLock::new(initial_bc),
            loose_events: RwLock::new(Vec::new()),
        })
    }

    /// Gets the length of the underlying [`Blockchain`]
    pub async fn bc_len(&self) -> usize {
        let unlocked = self.blockchain.read().await;
        unlocked.len()
    }

    /// Returns the overlap percentage between 2 [`BlockChain`] instances
    pub async fn chain_overlap(&self, chain: &BlockChain<T>) -> f64 {
        let unlocked = self.blockchain.read().await;
        unlocked.chain_overlap(chain)
    }

    /// Determines if a given [`Block`] is in the underlying [`BlockChain']
    pub async fn in_chain(&self, block: &Block<T>) -> bool {
        let unlocked = self.blockchain.read().await;
        unlocked.in_chain(block)
    }

    /// Adds a [`Block`] to the underlying [`BlockChain`]
    pub async fn add_block(&self, block: Block<T>, pair: Arc<UserPair<T>>) -> Result<()> {
        let mut unlocked = self.blockchain.write().await;
        unlocked.append(block, pair).await
    }
}
