# ATP Chain Registry
A comprehensive, type-safe chain registry for multi-chain applications, designed for ATP and cross-chain arbitrage systems. Built with Rust and following CAIP (Chain Agnostic Improvement Proposals) standards.

## 🎯 Overview

The Chain Registry provides a unified interface to manage blockchain networks, assets, and trading pairs across multiple chains. It enables developers to build chain-agnostic applications that can seamlessly work with any blockchain ecosystem.

### Key Features

- **🔗 Multi-Chain Support** - Ethereum, Solana, and extensible to any blockchain
- **🎨 Configuration Agnostic** - Works with any chain/asset configuration
- **💱 Trading Pair Management** - Cross-chain and same-chain trading routes
- **🔐 Cryptographic Curve Support** - secp256k1, ed25519, and more
- **⚡ High Performance** - Fast lookups and route discovery
- **🏥 Health Monitoring** - Built-in configuration validation
- **📊 Rich Metadata** - Extensive chain and asset information
- **🧪 Test Network Support** - Clean separation of mainnet/testnet

## 🚀 Quick Start

### Basic Usage

```rust
use atp_chain_registry::{ChainRegistry, ChainId, AssetId};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the default configuration
    let registry = ChainRegistry::default()?;
    
    // Get chain ID from name
    let chain_id = registry.get_chain_id_from_config("Ethereum Mainnet")?;
    println!("Ethereum Mainnet ID: {}", chain_id);
    
    // Get asset ID from symbol and chain
    let asset_id = registry.get_asset_id_from_config("ETH", "eip155:1")?;
    println!("ETH on Ethereum: {}", asset_id);
    
    // Find cross-chain trading opportunities
    let cross_chain_pairs = registry.get_cross_chain_pairs();
    println!("Found {} cross-chain trading pairs", cross_chain_pairs.len());
    
    // Discover trading routes
    let eth_asset = AssetId::new(ChainId::from_str("eip155:1")?, "slip44", "60")?;
    let sol_asset = AssetId::new(ChainId::from_str("solana:mainnet")?, "slip44", "501")?;
    let routes = registry.find_trading_routes(&eth_asset, &sol_asset, 3);
    println!("Found {} routes from ETH to SOL", routes.len());
    
    Ok(())
}
```

## 📋 Core Features

### Requested Features

The Chain Registry implements two key features for chain-agnostic applications:

#### 1. Get ChainId from Chain Configuration
```rust
let chain_id = registry.get_chain_id_from_config("Ethereum Mainnet")?;
// Returns: ChainId("eip155:1")
```

#### 2. Get AssetId from Symbol and Chain
```rust
let asset_id = registry.get_asset_id_from_config("ETH", "eip155:1")?;
// Returns: AssetId("eip155:1/slip44:60")
```

### Advanced Features

- **Cross-Chain Route Discovery** - Find optimal trading paths between any assets
- **Network Separation** - Clean isolation between mainnet and testnet environments
- **Health Monitoring** - Validate configuration integrity and detect issues
- **Performance Analytics** - Built-in performance metrics and benchmarking
- **Dynamic Configuration** - Add/remove chains and assets at runtime

## 🏗️ Architecture

The Chain Registry follows a clean, modular architecture:

```
├── types.rs          # Core data structures (ChainConfig, AssetConfig, TokenPair)
├── registry.rs       # Main registry implementation and CRUD operations  
├── query.rs          # Advanced query operations and analytics
├── error.rs          # Comprehensive error handling
└── lib.rs           # Public API and integration
```

### Key Components

- **`ChainRegistry`** - Main registry interface for managing chains, assets, and pairs
- **`ChainConfig`** - Blockchain configuration (RPC endpoints, metadata, assets)
- **`AssetConfig`** - Asset configuration (symbol, decimals, metadata)
- **`TokenPair`** - Trading pair configuration (fees, limits, routing)

## 🌐 Supported Networks

### Default Configuration

The registry comes with a default configuration supporting:

- **Ethereum Mainnet** (`eip155:1`) - ETH native token
- **Ethereum Sepolia** (`eip155:11155111`) - ETH testnet
- **Solana Mainnet** (`solana:mainnet`) - SOL native token  
- **Solana Devnet** (`solana:devnet`) - SOL testnet

### Trading Pairs

- **Mainnet**: ETH ↔ SOL (0.3% fee)
- **Testnet**: ETH ↔ SOL (0.5% fee)
- **Direct routes only** - No complex multi-hop routing needed

### Extending Support

The registry is designed to be easily extensible. Add support for new chains by:

1. Creating a `ChainConfig` with chain details
2. Defining `AssetConfig` for each asset on the chain
3. Setting up `TokenPair` configurations for trading
4. Adding to registry with `add_chain()`, `add_asset()`, `add_token_pair()`

## 🔧 Configuration

### Programmatic Configuration
```rust
use atp_chain_registry::*;

let mut registry = ChainRegistry::new();

// Add a new chain
let chain_config = ChainConfig {
    chain_id: "polygon:mainnet".to_string(),
    name: "Polygon Mainnet".to_string(),
    native_asset: "slip44:966".to_string(),
    rpc_endpoints: vec!["https://polygon-rpc.com".to_string()],
    explorer_url: Some("https://polygonscan.com".to_string()),
    cryptographic_curve: vec![Curve::Secp256k1],
    is_testnet: false,
    assets: vec![AssetIdBase::new("slip44", "966")?],
    metadata: HashMap::new(),
};
registry.add_chain(chain_config)?;
```

### TOML Configuration

```toml
[chains."polygon:mainnet"]
chain_id = "polygon:mainnet"
name = "Polygon Mainnet"
native_asset = "slip44:966"
rpc_endpoints = ["https://polygon-rpc.com"]
explorer_url = "https://polygonscan.com"
cryptographic_curve = ["secp256k1"]
is_testnet = false
assets = [
    { asset_namespace = "slip44", asset_reference = "966" }
]
```

### Loading from File

```rust
let registry = ChainRegistry::from_file("config.toml")?;
```

## 🚀 Examples

### Run the Configuration Summary Demo

```bash
cargo run --example configuration_summary
```

This example demonstrates:
- Dynamic discovery of all configured chains and assets
- Cross-chain trading pair analysis
- Network separation (mainnet vs testnet)
- Trading route discovery
- Performance benchmarking
- Health checks and validation

Expected output:
```
🚀 Chain Registry - Configuration Agnostic Demo

📊 Registry Overview:
   Total chains: 4
   ├─ Mainnet: 2
   └─ Testnet: 2
   Total assets: 2
   Trading pairs: 4
   ├─ Cross-chain: 4
   └─ Same-chain: 0

🎯 Testing Requested Features:
   ✓ Ethereum Mainnet -> eip155:1
   ✓ Ethereum Sepolia -> eip155:11155111
   ✓ Solana Mainnet -> solana:mainnet
   ✓ Solana Devnet -> solana:devnet
```

### More Examples

- `examples/configuration_summary.rs` - Show configuration summary

## 🔍 Testing

Run the full test suite:

```bash
cargo test
```

Run specific test categories:

```bash
# Test core functionality
cargo test registry

# Test configuration loading
cargo test config

# Test with output
cargo test -- --nocapture
```

## 📊 Analytics

### Registry Statistics

```rust
let stats = registry.get_statistics();
println!("Chains: {}, Assets: {}, Pairs: {}", 
         stats.total_chains, stats.total_assets, stats.total_pairs);
```


## 🔒 Security

### Validation

All configurations are validated on load:
- CAIP-2 compliant chain IDs
- CAIP-19 compliant asset IDs  
- Valid asset references
- Consistent metadata

### Error Handling

Comprehensive error types for robust applications:
```rust
match registry.get_chain(&chain_id) {
    Ok(chain) => { /* use chain */ },
    Err(ChainRegistryError::ChainNotFound(id)) => {
        println!("Chain {} not found", id);
    },
    Err(e) => println!("Error: {}", e),
}
```

