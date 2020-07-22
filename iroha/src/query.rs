//! This module contains query related Iroha functionality.

use crate::{account, asset, domain, dex, prelude::*};
use iroha_derive::Io;
use parity_scale_codec::{Decode, Encode};

/// I/O ready structure to send queries.
#[derive(Debug, Io, Encode, Decode)]
pub struct QueryRequest {
    /// Timestamp of the query creation.
    pub timestamp: String,
    /// Optional query signature.
    pub signature: Option<Signature>,
    /// Query definition.
    pub query: IrohaQuery,
}

/// Enumeration of all legal Iroha Queries.
#[derive(Clone, Debug, Encode, Decode)]
pub enum IrohaQuery {
    /// Query all Assets related to the Account.
    GetAccountAssets(asset::query::GetAccountAssets),
    /// Query all Assets with defined Definition related to the Account.
    GetAccountAssetsWithDefinition(asset::query::GetAccountAssetsWithDefinition),
    /// Query Account information.
    GetAccount(account::query::GetAccount),
    /// Query Domain information.
    GetDomain(domain::query::GetDomain),
    /// Query DEX information.
    GetDEX(dex::query::GetDEX),
    /// Query all active DEX in the network.
    GetDEXList(dex::query::GetDEXList),
    /// Query Token Pair information.
    GetTokenPair(dex::query::GetTokenPair),
    /// Query all active Token Pairs for DEX.
    GetTokenPairList(dex::query::GetTokenPairList),
    /// Query count of active Token Pairs in DEX.
    GetTokenPairCount(dex::query::GetTokenPairCount),
    /// Query info about active XYK Pool.
    GetXYKPoolInfo(dex::query::GetXYKPoolInfo),
    /// Query fee applied to swaps.
    GetFeeOnXYKPool(dex::query::GetFeeOnXYKPool),
    /// Query fee fraction that is treated as protocol fee.
    GetProtocolFeePartOnXYKPool(dex::query::GetProtocolFeePartOnXYKPool),
    /// Query spot price of token via indicated exchange path with desired input amount.
    GetPriceForInputTokensOnXYKPool(dex::query::GetPriceForInputTokensOnXYKPool),
    /// Query spot price of token via indicated exchange path with desired output amount.
    GetPriceForOutputTokensOnXYKPool(dex::query::GetPriceForOutputTokensOnXYKPool),
    /// Query base and target token quantities that will be returned by burning pool tokens.
    GetOwnedLiquidityOnXYKPool(dex::query::GetOwnedLiquidityOnXYKPool),
}

/// Result of queries execution.
#[derive(Debug, Io, Encode, Decode)]
pub enum QueryResult {
    /// Query all Assets related to the Account result.
    GetAccountAssets(asset::query::GetAccountAssetsResult),
    /// Query all Assets with defined Definition related to the Account.
    GetAccountAssetsWithDefinition(asset::query::GetAccountAssetsWithDefinitionResult),
    /// Query Account information result.
    GetAccount(account::query::GetAccountResult),
    /// Query Domain information.
    GetDomain(domain::query::GetDomainResult),
    /// Query DEX information.
    GetDEX(dex::query::GetDEXResult),
    /// Query all active DEX in the network result.
    GetDEXList(dex::query::GetDEXListResult),
    /// Query all active Token Pairs for DEX result.
    GetTokenPair(dex::query::GetTokenPairResult),
    /// Query all active Token Pairs for DEX result.
    GetTokenPairList(dex::query::GetTokenPairListResult),
    /// Query count of active Token Pairs in DEX result.
    GetTokenPairCount(dex::query::GetTokenPairCountResult),
    /// Query info about active XYK Pool result.
    GetXYKPoolInfo(dex::query::GetXYKPoolInfoResult),
    /// Query fee applied to swaps.
    GetFeeOnXYKPool(dex::query::GetFeeOnXYKPoolResult),
    /// Query fee fraction that is treated as protocol fee.
    GetProtocolFeePartOnXYKPool(dex::query::GetProtocolFeePartOnXYKPoolResult),
    /// Query price of token via indicated exchange path, indicate desired input amount.
    GetPriceForInputTokensOnXYKPool(dex::query::GetPriceOnXYKPoolResult),
    /// Query price of token via indicated exchange path, indicate desired output amount.
    GetPriceForOutputTokensOnXYKPool(dex::query::GetPriceOnXYKPoolResult),
    /// Query base and target token quantities that will be returned by burning pool tokens.
    GetOwnedLiquidityOnXYKPool(dex::query::GetOwnedLiquidityOnXYKPoolResult),
}

impl IrohaQuery {
    /// Execute query on the `WorldStateView`.
    ///
    /// Returns Ok(QueryResult) if succeeded and Err(String) if failed.
    pub fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
        match self {
            IrohaQuery::GetAccountAssets(query) => query.execute(world_state_view),
            IrohaQuery::GetAccountAssetsWithDefinition(query) => query.execute(world_state_view),
            IrohaQuery::GetAccount(query) => query.execute(world_state_view),
            IrohaQuery::GetDomain(query) => query.execute(world_state_view),
            IrohaQuery::GetDEX(query) => query.execute(world_state_view),
            IrohaQuery::GetDEXList(query) => query.execute(world_state_view),
            IrohaQuery::GetTokenPair(query) => query.execute(world_state_view),
            IrohaQuery::GetTokenPairList(query) => query.execute(world_state_view),
            IrohaQuery::GetTokenPairCount(query) => query.execute(world_state_view),
            IrohaQuery::GetXYKPoolInfo(query) => query.execute(world_state_view),
            IrohaQuery::GetFeeOnXYKPool(query) => query.execute(world_state_view),
            IrohaQuery::GetProtocolFeePartOnXYKPool(query) => query.execute(world_state_view),
            IrohaQuery::GetPriceForInputTokensOnXYKPool(query) => query.execute(world_state_view),
            IrohaQuery::GetPriceForOutputTokensOnXYKPool(query) => query.execute(world_state_view),
            IrohaQuery::GetOwnedLiquidityOnXYKPool(query) => query.execute(world_state_view),
        }
    }
}

/// This trait should be implemented for all Iroha Queries.
pub trait Query {
    /// Execute query on the `WorldStateView`.
    ///
    /// Returns Ok(QueryResult) if succeeded and Err(String) if failed.
    fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String>;
}
