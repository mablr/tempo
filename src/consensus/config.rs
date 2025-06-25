//! Configuration types for the Malachite consensus engine

use malachitebft_app::{
    config::{
        ConsensusConfig as MalachiteConsensusConfig, DiscoveryConfig, LoggingConfig, MetricsConfig,
        P2pConfig, PubSubProtocol, RuntimeConfig, TimeoutConfig, ValuePayload, ValueSyncConfig,
    },
    node::NodeConfig as MalachiteNodeConfig,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, str::FromStr};

/// Main configuration for the consensus node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node moniker (human-readable name)
    pub moniker: String,
    /// Consensus configuration
    pub consensus: MalachiteConsensusConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Runtime configuration
    pub runtime: RuntimeConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Value synchronization configuration
    pub value_sync: ValueSyncConfig,
}

impl NodeConfig {
    /// Create a new node configuration with default values
    pub fn new(moniker: String, listen_addr: String, peers: Vec<String>) -> Self {
        let listen_addr = multiaddr::Multiaddr::from_str(&listen_addr)
            .unwrap_or_else(|_| "/ip4/127.0.0.1/tcp/26656".parse().unwrap());

        let persistent_peers = peers
            .into_iter()
            .filter_map(|p| multiaddr::Multiaddr::from_str(&p).ok())
            .collect();

        Self {
            moniker,
            consensus: MalachiteConsensusConfig {
                value_payload: ValuePayload::ProposalAndParts,
                timeouts: TimeoutConfig::default(),
                p2p: P2pConfig {
                    protocol: PubSubProtocol::default(),
                    listen_addr,
                    persistent_peers,
                    discovery: DiscoveryConfig {
                        enabled: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            metrics: MetricsConfig {
                enabled: true,
                listen_addr: "127.0.0.1:9000".parse().unwrap(),
            },
            runtime: RuntimeConfig::default(),
            logging: LoggingConfig::default(),
            value_sync: ValueSyncConfig::default(),
        }
    }
}

impl MalachiteNodeConfig for NodeConfig {
    fn moniker(&self) -> &str {
        &self.moniker
    }

    fn consensus(&self) -> &MalachiteConsensusConfig {
        &self.consensus
    }

    fn value_sync(&self) -> &ValueSyncConfig {
        &self.value_sync
    }
}

/// WAL (Write-Ahead Log) configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalConfig {
    /// Directory path for WAL storage
    pub path: PathBuf,
    /// Maximum size per WAL file
    pub max_file_size: u64,
    /// Whether to retain all WAL files
    pub retain_all: bool,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./wal"),
            max_file_size: 100 * 1024 * 1024, // 100MB
            retain_all: false,
        }
    }
}

/// Network configuration specific to reth-malachite
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Chain ID
    pub chain_id: String,
    /// Network listen address
    pub listen_addr: SocketAddr,
    /// List of peer addresses
    pub peers: Vec<String>,
    /// Discovery settings
    pub discovery_enabled: bool,
}

impl NetworkConfig {
    pub fn new(chain_id: String, listen_addr: SocketAddr) -> Self {
        Self {
            chain_id,
            listen_addr,
            peers: Vec::new(),
            discovery_enabled: false,
        }
    }

    pub fn with_peers(mut self, peers: Vec<String>) -> Self {
        self.peers = peers;
        self
    }

    pub fn with_discovery(mut self, enabled: bool) -> Self {
        self.discovery_enabled = enabled;
        self
    }
}

/// Engine configuration combining all settings
#[derive(Clone, Debug)]
pub struct EngineConfig {
    /// Node configuration
    pub node: NodeConfig,
    /// WAL configuration
    pub wal: WalConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Height to start from (if resuming)
    pub start_height: Option<crate::Height>,
}

impl EngineConfig {
    /// Create a new engine configuration
    pub fn new(chain_id: String, moniker: String, listen_addr: SocketAddr) -> Self {
        let listen_str = format!("/ip4/{}/tcp/{}", listen_addr.ip(), listen_addr.port());
        let network = NetworkConfig::new(chain_id, listen_addr);
        let node = NodeConfig::new(moniker, listen_str, Vec::new());

        Self {
            node,
            wal: WalConfig::default(),
            network,
            start_height: None,
        }
    }

    /// Set the WAL directory
    pub fn with_wal_dir(mut self, path: PathBuf) -> Self {
        self.wal.path = path;
        self
    }

    /// Set the starting height
    pub fn with_start_height(mut self, height: crate::Height) -> Self {
        self.start_height = Some(height);
        self
    }

    /// Add peer addresses
    pub fn with_peers(mut self, peers: Vec<String>) -> Self {
        self.network.peers = peers.clone();
        self.node.consensus.p2p.persistent_peers = peers
            .into_iter()
            .filter_map(|p| multiaddr::Multiaddr::from_str(&p).ok())
            .collect();
        self
    }
}
