//! Store wrapper for easier integration with the State module.

use super::RethStore;
use crate::{context::MalachiteContext, height::Height, Value, ValueId};
use eyre::Result;
use malachitebft_app_channel::app::types::ProposedValue;
use malachitebft_core_types::{CommitCertificate, Round};
use reth_provider::DatabaseProviderFactory;
use std::sync::Arc;

/// A wrapper around RethStore that hides the generic parameter
#[derive(Clone)]
pub struct Store {
    inner: Arc<dyn StoreOps + Send + Sync>,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").finish()
    }
}

impl Store {
    /// Create a new Store from any provider that implements DatabaseProviderFactory
    pub fn new<P>(provider: Arc<P>) -> Self
    where
        P: DatabaseProviderFactory + Send + Sync + 'static,
        P::Provider: Send,
        P::ProviderRW: Send,
    {
        Self {
            inner: Arc::new(RethStore::new(provider)),
        }
    }

    /// Returns the maximum decided value height
    pub async fn max_decided_value_height(&self) -> Option<Height> {
        self.inner.max_decided_value_height().await
    }

    /// Get a decided value by height
    pub async fn get_decided_value(&self, height: Height) -> Result<Option<super::DecidedValue>> {
        self.inner.get_decided_value(height).await
    }

    /// Store a decided value with its certificate
    pub async fn store_decided_value(
        &self,
        certificate: &CommitCertificate<MalachiteContext>,
        value: Value,
    ) -> Result<()> {
        self.inner.store_decided_value(certificate, value).await
    }

    /// Get undecided proposals for a height and round
    pub async fn get_undecided_proposals(
        &self,
        height: Height,
        round: Round,
    ) -> Result<Vec<ProposedValue<MalachiteContext>>> {
        self.inner.get_undecided_proposals(height, round).await
    }

    /// Store an undecided proposal
    pub async fn store_undecided_proposal(
        &self,
        proposal: ProposedValue<MalachiteContext>,
    ) -> Result<()> {
        self.inner.store_undecided_proposal(proposal).await
    }

    /// Get an undecided proposal by height, round, and value ID
    pub async fn get_undecided_proposal(
        &self,
        height: Height,
        round: Round,
        value_id: ValueId,
    ) -> Result<Option<ProposedValue<MalachiteContext>>> {
        self.inner
            .get_undecided_proposal(height, round, value_id)
            .await
    }

    /// Verify that all consensus tables exist in the database
    pub async fn verify_tables(&self) -> Result<()> {
        self.inner.verify_tables().await
    }
}

/// Internal trait to hide the generic parameter
#[async_trait::async_trait]
trait StoreOps {
    async fn max_decided_value_height(&self) -> Option<Height>;
    async fn get_decided_value(&self, height: Height) -> Result<Option<super::DecidedValue>>;
    async fn store_decided_value(
        &self,
        certificate: &CommitCertificate<MalachiteContext>,
        value: Value,
    ) -> Result<()>;
    async fn get_undecided_proposals(
        &self,
        height: Height,
        round: Round,
    ) -> Result<Vec<ProposedValue<MalachiteContext>>>;
    async fn store_undecided_proposal(
        &self,
        proposal: ProposedValue<MalachiteContext>,
    ) -> Result<()>;
    async fn get_undecided_proposal(
        &self,
        height: Height,
        round: Round,
        value_id: ValueId,
    ) -> Result<Option<ProposedValue<MalachiteContext>>>;
    async fn verify_tables(&self) -> Result<()>;
}

#[async_trait::async_trait]
impl<P> StoreOps for RethStore<P>
where
    P: DatabaseProviderFactory + Send + Sync,
    P::Provider: Send,
    P::ProviderRW: Send,
{
    async fn max_decided_value_height(&self) -> Option<Height> {
        self.max_decided_value_height().await
    }

    async fn get_decided_value(&self, height: Height) -> Result<Option<super::DecidedValue>> {
        self.get_decided_value(height).await.map_err(Into::into)
    }

    async fn store_decided_value(
        &self,
        certificate: &CommitCertificate<MalachiteContext>,
        value: Value,
    ) -> Result<()> {
        self.store_decided_value(certificate, value)
            .await
            .map_err(Into::into)
    }

    async fn get_undecided_proposals(
        &self,
        height: Height,
        round: Round,
    ) -> Result<Vec<ProposedValue<MalachiteContext>>> {
        self.get_undecided_proposals(height, round)
            .await
            .map_err(Into::into)
    }

    async fn store_undecided_proposal(
        &self,
        proposal: ProposedValue<MalachiteContext>,
    ) -> Result<()> {
        self.store_undecided_proposal(proposal)
            .await
            .map_err(Into::into)
    }

    async fn get_undecided_proposal(
        &self,
        height: Height,
        round: Round,
        value_id: ValueId,
    ) -> Result<Option<ProposedValue<MalachiteContext>>> {
        self.get_undecided_proposal(height, round, value_id)
            .await
            .map_err(Into::into)
    }

    async fn verify_tables(&self) -> Result<()> {
        self.verify_tables().await.map_err(Into::into)
    }
}
