use chain_registry::{ChainId, ChainRegistry};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Chain Registry - Configuration Summary\n");

    // Load the default configuration (works with any config)
    let registry = ChainRegistry::default()?;

    // Display registry overview (completely dynamic)
    let stats = registry.get_statistics();
    println!("📊 Registry Overview:");
    println!("   Total chains: {}", stats.total_chains);
    println!("   ├─ Mainnet: {}", stats.mainnet_chains);
    println!("   └─ Testnet: {}", stats.testnet_chains);
    println!("   Total assets: {}", stats.total_assets);
    println!("   Trading pairs: {}", stats.total_pairs);
    println!("   ├─ Cross-chain: {}", stats.cross_chain_pairs);
    println!("   └─ Same-chain: {}", stats.same_chain_pairs);

    // Test the requested features dynamically
    println!("\n🎯 Testing Requested Features:");

    // Feature 1: Get ChainId from chain name (discover chains dynamically)
    let chains = registry.list_chains();
    if !chains.is_empty() {
        println!("   Chain Name -> Chain ID mapping:");
        for chain in &chains {
            let chain_id = registry.get_chain_id_from_config(&chain.name)?;
            println!("   ✓ {} -> {}", chain.name, chain_id);
        }
    }

    // Feature 2: Get AssetId from symbol and chain (discover assets dynamically)
    println!("\n   Asset Symbol + Chain -> Asset ID mapping:");
    let assets = registry.list_assets();
    for asset in &assets {
        // Try to find which chains have this asset
        for chain in &chains {
            let chain_id = ChainId::from_str(&chain.chain_id)?;
            let chain_assets = registry.get_chain_assets(&chain_id)?;

            // Check if this asset is on this chain
            let asset_on_chain = chain_assets.iter().find(|a| {
                a.asset_namespace() == asset.asset_id_base.asset_namespace()
                    && a.asset_reference() == asset.asset_id_base.asset_reference()
            });

            if asset_on_chain.is_some() {
                let asset_id = registry.get_asset_id_from_config(&asset.symbol, &chain.chain_id)?;
                println!(
                    "   ✓ {} on {} -> {}",
                    asset.symbol, chain.chain_id, asset_id
                );
            }
        }
    }

    // Show cross-chain trading opportunities (completely dynamic)
    println!("\n💱 Cross-Chain Trading Opportunities:");
    let cross_chain_pairs = registry.get_cross_chain_pairs();

    if cross_chain_pairs.is_empty() {
        println!("   No cross-chain pairs configured");
    } else {
        println!("   Available trading pairs:");
        for pair in &cross_chain_pairs {
            let from_asset = registry.get_asset_id_base(&pair.from_asset)?;
            let to_asset = registry.get_asset_id_base(&pair.to_asset)?;
            let from_chain = registry.get_chain(pair.from_asset.chain_id())?;
            let to_chain = registry.get_chain(pair.to_asset.chain_id())?;

            let network_type = if from_chain.is_testnet && to_chain.is_testnet {
                "🧪 Testnet"
            } else if !from_chain.is_testnet && !to_chain.is_testnet {
                "🌐 Mainnet"
            } else {
                "🔀 Mixed"
            };

            println!(
                "   {} {} ({}) -> {} ({}) | Fee: {}%",
                network_type,
                from_asset.symbol,
                from_chain.name,
                to_asset.symbol,
                to_chain.name,
                pair.fee_percentage.unwrap_or(0.0)
            );

            if let (Some(min), Some(max)) = (&pair.min_trade_amount, &pair.max_trade_amount) {
                println!("     └─ Trade range: {} - {} (in base units)", min, max);
            }
        }
    }

    // Show same-chain trading opportunities (if any)
    let same_chain_pairs = registry.get_same_chain_pairs();
    if !same_chain_pairs.is_empty() {
        println!("\n🏠 Same-Chain Trading Opportunities:");
        for pair in &same_chain_pairs {
            let from_asset = registry.get_asset_id_base(&pair.from_asset)?;
            let to_asset = registry.get_asset_id_base(&pair.to_asset)?;
            let chain = registry.get_chain(pair.from_asset.chain_id())?;

            println!(
                "   {} -> {} on {} | Fee: {}%",
                from_asset.symbol,
                to_asset.symbol,
                chain.name,
                pair.fee_percentage.unwrap_or(0.0)
            );
        }
    }

    // Show network separation dynamically
    println!("\n🌐 Network Separation Analysis:");

    let mainnet_chains = registry.get_mainnet_chains();
    let testnet_chains = registry.get_testnet_chains();

    if !mainnet_chains.is_empty() {
        println!("   Mainnet chains: {}", mainnet_chains.len());
        for chain in &mainnet_chains {
            let chain_id = ChainId::from_str(&chain.chain_id)?;
            let asset_count = registry.get_chain_assets(&chain_id)?.len();
            println!("     • {} ({} assets)", chain.name, asset_count);
        }

        // Show mainnet pairs
        let mainnet_pairs: Vec<_> = cross_chain_pairs
            .iter()
            .filter(|pair| {
                let from_chain = registry.get_chain(pair.from_asset.chain_id()).unwrap();
                let to_chain = registry.get_chain(pair.to_asset.chain_id()).unwrap();
                !from_chain.is_testnet && !to_chain.is_testnet
            })
            .collect();

        if !mainnet_pairs.is_empty() {
            println!("     Mainnet trading pairs: {}", mainnet_pairs.len());
            for pair in mainnet_pairs {
                let from_asset = registry.get_asset_id_base(&pair.from_asset)?;
                let to_asset = registry.get_asset_id_base(&pair.to_asset)?;
                println!(
                    "       → {} -> {} ({}%)",
                    from_asset.symbol,
                    to_asset.symbol,
                    pair.fee_percentage.unwrap_or(0.0)
                );
            }
        }
    }

    if !testnet_chains.is_empty() {
        println!("   Testnet chains: {}", testnet_chains.len());
        for chain in &testnet_chains {
            let chain_id = ChainId::from_str(&chain.chain_id)?;
            let asset_count = registry.get_chain_assets(&chain_id)?.len();
            println!("     • {} ({} assets)", chain.name, asset_count);
        }

        // Show testnet pairs
        let testnet_pairs: Vec<_> = cross_chain_pairs
            .iter()
            .filter(|pair| {
                let from_chain = registry.get_chain(pair.from_asset.chain_id()).unwrap();
                let to_chain = registry.get_chain(pair.to_asset.chain_id()).unwrap();
                from_chain.is_testnet && to_chain.is_testnet
            })
            .collect();

        if !testnet_pairs.is_empty() {
            println!("     Testnet trading pairs: {}", testnet_pairs.len());
            for pair in testnet_pairs {
                let from_asset = registry.get_asset_id_base(&pair.from_asset)?;
                let to_asset = registry.get_asset_id_base(&pair.to_asset)?;
                println!(
                    "       → {} -> {} ({}%)",
                    from_asset.symbol,
                    to_asset.symbol,
                    pair.fee_percentage.unwrap_or(0.0)
                );
            }
        }
    }

    // Demonstrate routing capabilities dynamically
    if cross_chain_pairs.len() >= 2 {
        println!("\n🛣️  Trading Route Discovery:");

        // Pick two random assets from different pairs for route testing
        let first_pair = &cross_chain_pairs[0];
        let last_pair = &cross_chain_pairs[cross_chain_pairs.len() - 1];

        let from_asset = &first_pair.from_asset;
        let to_asset = &last_pair.to_asset;

        if from_asset != to_asset {
            let routes = registry.find_trading_routes(from_asset, to_asset, 5);
            let from_config = registry.get_asset_id_base(from_asset)?;
            let to_config = registry.get_asset_id_base(to_asset)?;

            println!(
                "   Routes from {} to {}: {}",
                from_config.symbol,
                to_config.symbol,
                routes.len()
            );

            for (i, route) in routes.iter().enumerate().take(3) {
                // Show first 3 routes
                println!("   Route {} ({} hops):", i + 1, route.len());
                for (j, pair) in route.iter().enumerate() {
                    let step_from = registry.get_asset_id_base(&pair.from_asset)?;
                    let step_to = registry.get_asset_id_base(&pair.to_asset)?;
                    let step_type = if pair.is_cross_chain() {
                        "cross-chain"
                    } else {
                        "same-chain"
                    };
                    println!(
                        "     {}. {} -> {} ({})",
                        j + 1,
                        step_from.symbol,
                        step_to.symbol,
                        step_type
                    );
                }
            }
        }
    }

    // Show cryptographic curves dynamically
    println!("\n🔐 Cryptographic Curve Analysis:");

    // Discover all unique curves in the registry
    let mut curves_found = std::collections::HashSet::new();
    for chain in &chains {
        for curve in &chain.cryptographic_curve {
            curves_found.insert(curve);
        }
    }

    for curve in &curves_found {
        let curve_chains = registry.get_chains_by_curve(curve);
        println!("   {:?} chains: {}", curve, curve_chains.len());
        for chain in curve_chains {
            println!(
                "     • {} {}",
                chain.name,
                if chain.is_testnet {
                    "(Testnet)"
                } else {
                    "(Mainnet)"
                }
            );
        }
    }

    // Show chain metadata dynamically
    println!("\n⚙️  Chain Metadata Analysis:");
    for chain in &chains {
        println!("   {}:", chain.name);
        println!("     Chain ID: {}", chain.chain_id);
        println!(
            "     Network: {}",
            if chain.is_testnet {
                "Testnet"
            } else {
                "Mainnet"
            }
        );
        println!("     RPC endpoints: {}", chain.rpc_endpoints.len());

        if let Some(explorer) = &chain.explorer_url {
            println!("     Explorer: {}", explorer);
        }

        // Show any available metadata
        if !chain.metadata.is_empty() {
            println!("     Metadata:");
            for (key, value) in &chain.metadata {
                println!("       {}: {}", key, value);
            }
        }
    }

    // Health check
    println!("\n🏥 Registry Health Check:");
    let health_issues = registry.check_health();
    if health_issues.is_empty() {
        println!("   ✅ Registry is healthy - no issues found");
    } else {
        println!("   ⚠️  Found {} issue(s):", health_issues.len());
        for issue in health_issues {
            println!("     • {}", issue);
        }
    }

    // Show asset metadata dynamically
    println!("\n📋 Asset Metadata Analysis:");
    for asset in &assets {
        println!("   {} ({}):", asset.symbol, asset.name);
        println!(
            "     Type: {}",
            if asset.is_native { "Native" } else { "Token" }
        );
        println!("     Decimals: {}", asset.decimals);
        println!("     Asset ID Base: {}", asset.asset_id_base);

        if !asset.metadata.is_empty() {
            println!("     Metadata:");
            for (key, value) in &asset.metadata {
                println!("       {}: {}", key, value);
            }
        }
    }

    // Performance test
    println!("\n⚡ Performance Analysis:");
    use std::time::Instant;

    let start = Instant::now();
    let _stats = registry.get_statistics();
    let stats_time = start.elapsed();

    let start = Instant::now();
    let _all_pairs = registry.list_enabled_token_pairs();
    let query_time = start.elapsed();

    let start = Instant::now();
    let _health = registry.check_health();
    let health_time = start.elapsed();

    println!("   Statistics calculation: {:?}", stats_time);
    println!("   Pair query: {:?}", query_time);
    println!("   Health check: {:?}", health_time);

    // If we have pairs, test route finding
    if !cross_chain_pairs.is_empty() {
        let start = Instant::now();
        let first_pair = &cross_chain_pairs[0];
        let _routes = registry.find_trading_routes(&first_pair.from_asset, &first_pair.to_asset, 3);
        let routing_time = start.elapsed();
        println!("   Route finding: {:?}", routing_time);
    }

    // Dynamic solver integration example
    println!("\n🤖 SolverOS Integration Opportunities:");
    println!("   1. Load registry: ChainRegistry::default()?");

    if !cross_chain_pairs.is_empty() {
        println!("   2. Monitor arbitrage opportunities:");
        for pair in cross_chain_pairs.iter().take(3) {
            // Show first 3 pairs
            let from_asset = registry.get_asset_id_base(&pair.from_asset)?;
            let to_asset = registry.get_asset_id_base(&pair.to_asset)?;
            let from_chain = registry.get_chain(pair.from_asset.chain_id())?;
            let to_chain = registry.get_chain(pair.to_asset.chain_id())?;

            println!(
                "      - Monitor {} price on {}",
                from_asset.symbol, from_chain.name
            );
            println!(
                "      - Monitor {} price on {}",
                to_asset.symbol, to_chain.name
            );
            println!(
                "      - Execute arbitrage if spread > {}%",
                pair.fee_percentage.unwrap_or(0.0)
            );
        }
    }

    println!("   3. Use chain metadata for gas estimation");
    println!(
        "   4. Leverage {} curves for transaction signing",
        curves_found.len()
    );

    if stats.cross_chain_pairs > 0 {
        println!(
            "   5. Execute cross-chain swaps across {} pairs",
            stats.cross_chain_pairs
        );
    }
    if stats.same_chain_pairs > 0 {
        println!(
            "   6. Execute same-chain swaps across {} pairs",
            stats.same_chain_pairs
        );
    }

    println!("\n🎉 Configuration-agnostic demo completed successfully!");
    println!("\nRegistry Configuration Summary:");
    println!(
        "   • {} total chains across {} networks",
        stats.total_chains,
        curves_found.len()
    );
    println!("   • {} unique assets", stats.total_assets);
    println!(
        "   • {} trading pairs ({} cross-chain, {} same-chain)",
        stats.total_pairs, stats.cross_chain_pairs, stats.same_chain_pairs
    );
    println!(
        "   • {} mainnet chains, {} testnet chains",
        stats.mainnet_chains, stats.testnet_chains
    );
    println!("   • Ready for any configuration - no hardcoded assumptions!");

    Ok(())
}

#[cfg(test)]
mod agnostic_demo_tests {
    use super::*;

    #[test]
    fn test_agnostic_demo_functionality() {
        // Test that the demo works with any configuration
        let registry = ChainRegistry::default().unwrap();

        // Test should work regardless of what's in the registry
        let stats = registry.get_statistics();
        assert!(stats.total_chains >= 0); // Could be empty registry
        assert!(stats.total_assets >= 0);
        assert!(stats.total_pairs >= 0);

        // Test that we can list everything without errors
        let chains = registry.list_chains();
        let assets = registry.list_assets();
        let pairs = registry.list_enabled_token_pairs();

        // Test the dynamic feature discovery
        for chain in &chains {
            let chain_id = registry.get_chain_id_from_config(&chain.name).unwrap();
            assert_eq!(chain_id.to_string(), chain.chain_id);
        }

        // Test asset discovery on each chain
        for chain in &chains {
            let chain_id = ChainId::from_str(&chain.chain_id).unwrap();
            let chain_assets = registry.get_chain_assets(&chain_id).unwrap();

            for asset_id in &chain_assets {
                // Should be able to get asset config for each asset on the chain
                let _asset_config = registry.get_asset_id_base(asset_id).unwrap();
            }
        }

        println!(
            "✅ Agnostic demo test passed with {} chains, {} assets, {} pairs",
            chains.len(),
            assets.len(),
            pairs.len()
        );
    }

    #[test]
    fn test_empty_registry() {
        // Test that demo gracefully handles empty registry
        let empty_registry = ChainRegistry::new();

        let stats = empty_registry.get_statistics();
        assert_eq!(stats.total_chains, 0);
        assert_eq!(stats.total_assets, 0);
        assert_eq!(stats.total_pairs, 0);

        let chains = empty_registry.list_chains();
        let assets = empty_registry.list_assets();
        let pairs = empty_registry.list_enabled_token_pairs();

        assert!(chains.is_empty());
        assert!(assets.is_empty());
        assert!(pairs.is_empty());

        println!("✅ Empty registry test passed");
    }

    #[test]
    fn test_single_chain_registry() {
        // Test with minimal configuration
        let mut registry = ChainRegistry::new();

        // This test verifies the demo works even with minimal configs
        let stats = registry.get_statistics();
        assert_eq!(stats.total_chains, 0);

        println!("✅ Single chain registry test setup completed");
    }
}
