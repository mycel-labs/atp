use candid::CandidType;
/// Common chain namespaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType)]
pub enum ChainNamespace {
    Eip155,   // Ethereum and EVM-compatible chains
    Solana,   // Solana
    Cosmos,   // Cosmos chains
    Polkadot, // Polkadot chains
    Bip155,   // Bitcoin and Bitcoin-compatible chains
    Other(&'static str),
}

impl ChainNamespace {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Eip155 => "eip155",
            Self::Solana => "solana",
            Self::Cosmos => "cosmos",
            Self::Polkadot => "polkadot",
            Self::Bip155 => "bip155",
            Self::Other(s) => s,
        }
    }
}

impl From<ChainNamespace> for String {
    fn from(namespace: ChainNamespace) -> Self {
        namespace.as_str().to_string()
    }
}

/// Common asset namespaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType)]
pub enum AssetNamespace {
    Slip44,  // Native tokens
    Erc20,   // ERC-20 tokens
    Erc721,  // ERC-721 NFTs
    Erc1155, // ERC-1155 tokens
    Spl,     // Solana SPL tokens
    Other(&'static str),
}

impl AssetNamespace {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Slip44 => "slip44",
            Self::Erc20 => "erc20",
            Self::Erc721 => "erc721",
            Self::Erc1155 => "erc1155",
            Self::Spl => "spl",
            Self::Other(s) => s,
        }
    }
}

impl From<AssetNamespace> for String {
    fn from(namespace: AssetNamespace) -> Self {
        namespace.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_namespace() {
        assert_eq!(ChainNamespace::Eip155.as_str(), "eip155");
        assert_eq!(ChainNamespace::Solana.as_str(), "solana");
        assert_eq!(ChainNamespace::Cosmos.as_str(), "cosmos");
        assert_eq!(ChainNamespace::Polkadot.as_str(), "polkadot");
        assert_eq!(ChainNamespace::Bip155.as_str(), "bip155");
        assert_eq!(ChainNamespace::Other("custom").as_str(), "custom");

        let ns_str: String = ChainNamespace::Eip155.into();
        assert_eq!(ns_str, "eip155");
    }

    #[test]
    fn test_asset_namespace() {
        assert_eq!(AssetNamespace::Slip44.as_str(), "slip44");
        assert_eq!(AssetNamespace::Erc20.as_str(), "erc20");
        assert_eq!(AssetNamespace::Erc721.as_str(), "erc721");
        assert_eq!(AssetNamespace::Erc1155.as_str(), "erc1155");
        assert_eq!(AssetNamespace::Spl.as_str(), "spl");
        assert_eq!(AssetNamespace::Other("custom").as_str(), "custom");

        let ns_str: String = AssetNamespace::Erc20.into();
        assert_eq!(ns_str, "erc20");
    }
}
