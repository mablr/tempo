//! Configuration loader for Malachite consensus engine.
//! 
//! This module provides utilities for loading and parsing consensus configuration
//! from TOML files. It handles the extraction of P2P settings, metrics configuration,
//! and node-specific parameters required to run the Malachite consensus engine.
//! 
//! # Configuration Format
//! 
//! The configuration file should contain:
//! - `consensus.p2p.listen_addr`: The address to listen for P2P connections
//! - `consensus.p2p.persistent_peers`: Comma-separated list of peer addresses
//! - `metrics.listen_addr`: Optional metrics server address

use crate::consensus::config::{EngineConfig, NodeConfig};
use eyre::Result;
use std::{fs, path::Path};
use toml;

/// Load engine configuration from a TOML file
pub fn load_engine_config(
    config_path: &Path,
    chain_id: String,
    node_id: String,
) -> Result<EngineConfig> {
    // Read the TOML file
    tracing::info!("Reading config file from: {:?}", config_path);
    let config_str = fs::read_to_string(config_path)?;
    tracing::info!("Config file size: {} bytes", config_str.len());
    let config_value: toml::Value = toml::from_str(&config_str)?;
    
    // Extract consensus settings
    let consensus = config_value.get("consensus").ok_or_else(|| {
        eyre::eyre!("Missing 'consensus' section in config")
    })?;
    
    // Extract P2P settings
    let p2p = consensus.get("p2p").ok_or_else(|| {
        eyre::eyre!("Missing 'consensus.p2p' section in config")
    })?;
    
    // Parse listen address
    let listen_addr = p2p.get("listen_addr")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre::eyre!("Missing 'consensus.p2p.listen_addr'"))?;
    
    // Parse persistent peers
    let peers_str = p2p.get("persistent_peers")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let peers: Vec<String> = if peers_str.is_empty() {
        Vec::new()
    } else {
        peers_str.split(',').map(|s| s.trim().to_string()).collect()
    };
    
    // Extract metrics settings
    let metrics_port = config_value
        .get("metrics")
        .and_then(|m| m.get("listen_addr"))
        .and_then(|v| v.as_str())
        .and_then(|addr| addr.split(':').last())
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(9000);
    
    // Extract the consensus port from listen_addr
    let consensus_port = listen_addr
        .split('/')
        .last()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(26656);
    
    // Create socket address
    let socket_addr = format!("127.0.0.1:{}", consensus_port).parse()?;
    
    // Create node config with the loaded settings
    let mut node_config = NodeConfig::new(node_id.clone(), listen_addr.to_string(), peers.clone());
    
    // Update metrics port
    node_config.metrics.listen_addr = format!("0.0.0.0:{}", metrics_port).parse()?;
    
    // Create engine config
    let mut engine_config = EngineConfig::new(chain_id, node_id, socket_addr);
    engine_config.node = node_config;
    engine_config.network = engine_config.network.with_peers(peers);
    
    Ok(engine_config)
}