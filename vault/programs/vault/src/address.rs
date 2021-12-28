pub mod usdc {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet")]
    declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    #[cfg(not(feature = "mainnet"))]
    declare_id!("6PEh8n3p7BbCTykufbq1nSJYAZvUp6gSwEANAs1ZhsCX");
}
