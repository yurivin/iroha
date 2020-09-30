//! This module contains functionality related to `DEX`.

use crate::permission::*;
use crate::prelude::*;
use integer_sqrt::*;
use iroha_derive::Io;
use parity_scale_codec::{Decode, Encode};
use std::cmp;
use std::collections::BTreeMap;
use std::mem;

const STORAGE_ACCOUNT_NAME: &str = "STORE";
const XYK_POOL: &str = "XYKPOOL";
const MINIMUM_LIQUIDITY: u32 = 1000;
const MAX_BASIS_POINTS: u16 = 10000;

type Name = String;

/// Identification of a DEX, consists of its domain name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct DEXId {
    domain_name: Name,
}

impl DEXId {
    /// Default DEX identifier constructor.
    pub fn new(domain_name: &str) -> Self {
        DEXId {
            domain_name: domain_name.to_owned(),
        }
    }
}

/// `DEX` entity encapsulates data and logic for management of
/// decentralized exchanges in domains.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, Io)]
pub struct DEX {
    /// An identification of the `DEX`.
    pub id: <DEX as Identifiable>::Id,
    /// DEX owner's account Identification. Only this account will be able to manipulate the DEX.
    pub owner_account_id: <Account as Identifiable>::Id,
    /// Token Pairs belonging to this dex.
    pub token_pairs: BTreeMap<<TokenPair as Identifiable>::Id, TokenPair>,
    /// Base Asset identification.
    pub base_asset_id: <AssetDefinition as Identifiable>::Id,
}

impl DEX {
    /// Default `DEX` entity constructor.
    pub fn new(
        domain_name: &str,
        owner_account_id: <Account as Identifiable>::Id,
        base_asset_id: <AssetDefinition as Identifiable>::Id,
    ) -> Self {
        DEX {
            id: DEXId::new(domain_name),
            owner_account_id,
            token_pairs: BTreeMap::new(),
            base_asset_id,
        }
    }
}

impl Identifiable for DEX {
    type Id = DEXId;
}

/// Identification of a Token Pair. Consists of underlying asset ids.
#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Io)]
pub struct TokenPairId {
    /// Containing DEX identifier.
    pub dex_id: <DEX as Identifiable>::Id,
    /// Base token of exchange.
    pub base_asset_id: <AssetDefinition as Identifiable>::Id,
    /// Target token of exchange.
    pub target_asset_id: <AssetDefinition as Identifiable>::Id,
}

impl TokenPairId {
    /// Default Token Pair identifier constructor.
    pub fn new(
        dex_id: <DEX as Identifiable>::Id,
        base_asset_id: <AssetDefinition as Identifiable>::Id,
        target_asset_id: <AssetDefinition as Identifiable>::Id,
    ) -> Self {
        TokenPairId {
            dex_id,
            base_asset_id,
            target_asset_id,
        }
    }
    /// Symbol representation of the Token Pair.
    pub fn get_symbol(&self) -> String {
        format!(
            "{}-{}/{}-{}",
            self.base_asset_id.name,
            self.base_asset_id.domain_name,
            self.target_asset_id.name,
            self.target_asset_id.domain_name
        )
    }
}

/// `TokenPair` represents an exchange pair between two assets in a domain. Assets are
/// identified by their AssetDefinitionId's. Containing DEX is identified by domain name.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
pub struct TokenPair {
    /// An Identification of the `TokenPair`, holds pair of token Ids.
    pub id: <TokenPair as Identifiable>::Id,
    /// Liquidity Sources belonging to this TokenPair. At most one instance
    /// of each type.
    pub liquidity_sources: BTreeMap<<LiquiditySource as Identifiable>::Id, LiquiditySource>,
}

impl Identifiable for TokenPair {
    type Id = TokenPairId;
}

impl TokenPair {
    /// Default Token Pair constructor.
    pub fn new(
        dex_id: <DEX as Identifiable>::Id,
        base_asset: <AssetDefinition as Identifiable>::Id,
        target_asset: <AssetDefinition as Identifiable>::Id,
    ) -> Self {
        TokenPair {
            id: TokenPairId::new(dex_id, base_asset, target_asset),
            liquidity_sources: BTreeMap::new(),
        }
    }
}

/// Identification of a Liquidity Source. Consists of Token Pair Id and underlying
/// liquidity source type.
#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub struct LiquiditySourceId {
    /// Identifier of containing token pair.
    pub token_pair_id: <TokenPair as Identifiable>::Id,
    /// Type of liquidity source.
    pub liquidity_source_type: LiquiditySourceType,
}

impl LiquiditySourceId {
    /// Default constructor for Liquidity Source identifier.
    pub fn new(
        token_pair_id: <TokenPair as Identifiable>::Id,
        liquidity_source_type: LiquiditySourceType,
    ) -> Self {
        LiquiditySourceId {
            token_pair_id,
            liquidity_source_type,
        }
    }
}

/// Enumration representing types of liquidity sources.
#[non_exhaustive]
#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub enum LiquiditySourceType {
    /// X*Y=K model liquidity pool.
    XYKPool,
}

/// Data storage for XYK liquidity source.
#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub struct XYKPoolData {
    /// Asset definition of pool token belonging to given pool.
    pub pool_token_asset_definition_id: <AssetDefinition as Identifiable>::Id,
    /// Account that is used to store pool reserves, i.e. actual liquidity.
    storage_account_id: <Account as Identifiable>::Id,
    /// Account that will receive protocol fee part if enabled.
    fee_to: Option<<Account as Identifiable>::Id>,
    /// Fee for swapping tokens on pool, expressed in basis points.
    pub fee: u16,
    /// Fee fraction which is deduced from `fee` as protocol fee, expressed in basis points.
    pub protocol_fee_part: u16,
    /// Amount of active pool tokens.
    pub pool_token_total_supply: u32,
    /// Amount of base tokens in the pool (currently stored in storage account).
    pub base_asset_reserve: u32,
    /// Amount of target tokens in the pool (currently stored in storage account).
    pub target_asset_reserve: u32,
    /// K (constant product) value, updated by latest liquidity operation.
    k_last: u32,
}

impl XYKPoolData {
    /// Default constructor for XYK Pool Data entity.
    pub fn new(
        pool_token_asset_definition_id: <AssetDefinition as Identifiable>::Id,
        storage_account_id: <Account as Identifiable>::Id,
    ) -> Self {
        XYKPoolData {
            pool_token_asset_definition_id,
            storage_account_id,
            fee_to: None,
            fee: 30,
            protocol_fee_part: 0,
            pool_token_total_supply: 0,
            base_asset_reserve: 0,
            target_asset_reserve: 0,
            k_last: 0,
        }
    }
}

/// Try to unwrap reference `XYKPoolData` from `LiquiditySourceData` enum of `LiquiditySource` entity.
#[allow(unreachable_patterns)]
pub fn expect_xyk_pool_data(liquidity_source: &LiquiditySource) -> Result<&XYKPoolData, String> {
    match &liquidity_source.data {
        LiquiditySourceData::XYKPool(data) => Ok(data),
        _ => Err("wrong liquidity source data".to_owned()),
    }
}

/// Try to unwrap mutable reference `XYKPoolData` from `LiquiditySourceData` enum of `LiquiditySource` entity.
#[allow(unreachable_patterns)]
pub fn expect_xyk_pool_data_mut(
    liquidity_source: &mut LiquiditySource,
) -> Result<&mut XYKPoolData, String> {
    match &mut liquidity_source.data {
        LiquiditySourceData::XYKPool(data) => Ok(data),
        _ => Err("wrong liquidity source data".to_owned()),
    }
}

/// `LiquiditySource` represents an exchange pair between two assets in a domain. Assets are
/// identified by their AssetDefinitionId's. Containing DEX is identified by domain name.
#[non_exhaustive]
#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub enum LiquiditySourceData {
    /// Data representing state of the XYK liquidity pool.
    XYKPool(XYKPoolData),
}

/// Liquidity Source entity belongs to particular Token Pair, exchange operations
/// are handled through it.
#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub struct LiquiditySource {
    /// Identification of Liquidity source.
    pub id: <LiquiditySource as Identifiable>::Id,
    /// Varients represent LiquiditySourceType-specific data set for Liquidity Source.
    pub data: LiquiditySourceData,
}

impl Identifiable for LiquiditySource {
    type Id = LiquiditySourceId;
}

impl LiquiditySource {
    /// Default XYK Pool constructor.
    pub fn new_xyk_pool(
        token_pair_id: <TokenPair as Identifiable>::Id,
        pool_token_asset_definition_id: <AssetDefinition as Identifiable>::Id,
        storage_account_id: <Account as Identifiable>::Id,
    ) -> Self {
        let data = LiquiditySourceData::XYKPool(XYKPoolData::new(
            pool_token_asset_definition_id,
            storage_account_id,
        ));
        let id = LiquiditySourceId::new(token_pair_id, LiquiditySourceType::XYKPool);
        LiquiditySource { id, data }
    }
}

/// Iroha Special Instructions module provides helper-methods for operating DEX,
/// Token Pairs and Liquidity Sources.
pub mod isi {
    use super::*;
    use crate::dex::query::*;
    use crate::isi::prelude::*;
    use crate::permission::isi::PermissionInstruction;
    use std::collections::btree_map::Entry;

    /// Enumeration of all legal DEX related Instructions.
    #[derive(Clone, Debug, Io, Encode, Decode)]
    pub enum DEXInstruction {
        /// Variant of instruction to initialize `DEX` entity in `Domain`.
        InitializeDEX(DEX, <Domain as Identifiable>::Id),
        /// Variant of instruction to create new `TokenPair` entity in `DEX`.
        CreateTokenPair(TokenPair, <DEX as Identifiable>::Id),
        /// Variant of instruction to remove existing `TokenPair` entity from `DEX`.
        RemoveTokenPair(<TokenPair as Identifiable>::Id, <DEX as Identifiable>::Id),
        /// Variant of instruction to create liquidity source for existing `TokenPair`.
        CreateLiquiditySource(LiquiditySource, <TokenPair as Identifiable>::Id),
        /// Variant of instruction to deposit tokens to liquidity pool.
        /// `LiquiditySource` <-- Quantity Base Desired, Quantity Target Desired, Quantity Base Min, Quantity Target Min
        AddLiquidityToXYKPool(<LiquiditySource as Identifiable>::Id, u32, u32, u32, u32),
        /// Variant of instruction to withdraw tokens from liquidity pool by burning pool token.
        /// `LiquiditySource` --> Liquidity, Quantity Base Min, Quantity Target Min
        RemoveLiquidityFromXYKPool(<LiquiditySource as Identifiable>::Id, u32, u32, u32),
        /// Variant of instruction to swap with exact quantity of input tokens and receive corresponding quantity of output tokens.
        /// `AssetDefinition`'s, Input Quantity --> Output Quantity Min
        SwapExactTokensForTokensOnXYKPool(
            <DEX as Identifiable>::Id,
            Vec<<AssetDefinition as Identifiable>::Id>,
            u32,
            u32,
        ),
        /// Variant of instruction to swap with exact quantity of output tokens and send corresponding quantity of input tokens.
        /// `AssetDefinition`'a, Output Quantity --> Input Quantity Max
        SwapTokensForExactTokensOnXYKPool(
            <DEX as Identifiable>::Id,
            Vec<<AssetDefinition as Identifiable>::Id>,
            u32,
            u32,
        ),
        /// Variant of instruction to set value in basis points that is deduced from input quantity on swaps.
        SetFeeOnXYKPool(<LiquiditySource as Identifiable>::Id, u16),
        /// Variant of instruction to set value in basis points that is deduced from swap fees as protocol fee.
        SetProtocolFeePartOnXYKPool(<LiquiditySource as Identifiable>::Id, u16),
        /// Variant of instruction to mint permissions for account.
        /// TODO: this isi is debug-only and should be deleted when permission minting is elaborated in core
        AddTransferPermissionForAccount(
            <AssetDefinition as Identifiable>::Id,
            <Account as Identifiable>::Id,
        ),
    }

    impl DEXInstruction {
        /// Executes `DEXInstruction` on the given `WorldStateView`.
        /// Returns `Ok(())` if execution succeeded and `Err(String)` with error message if not.
        pub fn execute(
            &self,
            authority: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            match self {
                DEXInstruction::InitializeDEX(dex, domain_name) => {
                    Register::new(dex.clone(), domain_name.clone())
                        .execute(authority, world_state_view)
                }
                DEXInstruction::CreateTokenPair(token_pair, dex_id) => {
                    Add::new(token_pair.clone(), dex_id.clone())
                        .execute(authority, world_state_view)
                }
                DEXInstruction::RemoveTokenPair(token_pair_id, dex_id) => {
                    Remove::new(token_pair_id.clone(), dex_id.clone())
                        .execute(authority, world_state_view)
                }
                DEXInstruction::CreateLiquiditySource(liquidity_source, token_pair_id) => {
                    Add::new(liquidity_source.clone(), token_pair_id.clone())
                        .execute(authority, world_state_view)
                }
                DEXInstruction::AddLiquidityToXYKPool(
                    liquidity_source_id,
                    base_asset_desired_amount,
                    target_asset_desired_amount,
                    base_asset_min_amount,
                    target_asset_min_amount,
                ) => xyk_pool::AddLiquidity::new(
                    liquidity_source_id.clone(),
                    base_asset_desired_amount.clone(),
                    target_asset_desired_amount.clone(),
                    base_asset_min_amount.clone(),
                    target_asset_min_amount.clone(),
                    authority.clone(),
                )
                .execute(authority, world_state_view),
                DEXInstruction::RemoveLiquidityFromXYKPool(
                    liquidity_source_id,
                    pool_tokens_amount,
                    base_asset_min_amount,
                    target_asset_min_amount,
                ) => xyk_pool::RemoveLiquidity::new(
                    liquidity_source_id.clone(),
                    pool_tokens_amount.clone(),
                    base_asset_min_amount.clone(),
                    target_asset_min_amount.clone(),
                    authority.clone(),
                )
                .execute(authority, world_state_view),
                DEXInstruction::SwapExactTokensForTokensOnXYKPool(
                    dex_id,
                    path,
                    amount_in,
                    amount_out_min,
                ) => xyk_pool::SwapExactTokensForTokens::new(
                    dex_id.clone(),
                    &path,
                    amount_in.clone(),
                    amount_out_min.clone(),
                    authority.clone(),
                )
                .execute(authority, world_state_view),
                DEXInstruction::SwapTokensForExactTokensOnXYKPool(
                    dex_id,
                    path,
                    amount_out,
                    amount_in_max,
                ) => xyk_pool::SwapTokensForExactTokens::new(
                    dex_id.clone(),
                    &path,
                    amount_out.clone(),
                    amount_in_max.clone(),
                    authority.clone(),
                )
                .execute(authority, world_state_view),
                DEXInstruction::SetFeeOnXYKPool(liquidity_source_id, fee) => {
                    xyk_pool::set_fee_execute(
                        liquidity_source_id.clone(),
                        fee.clone(),
                        authority,
                        world_state_view,
                    )
                }
                DEXInstruction::SetProtocolFeePartOnXYKPool(
                    liquidity_source_id,
                    protocol_fee_part,
                ) => xyk_pool::set_protocol_fee_part_execute(
                    liquidity_source_id.clone(),
                    protocol_fee_part.clone(),
                    authority,
                    world_state_view,
                ),
                DEXInstruction::AddTransferPermissionForAccount(
                    asset_definition_id,
                    account_id,
                ) => add_transfer_permission_for_account_execute(
                    asset_definition_id.clone(),
                    account_id.clone(),
                    authority,
                    world_state_view,
                ),
            }
        }
    }

    /// Constructor of `Register<Domain, DEX>` ISI.
    ///
    /// Initializes DEX for the domain.
    pub fn initialize_dex(
        domain_name: &str,
        owner_account_id: <Account as Identifiable>::Id,
        base_asset_id: <AssetDefinition as Identifiable>::Id,
    ) -> Register<Domain, DEX> {
        Register {
            object: DEX::new(domain_name, owner_account_id, base_asset_id),
            destination_id: domain_name.to_owned(),
        }
    }

    impl Register<Domain, DEX> {
        pub(crate) fn execute(
            self,
            authority: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            let dex = self.object;
            let domain_name = self.destination_id;
            PermissionInstruction::CanInitializeDEX(authority).execute(world_state_view)?;
            world_state_view
                .read_account(&dex.owner_account_id)
                .ok_or("account not found")?;
            let domain = get_domain_mut(&domain_name, world_state_view)?;
            if domain.dex.is_none() {
                domain.dex = Some(dex);
                Ok(())
            } else {
                Err("dex is already initialized for domain".to_owned())
            }
        }
    }

    impl From<Register<Domain, DEX>> for Instruction {
        fn from(instruction: Register<Domain, DEX>) -> Self {
            Instruction::DEX(DEXInstruction::InitializeDEX(
                instruction.object,
                instruction.destination_id,
            ))
        }
    }

    /// Constructor of `Add<DEX, TokenPair>` ISI.
    ///
    /// Creates new Token Pair via given asset ids for the DEX
    /// identified by its domain name.
    pub fn create_token_pair(
        base_asset: <AssetDefinition as Identifiable>::Id,
        target_asset: <AssetDefinition as Identifiable>::Id,
        domain_name: &str,
    ) -> Add<DEX, TokenPair> {
        let dex_id = DEXId::new(domain_name);
        Add {
            object: TokenPair::new(dex_id.clone(), base_asset, target_asset),
            destination_id: dex_id,
        }
    }

    impl Add<DEX, TokenPair> {
        pub(crate) fn execute(
            self,
            authority: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            let token_pair = self.object;
            let domain_name = self.destination_id.domain_name;
            PermissionInstruction::CanManageDEX(authority, Some(domain_name.clone()))
                .execute(world_state_view)?;
            let base_asset_definition = &token_pair.id.base_asset_id;
            let target_asset_definition = &token_pair.id.target_asset_id;
            let dex_base_asset_id = get_dex(&domain_name, world_state_view)?
                .base_asset_id
                .clone();
            if base_asset_definition == target_asset_definition {
                return Err("assets in token pair must be different".to_owned());
            }
            let base_asset_domain =
                get_domain(&base_asset_definition.domain_name, world_state_view)?;
            let target_asset_domain =
                get_domain(&target_asset_definition.domain_name, world_state_view)?;
            if !base_asset_domain
                .asset_definitions
                .contains_key(base_asset_definition)
            {
                return Err(format!(
                    "base asset definition: {:?} not found",
                    base_asset_definition
                ));
            }
            if base_asset_definition != &dex_base_asset_id {
                return Err(format!(
                    "base asset definition is incorrect: {} != {}",
                    base_asset_definition, dex_base_asset_id
                ));
            }
            if !target_asset_domain
                .asset_definitions
                .contains_key(target_asset_definition)
            {
                return Err(format!(
                    "target asset definition: {:?} not found",
                    target_asset_definition
                ));
            }
            let dex = get_dex_mut(&domain_name, world_state_view)?;
            match dex.token_pairs.entry(token_pair.id.clone()) {
                Entry::Occupied(_) => Err("token pair already exists".to_owned()),
                Entry::Vacant(entry) => {
                    entry.insert(token_pair);
                    Ok(())
                }
            }
        }
    }

    impl From<Add<DEX, TokenPair>> for Instruction {
        fn from(instruction: Add<DEX, TokenPair>) -> Self {
            Instruction::DEX(DEXInstruction::CreateTokenPair(
                instruction.object,
                instruction.destination_id,
            ))
        }
    }

    /// Constructor of `Remove<DEX, TokenPairId>` ISI.
    ///
    /// Removes existing Token Pair by its id from the DEX.
    pub fn remove_token_pair(
        token_pair_id: <TokenPair as Identifiable>::Id,
    ) -> Remove<DEX, <TokenPair as Identifiable>::Id> {
        let dex_id = DEXId::new(&token_pair_id.dex_id.domain_name);
        Remove {
            object: token_pair_id,
            destination_id: dex_id,
        }
    }

    impl Remove<DEX, <TokenPair as Identifiable>::Id> {
        pub(crate) fn execute(
            self,
            authority: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            let token_pair_id = self.object;
            PermissionInstruction::CanManageDEX(
                authority,
                Some(token_pair_id.dex_id.domain_name.clone()),
            )
            .execute(world_state_view)?;
            let dex = get_dex_mut(&token_pair_id.dex_id.domain_name, world_state_view)?;
            match dex.token_pairs.entry(token_pair_id.clone()) {
                Entry::Occupied(entry) => {
                    entry.remove();
                    Ok(())
                }
                Entry::Vacant(_) => Err("token pair does not exist".to_owned()),
            }
        }
    }

    impl From<Remove<DEX, <TokenPair as Identifiable>::Id>> for Instruction {
        fn from(instruction: Remove<DEX, <TokenPair as Identifiable>::Id>) -> Self {
            Instruction::DEX(DEXInstruction::RemoveTokenPair(
                instruction.object,
                instruction.destination_id,
            ))
        }
    }

    /// XYK Pool module provides functions to operate the pool,
    /// namely: CreatePool, AddLiquidity, RemoveLiquidity, SwapTokens
    pub mod xyk_pool {
        use super::*;

        /// Construct name of pool token based on TokenPair to which it belongs.
        pub fn token_asset_name(token_pair_id: &<TokenPair as Identifiable>::Id) -> String {
            format!("{} {}", XYK_POOL, token_pair_id.get_symbol())
        }

        /// Construct name of pool storage account based on TokenPair to which it belongs.
        pub fn storage_account_name(token_pair_id: &<TokenPair as Identifiable>::Id) -> String {
            format!(
                "{} {} {}",
                STORAGE_ACCOUNT_NAME,
                XYK_POOL,
                token_pair_id.get_symbol()
            )
        }

        /// Constructor of `Add<DEX, LiquiditySource>` ISI.
        ///
        /// Add new XYK Liquidity Pool for DEX with given `TokenPair`.
        pub fn create(token_pair_id: <TokenPair as Identifiable>::Id) -> Instruction {
            let domain_name = token_pair_id.dex_id.domain_name.clone();
            let asset_name = token_asset_name(&token_pair_id);
            let pool_token_asset_definition =
                AssetDefinition::new(AssetDefinitionId::new(&asset_name, &domain_name));
            let storage_account_name = storage_account_name(&token_pair_id);
            let storage_account = Account::new(&storage_account_name, &domain_name);
            Instruction::If(
                Box::new(Instruction::ExecuteQuery(IrohaQuery::GetTokenPair(
                    GetTokenPair {
                        token_pair_id: token_pair_id.clone(),
                    },
                ))),
                Box::new(Instruction::Sequence(vec![
                    // register asset definition for pool_token
                    Register {
                        object: pool_token_asset_definition.clone(),
                        destination_id: domain_name.clone(),
                    }
                    .into(),
                    // register storage account for pool
                    Register {
                        object: storage_account.clone(),
                        destination_id: domain_name.clone(),
                    }
                    .into(),
                    // create xyk pool for pair
                    Add {
                        object: LiquiditySource::new_xyk_pool(
                            token_pair_id.clone(),
                            pool_token_asset_definition.id.clone(),
                            storage_account.id.clone(),
                        ),
                        destination_id: token_pair_id,
                    }
                    .into(),
                ])),
                Some(Box::new(Instruction::Fail(
                    "token pair not found".to_string(),
                ))),
            )
        }

        impl Add<TokenPair, LiquiditySource> {
            pub(crate) fn execute(
                self,
                authority: <Account as Identifiable>::Id,
                world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                let liquidity_source = self.object;
                let token_pair_id = &liquidity_source.id.token_pair_id;
                PermissionInstruction::CanManageDEX(
                    authority,
                    Some(token_pair_id.dex_id.domain_name.clone()),
                )
                .execute(world_state_view)?;
                let token_pair = get_token_pair_mut(token_pair_id, world_state_view)?;
                match token_pair
                    .liquidity_sources
                    .entry(liquidity_source.id.clone())
                {
                    Entry::Occupied(_) => {
                        Err("liquidity source already exists for token pair".to_owned())
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(liquidity_source);
                        Ok(())
                    }
                }
            }
        }

        impl From<Add<TokenPair, LiquiditySource>> for Instruction {
            fn from(instruction: Add<TokenPair, LiquiditySource>) -> Self {
                Instruction::DEX(DEXInstruction::CreateLiquiditySource(
                    instruction.object,
                    instruction.destination_id,
                ))
            }
        }

        impl XYKPoolData {
            /// Mint pool_token tokens representing liquidity in pool for depositing account.
            pub fn mint_pool_token_with_fee(
                &mut self,
                to: <Account as Identifiable>::Id,
                token_pair_id: <TokenPair as Identifiable>::Id,
                base_asset_amount: u32,
                target_asset_amount: u32,
                world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                let balance_base = get_asset_quantity(
                    self.storage_account_id.clone(),
                    token_pair_id.base_asset_id.clone(),
                    world_state_view,
                )?;
                let balance_target = get_asset_quantity(
                    self.storage_account_id.clone(),
                    token_pair_id.target_asset_id.clone(),
                    world_state_view,
                )?;
                self.mint_protocol_fee(world_state_view)?;
                let liquidity;
                if self.pool_token_total_supply == 0 {
                    liquidity = (base_asset_amount * target_asset_amount).integer_sqrt()
                        - MINIMUM_LIQUIDITY;
                    self.pool_token_total_supply = MINIMUM_LIQUIDITY;
                } else {
                    liquidity = cmp::min(
                        (base_asset_amount * self.pool_token_total_supply)
                            / self.base_asset_reserve,
                        (target_asset_amount * self.pool_token_total_supply)
                            / self.target_asset_reserve,
                    );
                };
                if !(liquidity > 0) {
                    return Err("insufficient liquidity minted".to_owned());
                }
                self.mint_pool_token(to, liquidity, world_state_view)?;
                self.update(balance_base, balance_target, world_state_view)?;
                if self.fee_to.is_some() {
                    self.k_last = self.base_asset_reserve * self.target_asset_reserve;
                }
                Ok(())
            }

            /// Mint pool token for protocol account if configured.
            pub fn mint_protocol_fee(
                &mut self,
                world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                if let Some(fee_to) = self.fee_to.clone() {
                    if self.k_last != 0 {
                        let root_k =
                            (self.base_asset_reserve * self.target_asset_reserve).integer_sqrt();
                        let root_k_last = (self.k_last).integer_sqrt();
                        if root_k > root_k_last {
                            let numerator = self.pool_token_total_supply * (root_k - root_k_last);
                            let denominator = (MAX_BASIS_POINTS - self.protocol_fee_part) as u32
                                / self.protocol_fee_part as u32
                                * root_k
                                + root_k_last;
                            let liquidity = numerator / denominator;
                            if liquidity > 0 {
                                self.mint_pool_token(fee_to, liquidity, world_state_view)?;
                            }
                        }
                    }
                } else if self.k_last != 0 {
                    self.k_last = 0;
                }
                Ok(())
            }

            /// Mint pool token for Account.
            pub fn mint_pool_token(
                &mut self,
                to: <Account as Identifiable>::Id,
                quantity: u32,
                world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                let asset_id = AssetId::new(self.pool_token_asset_definition_id.clone(), to);
                mint_asset_unchecked(asset_id, quantity, world_state_view)?;
                self.pool_token_total_supply += quantity;
                Ok(())
            }

            // returns (amount_a, amount_b)
            fn burn_pool_token_with_protocol_fee(
                &mut self,
                pair_tokens_to: <Account as Identifiable>::Id,
                token_pair_id: <TokenPair as Identifiable>::Id,
                liquidity: u32,
                world_state_view: &mut WorldStateView,
            ) -> Result<(u32, u32), String> {
                let balance_base = get_asset_quantity(
                    self.storage_account_id.clone(),
                    token_pair_id.base_asset_id.clone(),
                    world_state_view,
                )?;
                let balance_target = get_asset_quantity(
                    self.storage_account_id.clone(),
                    token_pair_id.target_asset_id.clone(),
                    world_state_view,
                )?;

                self.mint_protocol_fee(world_state_view)?;

                let amount_base = liquidity * balance_base / self.pool_token_total_supply;
                let amount_target = liquidity * balance_target / self.pool_token_total_supply;
                if !(amount_base > 0 && amount_target > 0) {
                    return Err("insufficient liqudity burned".to_owned());
                }
                self.burn_pool_token(self.storage_account_id.clone(), liquidity, world_state_view)?;
                transfer_from_unchecked(
                    token_pair_id.base_asset_id.clone(),
                    self.storage_account_id.clone(),
                    pair_tokens_to.clone(),
                    amount_base,
                    world_state_view,
                )?;
                transfer_from_unchecked(
                    token_pair_id.target_asset_id.clone(),
                    self.storage_account_id.clone(),
                    pair_tokens_to.clone(),
                    amount_target,
                    world_state_view,
                )?;
                // update balances after transfers
                let balance_base = get_asset_quantity(
                    self.storage_account_id.clone(),
                    token_pair_id.base_asset_id.clone(),
                    world_state_view,
                )?;
                let balance_target = get_asset_quantity(
                    self.storage_account_id.clone(),
                    token_pair_id.target_asset_id.clone(),
                    world_state_view,
                )?;

                self.update(balance_base, balance_target, world_state_view)?;

                if !(self.base_asset_reserve > 0 && self.target_asset_reserve > 0) {
                    return Err("Insufficient reserves.".to_string());
                }
                if self.fee_to.is_some() {
                    self.k_last = self.base_asset_reserve * self.target_asset_reserve;
                }
                Ok((amount_base, amount_target))
            }

            /// Burn pool token from Account.
            pub fn burn_pool_token(
                &mut self,
                from: <Account as Identifiable>::Id,
                value: u32,
                world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                let asset_id = AssetId::new(self.pool_token_asset_definition_id.clone(), from);
                burn_asset_unchecked(asset_id, value, world_state_view)?;
                self.pool_token_total_supply -= value;
                Ok(())
            }

            /// Update reserves records up to actual token balance.
            fn update(
                &mut self,
                balance_base: u32,
                balance_target: u32,
                _world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                self.base_asset_reserve = balance_base;
                self.target_asset_reserve = balance_target;
                // TODO: implement accumulators for oracle functionality
                Ok(())
            }
        }

        /// Constructor if `AddLiquidityToXYKPool` ISI.
        pub fn add_liquidity(
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
            base_asset_desired_amount: u32,
            target_asset_desired_amount: u32,
            base_asset_min_amount: u32,
            target_asset_min_amount: u32,
        ) -> Instruction {
            Instruction::DEX(DEXInstruction::AddLiquidityToXYKPool(
                liquidity_source_id,
                base_asset_desired_amount,
                target_asset_desired_amount,
                base_asset_min_amount,
                target_asset_min_amount,
            ))
        }

        /// AddLiquidity instruction is used to deposit pair tokens to pool and receive pool tokens.
        pub struct AddLiquidity {
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
            base_asset_desired_amount: u32,
            target_asset_desired_amount: u32,
            base_asset_min_amount: u32,
            target_asset_min_amount: u32,
            pool_tokens_to: <Account as Identifiable>::Id,
        }

        impl AddLiquidity {
            /// Constructor of `AddLiquidity` instruction.
            pub fn new(
                liquidity_source_id: <LiquiditySource as Identifiable>::Id,
                base_asset_desired_amount: u32,
                target_asset_desired_amount: u32,
                base_asset_min_amount: u32,
                target_asset_min_amount: u32,
                pool_tokens_to: <Account as Identifiable>::Id,
            ) -> Self {
                AddLiquidity {
                    liquidity_source_id,
                    base_asset_desired_amount,
                    target_asset_desired_amount,
                    base_asset_min_amount,
                    target_asset_min_amount,
                    pool_tokens_to,
                }
            }

            /// Core logic of `AddLiquidityToXYKPool` ISI, called by its `execute` function.
            pub fn execute(
                self,
                authority: <Account as Identifiable>::Id,
                world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                let liquidity_source =
                    get_liquidity_source(&self.liquidity_source_id, world_state_view)?;
                let token_pair_id = liquidity_source.id.token_pair_id.clone();
                let mut data = expect_xyk_pool_data(liquidity_source)?.clone();
                // calculate appropriate deposit quantities to preserve pool proportions
                let (amount_base, amount_target) = get_optimal_deposit_amounts(
                    data.base_asset_reserve,
                    data.target_asset_reserve,
                    self.base_asset_desired_amount,
                    self.target_asset_desired_amount,
                    self.base_asset_min_amount,
                    self.target_asset_min_amount,
                )?;
                // deposit tokens into the storage account
                transfer_from(
                    token_pair_id.base_asset_id.clone(),
                    authority.clone(),
                    data.storage_account_id.clone(),
                    amount_base.clone(),
                    authority.clone(),
                    world_state_view,
                )?;
                transfer_from(
                    token_pair_id.target_asset_id.clone(),
                    authority.clone(),
                    data.storage_account_id.clone(),
                    amount_target.clone(),
                    authority.clone(),
                    world_state_view,
                )?;
                // mint pool_token for sender based on deposited amount
                data.mint_pool_token_with_fee(
                    self.pool_tokens_to,
                    token_pair_id,
                    amount_base,
                    amount_target,
                    world_state_view,
                )?;
                // update pool data
                let liquidity_source =
                    get_liquidity_source_mut(&self.liquidity_source_id, world_state_view)?;
                let _val = mem::replace(expect_xyk_pool_data_mut(liquidity_source)?, data);
                Ok(())
            }
        }

        /// Based on given reserves, desired and minimal amounts to add liquidity, either return
        /// optimal values (needed to preserve reserves proportion) or error if it's not possible
        /// to keep proportion with proposed amounts.
        pub fn get_optimal_deposit_amounts(
            reserve_a: u32,
            reserve_b: u32,
            amount_a_desired: u32,
            amount_b_desired: u32,
            amount_a_min: u32,
            amount_b_min: u32,
        ) -> Result<(u32, u32), String> {
            Ok(if reserve_a == 0u32 && reserve_b == 0u32 {
                (amount_a_desired, amount_b_desired)
            } else {
                let amount_b_optimal = quote(
                    amount_a_desired.clone(),
                    reserve_a.clone(),
                    reserve_b.clone(),
                )?;
                if amount_b_optimal <= amount_b_desired {
                    if !(amount_b_optimal >= amount_b_min) {
                        return Err("insufficient b amount".to_owned());
                    }
                    (amount_a_desired, amount_b_optimal)
                } else {
                    let amount_a_optimal = quote(amount_b_desired.clone(), reserve_b, reserve_a)?;
                    assert!(amount_a_optimal <= amount_a_desired);
                    if !(amount_a_optimal >= amount_a_min) {
                        return Err("insufficient a amount".to_owned());
                    }
                    (amount_a_optimal, amount_b_desired)
                }
            })
        }

        /// Given some amount of an asset and pair reserves, returns an equivalent amount of the other Asset.
        pub fn quote(amount_a: u32, reserve_a: u32, reserve_b: u32) -> Result<u32, String> {
            if !(amount_a > 0) {
                return Err("insufficient amount".to_owned());
            }
            if !(reserve_a > 0 && reserve_b > 0) {
                return Err("insufficient liquidity".to_owned());
            }
            Ok((amount_a * reserve_b) / reserve_a) // calculate amount_b via proportion
        }

        /// Constructor if `RemoveLiquidityFromXYKPool` ISI.
        pub fn remove_liquidity(
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
            pool_tokens_amount: u32,
            base_asset_min_amount: u32,
            target_asset_min_amount: u32,
        ) -> Instruction {
            Instruction::DEX(DEXInstruction::RemoveLiquidityFromXYKPool(
                liquidity_source_id,
                pool_tokens_amount,
                base_asset_min_amount,
                target_asset_min_amount,
            ))
        }

        /// RemoveLiquidity instruction is used to withdraw pair tokens from pool by burning pool tokens.
        pub struct RemoveLiquidity {
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
            pool_tokens_amount: u32,
            base_asset_min_amount: u32,
            target_asset_min_amount: u32,
            pair_tokens_to: <Account as Identifiable>::Id,
        }

        impl RemoveLiquidity {
            /// Constructor of `RemoveLiquidity` instruction.
            pub fn new(
                liquidity_source_id: <LiquiditySource as Identifiable>::Id,
                pool_tokens_amount: u32,
                base_asset_min_amount: u32,
                target_asset_min_amount: u32,
                pair_tokens_to: <Account as Identifiable>::Id,
            ) -> Self {
                RemoveLiquidity {
                    liquidity_source_id,
                    pool_tokens_amount,
                    base_asset_min_amount,
                    target_asset_min_amount,
                    pair_tokens_to,
                }
            }

            /// Core logic of `RemoveLiquidityFromXYKPool` ISI, called by its `execute` function.
            pub fn execute(
                self,
                authority: <Account as Identifiable>::Id,
                world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                let liquidity_source =
                    get_liquidity_source(&self.liquidity_source_id, world_state_view)?;
                let mut data = expect_xyk_pool_data(liquidity_source)?.clone();

                transfer_from(
                    data.pool_token_asset_definition_id.clone(),
                    authority.clone(),
                    data.storage_account_id.clone(),
                    self.pool_tokens_amount,
                    authority.clone(),
                    world_state_view,
                )?;

                let (amount_base, amount_target) = data.burn_pool_token_with_protocol_fee(
                    self.pair_tokens_to.clone(),
                    self.liquidity_source_id.token_pair_id.clone(),
                    self.pool_tokens_amount,
                    world_state_view,
                )?;
                if !(amount_base >= self.base_asset_min_amount) {
                    return Err("insufficient a amount".to_owned());
                }
                if !(amount_target >= self.target_asset_min_amount) {
                    return Err("insufficient b amount".to_owned());
                }
                let liquidity_source =
                    get_liquidity_source_mut(&self.liquidity_source_id, world_state_view)?;
                let _val = mem::replace(expect_xyk_pool_data_mut(liquidity_source)?, data);
                Ok(())
            }
        }

        /// Constructor of `SwapExactTokensForTokensOnXYKPool` ISI.
        pub fn swap_exact_tokens_for_tokens(
            dex_id: <DEX as Identifiable>::Id,
            path: Vec<<AssetDefinition as Identifiable>::Id>,
            amount_in: u32,
            amount_out_min: u32,
        ) -> Instruction {
            Instruction::DEX(DEXInstruction::SwapExactTokensForTokensOnXYKPool(
                dex_id,
                path,
                amount_in,
                amount_out_min,
            ))
        }

        /// SwapExactTokensForTokens instruction is used to exchange input tokens for output tokens via path.
        pub struct SwapExactTokensForTokens<'a> {
            dex_id: <DEX as Identifiable>::Id,
            path: &'a [<AssetDefinition as Identifiable>::Id],
            input_asset_amount: u32,
            output_asset_min_amount: u32,
            output_asset_to: <Account as Identifiable>::Id,
        }

        impl<'a> SwapExactTokensForTokens<'a> {
            /// Consturctor of `SwapExactTokensForTokens` instruction.
            pub fn new(
                dex_id: <DEX as Identifiable>::Id,
                path: &'a [<AssetDefinition as Identifiable>::Id],
                input_asset_amount: u32,
                output_asset_min_amount: u32,
                output_asset_to: <Account as Identifiable>::Id,
            ) -> Self {
                SwapExactTokensForTokens {
                    dex_id,
                    path,
                    input_asset_amount,
                    output_asset_min_amount,
                    output_asset_to,
                }
            }

            /// Core logic of `SwapExactTokensForTokensOnXYKPool` ISI, called by its `execute` function.
            pub fn execute(
                self,
                authority: <Account as Identifiable>::Id,
                world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                let (initial_deposit, amounts) = get_amounts_out(
                    self.dex_id,
                    self.input_asset_amount,
                    self.path,
                    world_state_view,
                )?;
                if !(amounts.last().unwrap().asset_output.amount() >= self.output_asset_min_amount)
                {
                    return Err("insufficient output amount".to_owned());
                }
                swap_tokens_execute(
                    initial_deposit,
                    &amounts,
                    self.output_asset_to.clone(),
                    authority,
                    world_state_view,
                )
            }
        }

        /// Constructor of `SwapTokensForExactTokensOnXYKPool` ISI.
        pub fn swap_tokens_for_exact_tokens(
            dex_id: <DEX as Identifiable>::Id,
            path: Vec<<AssetDefinition as Identifiable>::Id>,
            amount_out: u32,
            amount_in_max: u32,
        ) -> Instruction {
            Instruction::DEX(DEXInstruction::SwapTokensForExactTokensOnXYKPool(
                dex_id,
                path,
                amount_out,
                amount_in_max,
            ))
        }

        /// SwapExactTokensForTokens instruction is used to exchange input tokens for output tokens via path.
        pub struct SwapTokensForExactTokens<'a> {
            dex_id: <DEX as Identifiable>::Id,
            path: &'a [<AssetDefinition as Identifiable>::Id],
            output_asset_amount: u32,
            input_asset_max_amount: u32,
            output_asset_to: <Account as Identifiable>::Id,
        }

        impl<'a> SwapTokensForExactTokens<'a> {
            /// Constructor of `SwapTokensForExactTokens` instruction.
            pub fn new(
                dex_id: <DEX as Identifiable>::Id,
                path: &'a [<AssetDefinition as Identifiable>::Id],
                output_asset_amount: u32,
                input_asset_max_amount: u32,
                output_asset_to: <Account as Identifiable>::Id,
            ) -> Self {
                SwapTokensForExactTokens {
                    dex_id,
                    path,
                    output_asset_amount,
                    input_asset_max_amount,
                    output_asset_to,
                }
            }

            /// Core logic of `SwapTokensForExactTokensOnXYKPool` ISI, called by its `execute` function.
            pub fn execute(
                self,
                authority: <Account as Identifiable>::Id,
                world_state_view: &mut WorldStateView,
            ) -> Result<(), String> {
                let (initial_deposit, amounts) = get_amounts_in(
                    self.dex_id,
                    self.output_asset_amount,
                    self.path,
                    world_state_view,
                )?;
                if !(initial_deposit.input_amount <= self.input_asset_max_amount) {
                    return Err("excessive input amount".to_owned());
                }
                swap_tokens_execute(
                    initial_deposit,
                    &amounts,
                    self.output_asset_to.clone(),
                    authority,
                    world_state_view,
                )
            }
        }

        /// Entry point for SwapTokens-related ISI's.
        ///
        /// # Panics
        /// Panics if `amounts` is less than 1.
        pub fn swap_tokens_execute(
            initial_deposit: InitialDeposit,
            amounts: &[SwapOutput],
            output_tokens_to: <Account as Identifiable>::Id,
            authority: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            let first_pool_storage_account_id = &amounts.first().unwrap().storage_account_id;
            transfer_from(
                initial_deposit.input_token.clone(),
                authority.clone(),
                first_pool_storage_account_id.clone(),
                initial_deposit.input_amount.clone(),
                authority.clone(),
                world_state_view,
            )?;
            swap_all(amounts, output_tokens_to, world_state_view)
        }

        /// Intermediary data to describe single swap in a chain of swaps,
        /// amounts described are assuming input tokens were already deposited
        /// to storage.
        #[derive(Clone, Debug, Encode, Decode, PartialEq)]
        pub struct SwapOutput {
            /// Pool Identifier.
            pub liquidity_source_id: <LiquiditySource as Identifiable>::Id,
            /// Storage Account Identifier.
            storage_account_id: <Account as Identifiable>::Id,
            /// Amount of either Base or Target asset to be transferred to next account in chain.
            pub asset_output: PairAmount,
            /// Amount of either Base or Target asset to be transferred to fee-storage account.
            pub fee_output: PairAmount,
        }

        impl SwapOutput {
            /// Constructor of `SwapOutput` intermediary container.
            pub fn new(
                liquidity_source_id: <LiquiditySource as Identifiable>::Id,
                storage_account_id: <Account as Identifiable>::Id,
                asset_output: PairAmount,
                fee_output: PairAmount,
            ) -> Self {
                SwapOutput {
                    liquidity_source_id,
                    storage_account_id,
                    asset_output,
                    fee_output,
                }
            }
        }

        /// Amount representing direction of amount deduction on pair.
        #[derive(Clone, Debug, PartialEq, Encode, Decode)]
        pub enum PairAmount {
            /// Variant meaning output should be deduced from base token store.
            BaseToken(u32),
            /// Variant meaning output should be deduced from target token store.
            TargetToken(u32),
        }

        impl PairAmount {
            /// Get value of underlying amount without direction.
            pub fn amount(&self) -> u32 {
                match self {
                    PairAmount::BaseToken(amount) | PairAmount::TargetToken(amount) => {
                        amount.clone()
                    }
                }
            }
        }
        /// Intermediary data to describe initial deposit by account
        /// initiating swap.
        #[derive(Debug)]
        pub struct InitialDeposit {
            /// AssetDefinition of token.
            pub input_token: <AssetDefinition as Identifiable>::Id,
            /// Amount to be deposited.
            pub input_amount: u32,
        }

        impl InitialDeposit {
            /// Constructor of `InitialDeposit` struct.
            pub fn new(
                input_token: <AssetDefinition as Identifiable>::Id,
                input_amount: u32,
            ) -> Self {
                InitialDeposit {
                    input_token,
                    input_amount,
                }
            }
        }

        /// Given a path of Tokens, calculate amounts to be swapped pair-wise.
        /// Path of length N (where N>=2) will result in amounts of length N-1.
        pub fn get_amounts_out(
            dex_id: <DEX as Identifiable>::Id,
            amount_in: u32,
            path: &[<AssetDefinition as Identifiable>::Id],
            world_state_view: &WorldStateView,
        ) -> Result<(InitialDeposit, Vec<SwapOutput>), String> {
            if !(path.len() >= 2) {
                return Err("invalid path".to_owned());
            }
            let dex = get_dex(&dex_id.domain_name, world_state_view)?;
            let dex_base_asset_id = &dex.base_asset_id;
            let mut amounts: Vec<SwapOutput> = Vec::new();
            for i in 0..path.len() - 1 {
                let (input_asset_id, output_asset_id) =
                    (path.get(i).unwrap(), path.get(i + 1).unwrap());
                let amount_in = if amounts.is_empty() {
                    amount_in
                } else {
                    amounts.last().unwrap().asset_output.amount()
                };
                let get_xyk_pool =
                    |base_asset_id: &<AssetDefinition as Identifiable>::Id,
                     target_asset_id: &<AssetDefinition as Identifiable>::Id| {
                        let token_pair_id = TokenPairId::new(
                            dex_id.clone(),
                            base_asset_id.clone(),
                            target_asset_id.clone(),
                        );
                        let liquidity_source_id =
                            LiquiditySourceId::new(token_pair_id, LiquiditySourceType::XYKPool);
                        get_liquidity_source(&liquidity_source_id, world_state_view)
                    };
                if input_asset_id == dex_base_asset_id {
                    let liquidity_source = get_xyk_pool(input_asset_id, output_asset_id)?;
                    let pool_data = expect_xyk_pool_data(liquidity_source)?;
                    let (asset_out, fee_out) = get_target_amount_out(
                        amount_in,
                        pool_data.base_asset_reserve,
                        pool_data.target_asset_reserve,
                        pool_data.fee,
                    )?;
                    amounts.push(SwapOutput::new(
                        liquidity_source.id.clone(),
                        pool_data.storage_account_id.clone(),
                        asset_out,
                        fee_out,
                    ));
                } else if output_asset_id == dex_base_asset_id {
                    let liquidity_source = get_xyk_pool(output_asset_id, input_asset_id)?;
                    let pool_data = expect_xyk_pool_data(liquidity_source)?;
                    let (asset_out, fee_out) = get_base_amount_out(
                        amount_in,
                        pool_data.base_asset_reserve,
                        pool_data.target_asset_reserve,
                        pool_data.fee,
                    )?;
                    amounts.push(SwapOutput::new(
                        liquidity_source.id.clone(),
                        pool_data.storage_account_id.clone(),
                        asset_out,
                        fee_out,
                    ));
                } else {
                    return Err("neither of tokens is base asset".to_owned());
                }
            }
            let initial_deposit = InitialDeposit::new(path.first().unwrap().clone(), amount_in);
            Ok((initial_deposit, amounts))
        }

        /// Given amount of input tokens for a pool where input tokens
        /// are base tokens, calculate output amount for recipient and fee amount.
        pub fn get_target_amount_out(
            base_amount_in: u32,
            base_reserve: u32,
            target_reserve: u32,
            fee: u16,
        ) -> Result<(PairAmount, PairAmount), String> {
            if !(base_amount_in > 0) {
                return Err("insufficient input amount".to_owned());
            }
            if !(base_reserve > 0 && target_reserve > 0) {
                return Err("insufficient liquidity".to_owned());
            }
            let fee_amount = base_amount_in as u128 * fee as u128 / MAX_BASIS_POINTS as u128;
            let amount_in_with_fee = base_amount_in as u128 - fee_amount;
            let numerator = amount_in_with_fee * target_reserve as u128;
            let denominator = base_reserve as u128 + amount_in_with_fee;
            Ok((
                PairAmount::TargetToken((numerator / denominator) as u32),
                PairAmount::BaseToken(fee_amount as u32),
            ))
        }

        /// Given amount of input tokens for a pool where output tokens
        /// are base tokens, calculate output amount for recipient and fee amount.
        pub fn get_base_amount_out(
            target_amount_in: u32,
            base_reserve: u32,
            target_reserve: u32,
            fee: u16,
        ) -> Result<(PairAmount, PairAmount), String> {
            if !(target_amount_in > 0) {
                return Err("insufficient input amount".to_owned());
            }
            if !(target_reserve > 0 && base_reserve > 0) {
                return Err("insufficient liquidity".to_owned());
            }
            let numerator = target_amount_in as u128 * base_reserve as u128;
            let denominator = target_reserve as u128 + target_amount_in as u128;
            let amount_out_without_fee = numerator / denominator;
            let fee_amount = amount_out_without_fee * fee as u128 / MAX_BASIS_POINTS as u128;
            Ok((
                PairAmount::BaseToken((amount_out_without_fee - fee_amount) as u32),
                PairAmount::BaseToken(fee_amount as u32),
            ))
        }

        /// Given a path of Tokens, calculate amounts to be swapped pair-wise.
        /// Path of length N (where N>=2) will result in amounts of length N-1.
        pub fn get_amounts_in(
            dex_id: <DEX as Identifiable>::Id,
            mut amount_out: u32,
            path: &[<AssetDefinition as Identifiable>::Id],
            world_state_view: &WorldStateView,
        ) -> Result<(InitialDeposit, Vec<SwapOutput>), String> {
            if !(path.len() >= 2) {
                return Err("invalid path".to_owned());
            }
            let dex = get_dex(&dex_id.domain_name, world_state_view)?;
            let dex_base_asset_id = &dex.base_asset_id;
            let mut amounts: Vec<SwapOutput> = Vec::new();
            for i in (1..path.len()).rev() {
                let (input_asset_id, output_asset_id) =
                    (path.get(i - 1).unwrap(), path.get(i).unwrap());
                let get_xyk_pool =
                    |base_asset_id: &<AssetDefinition as Identifiable>::Id,
                     target_asset_id: &<AssetDefinition as Identifiable>::Id| {
                        let token_pair_id = TokenPairId::new(
                            dex_id.clone(),
                            base_asset_id.clone(),
                            target_asset_id.clone(),
                        );
                        let liquidity_source_id =
                            LiquiditySourceId::new(token_pair_id, LiquiditySourceType::XYKPool);
                        get_liquidity_source(&liquidity_source_id, world_state_view)
                    };
                if input_asset_id == dex_base_asset_id {
                    let liquidity_source = get_xyk_pool(input_asset_id, output_asset_id)?;
                    let pool_data = expect_xyk_pool_data(liquidity_source)?;
                    let (asset_in, asset_out, fee_out) = get_base_amount_in(
                        amount_out,
                        pool_data.base_asset_reserve,
                        pool_data.target_asset_reserve,
                        pool_data.fee,
                    )?;
                    amounts.push(SwapOutput::new(
                        liquidity_source.id.clone(),
                        pool_data.storage_account_id.clone(),
                        asset_out,
                        fee_out,
                    ));
                    amount_out = asset_in;
                } else if output_asset_id == dex_base_asset_id {
                    let liquidity_source = get_xyk_pool(output_asset_id, input_asset_id)?;
                    let pool_data = expect_xyk_pool_data(liquidity_source)?;
                    let (asset_in, asset_out, fee_out) = get_target_amount_in(
                        amount_out,
                        pool_data.base_asset_reserve,
                        pool_data.target_asset_reserve,
                        pool_data.fee,
                    )?;
                    amounts.push(SwapOutput::new(
                        liquidity_source.id.clone(),
                        pool_data.storage_account_id.clone(),
                        asset_out,
                        fee_out,
                    ));
                    amount_out = asset_in;
                } else {
                    return Err("neither of tokens is base asset".to_owned());
                }
            }
            amounts.reverse();
            // last `amount_out = amount_in` implies amount_out contains value for initial deposit
            let initial_deposit = InitialDeposit::new(path.first().unwrap().clone(), amount_out);
            Ok((initial_deposit, amounts))
        }

        /// Given amount of target output tokens for a pool where input tokens
        /// are base tokens, calculate input amount for sender and fee amount.
        pub fn get_base_amount_in(
            target_amount_out: u32,
            base_reserve: u32,
            target_reserve: u32,
            fee: u16,
        ) -> Result<(u32, PairAmount, PairAmount), String> {
            if !(target_amount_out > 0) {
                return Err("insufficient output amount".to_owned());
            }
            if !(base_reserve > 0 && target_reserve > 0) {
                return Err("insufficient liquidity".to_owned());
            }
            if !(target_reserve != target_amount_out) {
                return Err("can't withdraw full reserve".to_owned());
            }
            let numerator = base_reserve as u128 * target_amount_out as u128;
            let denominator = target_reserve as u128 - target_amount_out as u128;
            let base_amount_in_without_fee = numerator / denominator;
            let base_amount_in_with_fee = 1 + base_amount_in_without_fee * MAX_BASIS_POINTS as u128
                / (MAX_BASIS_POINTS as u128 - fee as u128);
            Ok((
                base_amount_in_with_fee as u32,
                PairAmount::TargetToken(target_amount_out),
                PairAmount::BaseToken(
                    (base_amount_in_with_fee - base_amount_in_without_fee) as u32,
                ),
            ))
        }

        /// Given amount of base output tokens for a pool where input tokens
        /// are target tokens, calculate input amount for sender and fee amount.
        pub fn get_target_amount_in(
            base_amount_out: u32,
            base_asset_reserve: u32,
            target_asset_reserve: u32,
            fee: u16,
        ) -> Result<(u32, PairAmount, PairAmount), String> {
            if !(base_amount_out > 0) {
                return Err("insufficient output amount".to_owned());
            }
            if !(target_asset_reserve > 0 && base_asset_reserve > 0) {
                return Err("insufficient liquidity".to_owned());
            }
            if !(base_asset_reserve != base_amount_out) {
                return Err("can't withdraw full reserve".to_owned());
            }
            let base_amount_out_with_fee = base_amount_out as u128 * MAX_BASIS_POINTS as u128
                / (MAX_BASIS_POINTS as u128 - fee as u128);
            let numerator = target_asset_reserve as u128 * base_amount_out_with_fee as u128;
            let denominator = base_asset_reserve as u128 - base_amount_out_with_fee as u128;
            let target_amount_in = (numerator / denominator) + 1;
            Ok((
                target_amount_in as u32,
                PairAmount::BaseToken(base_amount_out),
                PairAmount::BaseToken((base_amount_out_with_fee - base_amount_out as u128) as u32),
            ))
        }

        /// Iterate through the path with according amounts and perform swaps on
        /// each of xyk pools.
        fn swap_all(
            amounts: &[SwapOutput],
            to: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            for i in 0..amounts.len() {
                let next_account = if i < amounts.len() - 1 {
                    amounts.get(i + 1).unwrap().storage_account_id.clone()
                } else {
                    to.clone()
                };
                swap(amounts.get(i).unwrap(), next_account, world_state_view)?
            }
            Ok(())
        }

        /// Assuming that input tokens are already deposited into pool,
        /// withdraw corresponding output tokens.
        fn swap(
            swap_output: &SwapOutput,
            output_tokens_to: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            let token_pair_id = &swap_output.liquidity_source_id.token_pair_id;
            let xyk_pool =
                get_liquidity_source(&swap_output.liquidity_source_id, world_state_view)?;
            let mut pool_data = expect_xyk_pool_data(xyk_pool)?.clone();
            if !(swap_output.asset_output.amount() > 0) {
                return Err("insufficient output amount".to_owned());
            }
            match swap_output.asset_output {
                PairAmount::BaseToken(amount) => {
                    if !(amount < pool_data.base_asset_reserve) {
                        return Err("insufficient liquidity".to_owned());
                    }
                    transfer_from_unchecked(
                        token_pair_id.base_asset_id.clone(),
                        pool_data.storage_account_id.clone(),
                        output_tokens_to.clone(),
                        amount,
                        world_state_view,
                    )?
                }
                PairAmount::TargetToken(amount) => {
                    if !(amount < pool_data.target_asset_reserve) {
                        return Err("insufficient liquidity".to_owned());
                    }
                    transfer_from_unchecked(
                        token_pair_id.target_asset_id.clone(),
                        pool_data.storage_account_id.clone(),
                        output_tokens_to.clone(),
                        amount,
                        world_state_view,
                    )?
                }
            }
            // swap_output.fee_output can be used to redirect fee from returning to pool
            // into e.g. fee-storage account for alternative use.
            let base_balance = get_asset_quantity(
                pool_data.storage_account_id.clone(),
                token_pair_id.base_asset_id.clone(),
                world_state_view,
            )?;
            let target_balance = get_asset_quantity(
                pool_data.storage_account_id.clone(),
                token_pair_id.target_asset_id.clone(),
                world_state_view,
            )?;
            pool_data.update(base_balance, target_balance, world_state_view)?;
            let xyk_pool =
                get_liquidity_source_mut(&swap_output.liquidity_source_id, world_state_view)?;
            let _val = mem::replace(expect_xyk_pool_data_mut(xyk_pool)?, pool_data);
            Ok(())
        }

        /// Set value for fees that are deduced from user tokens during swaps.
        pub fn set_fee_execute(
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
            fee: u16,
            authority: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            let token_pair_id = &liquidity_source_id.token_pair_id;
            PermissionInstruction::CanManageDEX(
                authority,
                Some(token_pair_id.dex_id.domain_name.clone()),
            )
            .execute(world_state_view)?;
            let liquidity_source =
                get_liquidity_source_mut(&liquidity_source_id, world_state_view)?;
            let xyk_pool = expect_xyk_pool_data_mut(liquidity_source)?;
            if fee > MAX_BASIS_POINTS {
                return Err("fee could not be greater than 100 percent".to_owned());
            }
            xyk_pool.fee = fee;
            Ok(())
        }

        /// Set value for protocol fee fraction that is deduced from swap fees.
        pub fn set_protocol_fee_part_execute(
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
            protocol_fee_part: u16,
            authority: <Account as Identifiable>::Id,
            world_state_view: &mut WorldStateView,
        ) -> Result<(), String> {
            let token_pair_id = &liquidity_source_id.token_pair_id;
            PermissionInstruction::CanManageDEX(
                authority,
                Some(token_pair_id.dex_id.domain_name.clone()),
            )
            .execute(world_state_view)?;
            let liquidity_source =
                get_liquidity_source_mut(&liquidity_source_id, world_state_view)?;
            let xyk_pool = expect_xyk_pool_data_mut(liquidity_source)?;
            if protocol_fee_part > MAX_BASIS_POINTS {
                return Err(
                    "protocol fee fraction could not be greater than 100 percent".to_owned(),
                );
            }
            xyk_pool.protocol_fee_part = protocol_fee_part;
            Ok(())
        }
    }

    /// Helper function for performing token transfers.
    fn transfer_from(
        token: <AssetDefinition as Identifiable>::Id,
        from: <Account as Identifiable>::Id,
        to: <Account as Identifiable>::Id,
        value: u32,
        authority: <Account as Identifiable>::Id,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        let asset_id = AssetId::new(token, from.clone());
        AccountInstruction::TransferAsset(from, to, Asset::with_quantity(asset_id, value))
            .execute(authority, world_state_view)
    }

    /// Helper function for minting tokens.
    /// Low-level function, should be called from function which performs important safety checks.
    fn mint_asset_unchecked(
        asset_id: <Asset as Identifiable>::Id,
        quantity: u32,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        world_state_view
            .asset_definition(&asset_id.definition_id)
            .ok_or("Failed to find asset.")?;
        match world_state_view.asset(&asset_id) {
            Some(asset) => {
                asset.quantity += quantity;
            }
            None => world_state_view.add_asset(Asset::with_quantity(asset_id.clone(), quantity)),
        }
        Ok(())
    }

    /// Helper function for burning tokens.
    /// Low-level function, should be called from function which performs important safety checks.
    fn burn_asset_unchecked(
        asset_id: <Asset as Identifiable>::Id,
        quantity: u32,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        world_state_view
            .asset_definition(&asset_id.definition_id)
            .ok_or("Failed to find asset definition.")?;
        match world_state_view.asset(&asset_id) {
            Some(asset) => {
                if quantity > asset.quantity {
                    return Err("Insufficient asset quantity to burn.".to_string());
                }
                asset.quantity -= quantity;
            }
            None => return Err("Account does not contain the asset.".to_string()),
        }
        Ok(())
    }

    /// Helper function for performing asset transfers.
    /// Low-level function, should be called from function which performs important safety checks.
    fn transfer_from_unchecked(
        asset_definition_id: <AssetDefinition as Identifiable>::Id,
        from: <Account as Identifiable>::Id,
        to: <Account as Identifiable>::Id,
        value: u32,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        let asset_id = AssetId::new(asset_definition_id.clone(), from.clone());
        let asset = Asset::with_quantity(asset_id.clone(), value);
        let source = world_state_view
            .account(&from)
            .ok_or("Failed to find accounts.")?
            .assets
            .get_mut(&asset_id)
            .ok_or("Asset's component was not found.")?;
        let quantity_to_transfer = asset.quantity;
        if source.quantity < quantity_to_transfer {
            return Err(format!("Not enough assets: {:?}, {:?}.", source, asset));
        }
        source.quantity -= quantity_to_transfer;
        let transferred_asset = {
            let mut object = asset.clone();
            object.id.account_id = to.clone();
            object
        };

        world_state_view
            .account(&to)
            .ok_or("Failed to find destination account.")?
            .assets
            .entry(transferred_asset.id.clone())
            .and_modify(|asset| asset.quantity += quantity_to_transfer)
            .or_insert(transferred_asset);
        Ok(())
    }

    /// Constructor of `AddTransferPermissionForAccount` ISI
    pub fn add_transfer_permission_for_account(
        asset_definition_id: <AssetDefinition as Identifiable>::Id,
        account_id: <Account as Identifiable>::Id,
    ) -> Instruction {
        Instruction::DEX(DEXInstruction::AddTransferPermissionForAccount(
            asset_definition_id,
            account_id,
        ))
    }

    /// Mint permission for account.
    /// TODO: this is temporary function made for debug purposes, remove when permission minting is elaborated in core
    pub fn add_transfer_permission_for_account_execute(
        asset_definition_id: <AssetDefinition as Identifiable>::Id,
        account_id: <Account as Identifiable>::Id,
        _authority: <Account as Identifiable>::Id,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        let domain_name = account_id.domain_name.clone();
        let domain = get_domain_mut(&domain_name, world_state_view)?;
        let asset_id = AssetId {
            definition_id: permission_asset_definition_id(),
            account_id: account_id.clone(),
        };
        domain
            .accounts
            .get_mut(&account_id)
            .ok_or("failed to find account")?
            .assets
            .entry(asset_id.clone())
            .and_modify(|asset| {
                let permission = Permission::TransferAsset(None, Some(asset_definition_id.clone()));
                if !asset.permissions.origin.contains(&permission) {
                    asset.permissions.origin.push(permission);
                }
            })
            .or_insert(Asset::with_permission(
                asset_id.clone(),
                Permission::TransferAsset(None, Some(asset_definition_id.clone())),
            ));
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::account::query::*;
        use crate::peer::PeerId;
        use crate::query::QueryResult;
        use std::collections::BTreeMap;

        struct TestKit {
            world_state_view: WorldStateView,
            root_account_id: <Account as Identifiable>::Id,
            dex_owner_account_id: <Account as Identifiable>::Id,
            domain_name: <Domain as Identifiable>::Id,
            base_asset_id: <AssetDefinition as Identifiable>::Id,
        }

        impl TestKit {
            pub fn new() -> Self {
                let domain_name = "Soramitsu".to_string();
                let base_asset_id = AssetDefinitionId::new("XOR", &domain_name);
                let key_pair = KeyPair::generate().expect("Failed to generate KeyPair.");
                let mut asset_definitions = BTreeMap::new();
                let mut accounts = BTreeMap::new();

                let permission_asset_definition_id = permission_asset_definition_id();
                asset_definitions.insert(
                    permission_asset_definition_id.clone(),
                    AssetDefinition::new(permission_asset_definition_id.clone()),
                );

                let root_account_id = AccountId::new("root", &domain_name);
                let asset_id = AssetId {
                    definition_id: permission_asset_definition_id.clone(),
                    account_id: root_account_id.clone(),
                };
                let asset = Asset::with_permission(asset_id.clone(), Permission::Anything);
                let mut account = Account::with_signatory(
                    &root_account_id.name,
                    &root_account_id.domain_name,
                    key_pair.public_key.clone(),
                );
                account.assets.insert(asset_id.clone(), asset.clone());
                accounts.insert(root_account_id.clone(), account);

                let key_pair = KeyPair::generate().expect("Failed to generate KeyPair.");
                let dex_owner_account_id = AccountId::new("dex owner", &domain_name);
                let asset_id = AssetId {
                    definition_id: permission_asset_definition_id.clone(),
                    account_id: dex_owner_account_id.clone(),
                };

                let asset = Asset::with_permissions(
                    asset_id.clone(),
                    &[
                        Permission::InitalizeDEX,
                        Permission::ManageDEX(Some(dex_owner_account_id.domain_name.clone())),
                        Permission::RegisterAccount(None),
                        Permission::RegisterAssetDefinition(None),
                    ],
                );
                let mut account = Account::with_signatory(
                    &dex_owner_account_id.name,
                    &dex_owner_account_id.domain_name,
                    key_pair.public_key.clone(),
                );

                account.assets.insert(asset_id.clone(), asset);
                accounts.insert(dex_owner_account_id.clone(), account);

                let domain = Domain {
                    name: domain_name.clone(),
                    accounts,
                    asset_definitions,
                    ..Default::default()
                };
                let mut domains = BTreeMap::new();
                domains.insert(domain_name.clone(), domain);
                let address = "127.0.0.1:8080".to_string();
                let world_state_view = WorldStateView::new(Peer::with_domains(
                    PeerId {
                        address: address.clone(),
                        public_key: key_pair.public_key,
                    },
                    &Vec::new(),
                    domains,
                ));
                TestKit {
                    world_state_view,
                    root_account_id,
                    dex_owner_account_id,
                    domain_name,
                    base_asset_id,
                }
            }

            fn initialize_dex(&mut self) {
                let world_state_view = &mut self.world_state_view;

                println!(
                    "Initializing DEX in domain: {}, with owner: {}, base_asset: {}",
                    &self.domain_name,
                    &self.dex_owner_account_id,
                    &self.base_asset_id.clone()
                );
                // initialize dex in domain
                initialize_dex(
                    &self.domain_name,
                    self.dex_owner_account_id.clone(),
                    self.base_asset_id.clone(),
                )
                .execute(self.dex_owner_account_id.clone(), world_state_view)
                .expect("failed to initialize dex");
            }

            fn register_domain(&mut self, domain_name: &str) {
                println!("Register Domain with name: {}", domain_name,);
                let domain = Domain::new(domain_name.to_owned());
                self.world_state_view.add_domain(domain);
            }

            fn register_asset(&mut self, asset: &str) -> <AssetDefinition as Identifiable>::Id {
                println!("Register Asset with Id: {}", asset);
                let world_state_view = &mut self.world_state_view;
                let asset_definition = AssetDefinition::new(AssetDefinitionId::from(asset));
                let domain = world_state_view
                    .read_domain(&asset_definition.id.domain_name)
                    .expect("domain not found")
                    .clone();

                domain
                    .register_asset(asset_definition.clone())
                    .execute(self.root_account_id.clone(), world_state_view)
                    .expect("failed to register asset");
                asset_definition.id.clone()
            }

            fn create_token_pair(
                &mut self,
                base_asset: &str,
                target_asset: &str,
            ) -> <TokenPair as Identifiable>::Id {
                println!(
                    "Create Token Pair with base asset: {}, target asset: {}",
                    base_asset, target_asset
                );
                let world_state_view = &mut self.world_state_view;
                let asset_definition_a_id = AssetDefinitionId::from(base_asset);
                let asset_definition_b_id = AssetDefinitionId::from(target_asset);

                // register pair for exchange assets
                create_token_pair(
                    asset_definition_a_id.clone(),
                    asset_definition_b_id.clone(),
                    &self.domain_name,
                )
                .execute(self.dex_owner_account_id.clone(), world_state_view)
                .expect("create token pair failed");

                // create resulting token pair id
                let token_pair_id = TokenPairId::new(
                    DEXId::new(&self.domain_name),
                    asset_definition_a_id.clone(),
                    asset_definition_b_id.clone(),
                );
                token_pair_id
            }

            fn mint_asset(&mut self, asset: &str, account: &str, quantity: u32) {
                println!(
                    "Mint Asset with Id: {}, for Account: {}, with quantity: {}",
                    asset, account, quantity
                );
                let asset_definition_id = AssetDefinitionId::from(asset);
                let account_id = AccountId::from(account);
                let asset_id = AssetId::new(asset_definition_id, account_id);
                Mint::new(quantity, asset_id)
                    .execute(self.root_account_id.clone(), &mut self.world_state_view)
                    .expect("mint asset failed");
            }

            fn xyk_pool_create(
                &mut self,
                token_pair_id: <TokenPair as Identifiable>::Id,
            ) -> (
                <LiquiditySource as Identifiable>::Id,
                <Account as Identifiable>::Id,
                <AssetDefinition as Identifiable>::Id,
            ) {
                println!(
                    "Create XYK Pool in Domain: {}, for pair: {}",
                    &token_pair_id.dex_id.domain_name,
                    &token_pair_id.get_symbol()
                );
                xyk_pool::create(token_pair_id.clone())
                    .execute(
                        self.dex_owner_account_id.clone(),
                        &mut self.world_state_view,
                    )
                    .expect("create xyk pool failed");
                let xyk_pool_id =
                    LiquiditySourceId::new(token_pair_id.clone(), LiquiditySourceType::XYKPool);
                let storage_account_id = AccountId::new(
                    &xyk_pool::storage_account_name(&token_pair_id),
                    &self.domain_name,
                );
                let pool_token_asset_definition_id = AssetDefinitionId::new(
                    &xyk_pool::token_asset_name(&token_pair_id),
                    &self.domain_name,
                );
                (
                    xyk_pool_id,
                    storage_account_id,
                    pool_token_asset_definition_id,
                )
            }

            fn create_account(&mut self, account: &str) -> <Account as Identifiable>::Id {
                println!("Create Account with Id: {}", account);
                let world_state_view = &mut self.world_state_view;
                let domain = world_state_view
                    .read_domain(&self.domain_name)
                    .expect("domain not found")
                    .clone();
                let key_pair = KeyPair::generate().expect("Failed to generate KeyPair.");
                let account_id = AccountId::from(account);
                let account = Account::with_signatory(
                    &account_id.name,
                    &account_id.domain_name,
                    key_pair.public_key.clone(),
                );
                domain
                    .register_account(account)
                    .execute(self.root_account_id.clone(), world_state_view)
                    .expect("failed to create account");
                account_id
            }

            fn add_transfer_permission(&mut self, account: &str, asset: &str) {
                println!(
                    "Add transfer permission for Account: {}, to transfer asset: {}",
                    account, asset
                );
                let world_state_view = &mut self.world_state_view;
                let account_id = AccountId::from(account);
                let asset_definition_id = AssetDefinitionId::from(asset);
                add_transfer_permission_for_account(asset_definition_id, account_id)
                    .execute(self.root_account_id.clone(), world_state_view)
                    .expect("failed to add transfer permission");
            }

            fn check_xyk_pool_state(
                &self,
                pool_token_total_supply: u32,
                base_asset_reserve: u32,
                target_asset_reserve: u32,
                k_last: u32,
                liquidity_source_id: &<LiquiditySource as Identifiable>::Id,
            ) {
                println!(
                    "Starting checking XYK Pool state: in domain {} for pair {}",
                    &liquidity_source_id.token_pair_id.dex_id.domain_name,
                    &liquidity_source_id.token_pair_id.get_symbol()
                );
                let liquidity_source =
                    get_liquidity_source(liquidity_source_id, &self.world_state_view).unwrap();
                let pool_data = expect_xyk_pool_data(liquidity_source).unwrap();

                println!("Checking XYK Pool state - pool token total supply: {}, base asset reserve: {}, target asset reserve: {}, k last: {}", 
                    &pool_data.pool_token_total_supply,
                    &pool_data.base_asset_reserve,
                    &pool_data.target_asset_reserve,
                    &pool_data.k_last);
                assert_eq!(pool_data.pool_token_total_supply, pool_token_total_supply);
                assert_eq!(pool_data.base_asset_reserve, base_asset_reserve);
                assert_eq!(pool_data.target_asset_reserve, target_asset_reserve);
                assert_eq!(pool_data.k_last, k_last);
            }

            fn check_xyk_pool_storage_account(
                &self,
                base_asset_balance: u32,
                target_asset_balance: u32,
                liquidity_source_id: &<LiquiditySource as Identifiable>::Id,
            ) {
                println!(
                    "Starting checking XYK Pool Storage: in domain {} for pair {}",
                    &liquidity_source_id.token_pair_id.dex_id.domain_name,
                    &liquidity_source_id.token_pair_id.get_symbol()
                );
                let liquidity_source =
                    get_liquidity_source(liquidity_source_id, &self.world_state_view).unwrap();
                let pool_data = expect_xyk_pool_data(liquidity_source).unwrap();
                if let QueryResult::GetAccount(account_result) =
                    GetAccount::build_request(pool_data.storage_account_id.clone())
                        .query
                        .execute(&self.world_state_view)
                        .expect("failed to query token pair")
                {
                    let storage_base_asset_id = AssetId::new(
                        liquidity_source_id.token_pair_id.base_asset_id.clone(),
                        pool_data.storage_account_id.clone(),
                    );
                    let storage_target_asset_id = AssetId::new(
                        liquidity_source_id.token_pair_id.target_asset_id.clone(),
                        pool_data.storage_account_id.clone(),
                    );
                    let account = account_result.account;
                    let base_asset = account
                        .assets
                        .get(&storage_base_asset_id)
                        .expect("failed to get base asset");
                    let target_asset = account
                        .assets
                        .get(&storage_target_asset_id)
                        .expect("failed to get target asset");
                    println!("Checking XYK Pool Storage - base asset reserve: {}, target asset reserve: {}", 
                        &base_asset.quantity,
                        &target_asset.quantity,
                    );
                    assert_eq!(base_asset.quantity.clone(), base_asset_balance);
                    assert_eq!(target_asset.quantity.clone(), target_asset_balance);
                } else {
                    panic!("wrong enum variant returned for GetAccount");
                }
            }

            fn check_asset_amount(&self, account: &str, asset: &str, amount: u32) {
                let account_id = AccountId::from(account);
                let asset_definition_id = AssetDefinitionId::from(asset);
                let quantity =
                    get_asset_quantity(account_id, asset_definition_id, &self.world_state_view)
                        .unwrap();
                println!(
                    "Checking Asset quantity for Account: {}, of Asset: {}, quantity: {}",
                    account, asset, quantity,
                );
                assert_eq!(quantity, amount);
            }
        }

        #[test]
        fn test_initialize_dex_should_pass() {
            println!("\n\nStarting Test: Initialize DEX");

            let mut testkit = TestKit::new();
            let domain_name = testkit.domain_name.clone();

            // get world state view and dex domain
            let world_state_view = &mut testkit.world_state_view;

            println!(
                "Initializing DEX in domain: {}, with owner: {}, base_asset: {}",
                &domain_name,
                &testkit.dex_owner_account_id,
                &AssetDefinitionId::new("XOR", &domain_name)
            );
            initialize_dex(
                &domain_name,
                testkit.dex_owner_account_id.clone(),
                AssetDefinitionId::new("XOR", &domain_name),
            )
            .execute(testkit.dex_owner_account_id.clone(), world_state_view)
            .expect("failed to initialize dex");

            let dex_query_result =
                get_dex(&domain_name, world_state_view).expect("query dex failed");
            assert_eq!(&dex_query_result.id.domain_name, &domain_name);

            if let QueryResult::GetDEXList(dex_list_result) = GetDEXList::build_request()
                .query
                .execute(world_state_view)
                .expect("failed to query dex list")
            {
                assert_eq!(&dex_list_result.dex_list, &[dex_query_result.clone()]);
                println!("Test Success: new dex initialized");
            } else {
                panic!("wrong enum variant returned for GetDEXList");
            }
        }

        #[test]
        fn test_initialize_dex_should_fail_with_permission_not_found() {
            println!("\n\nStarting Test: Initialize DEX without permission");
            let mut testkit = TestKit::new();
            let domain_name = testkit.domain_name.clone();

            // create dex owner account
            let dex_owner_public_key = KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key;
            let dex_owner_account =
                Account::with_signatory("dex_owner", &domain_name, dex_owner_public_key);

            // get world state view and dex domain
            let world_state_view = &mut testkit.world_state_view;
            let domain = world_state_view
                .domain(&domain_name)
                .expect("domain not found")
                .clone();

            // register dex owner account
            println!("Registering account: {}", &dex_owner_account.id);
            let register_account = domain.register_account(dex_owner_account.clone());
            register_account
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register dex owner account");

            println!(
                "Initializing DEX in domain: {}, with owner: {}, base_asset: {}",
                &domain_name,
                &dex_owner_account.id,
                &AssetDefinitionId::new("XOR", &domain_name)
            );
            assert!(initialize_dex(
                &domain_name,
                dex_owner_account.id.clone(),
                AssetDefinitionId::new("XOR", &domain_name)
            )
            .execute(dex_owner_account.id.clone(), world_state_view)
            .unwrap_err()
            .contains("Error: Permission not found."));

            if let QueryResult::GetDEXList(dex_list_result) = GetDEXList::build_request()
                .query
                .execute(world_state_view)
                .expect("failed to query dex list")
            {
                assert_eq!(&dex_list_result.dex_list, &[]);
                println!("Test Success: dex not initialized");
            } else {
                panic!("wrong enum variant returned for GetDEXList");
            }
        }

        #[test]
        fn test_create_and_delete_token_pair_should_pass() {
            println!("\n\nStarting Test: Create and delete token pair");
            let mut testkit = TestKit::new();
            let domain_name = testkit.domain_name.clone();

            testkit.initialize_dex();
            testkit.register_asset("XOR#Soramitsu");
            testkit.register_domain("Polkadot");
            testkit.register_asset("DOT#Polkadot");
            let token_pair_id = testkit.create_token_pair("XOR#Soramitsu", "DOT#Polkadot");

            let token_pair = query_token_pair(token_pair_id.clone(), &mut testkit.world_state_view)
                .expect("failed to query token pair")
                .clone();
            assert_eq!(&token_pair_id, &token_pair.id);

            if let QueryResult::GetTokenPairList(token_pair_list_result) =
                GetTokenPairList::build_request(domain_name.clone())
                    .query
                    .execute(&mut testkit.world_state_view)
                    .expect("failed to query token pair list")
            {
                assert_eq!(
                    &token_pair_list_result.token_pair_list,
                    &[token_pair.clone()]
                );
                println!("Token pair created");
            } else {
                panic!("wrong enum variant returned for GetTokenPairList");
            }

            if let QueryResult::GetTokenPairCount(token_pair_count_result) =
                GetTokenPairCount::build_request(DEXId::new(&domain_name))
                    .query
                    .execute(&mut testkit.world_state_view)
                    .expect("failed to query token pair count")
            {
                assert_eq!(token_pair_count_result.count, 1);
            } else {
                panic!("wrong token pair count");
            }

            println!("Removing token pair: {}", token_pair_id.get_symbol());
            remove_token_pair(token_pair_id.clone())
                .execute(
                    testkit.dex_owner_account_id.clone(),
                    &mut testkit.world_state_view,
                )
                .expect("remove token pair failed");

            if let QueryResult::GetTokenPairList(token_pair_list_result) =
                GetTokenPairList::build_request(domain_name.clone())
                    .query
                    .execute(&mut testkit.world_state_view)
                    .expect("failed to query token pair list")
            {
                assert!(&token_pair_list_result.token_pair_list.is_empty());
                println!("Token pair removed");
            } else {
                panic!("wrong enum variant returned for GetTokenPairList");
            }

            if let QueryResult::GetTokenPairCount(token_pair_count_result) =
                GetTokenPairCount::build_request(DEXId::new(&domain_name))
                    .query
                    .execute(&mut testkit.world_state_view)
                    .expect("failed to query token pair count")
            {
                assert_eq!(token_pair_count_result.count, 0);
            } else {
                panic!("wrong token pair count");
            }
            println!("Test Success: Token pair created and removed");
        }

        #[test]
        fn test_xyk_pool_create_should_pass() {
            println!("\n\nStarting Test: Create XYK Pool");
            let mut testkit = TestKit::new();

            testkit.initialize_dex();
            testkit.register_asset("XOR#Soramitsu");
            testkit.register_domain("Polkadot");
            testkit.register_asset("DOT#Polkadot");
            let token_pair_id = testkit.create_token_pair("XOR#Soramitsu", "DOT#Polkadot");
            let (xyk_pool_id, storage_account_id, pool_token_asset_definition_id) =
                testkit.xyk_pool_create(token_pair_id.clone());

            let xyk_pool = get_liquidity_source(&xyk_pool_id, &testkit.world_state_view).unwrap();
            let xyk_pool_data = expect_xyk_pool_data(&xyk_pool).unwrap();

            assert_eq!(&storage_account_id, &xyk_pool_data.storage_account_id);
            assert_eq!(
                &pool_token_asset_definition_id,
                &xyk_pool_data.pool_token_asset_definition_id
            );
            assert_eq!(0u32, xyk_pool_data.base_asset_reserve);
            assert_eq!(0u32, xyk_pool_data.target_asset_reserve);
            assert_eq!(0u32, xyk_pool_data.k_last);
            assert_eq!(0u32, xyk_pool_data.pool_token_total_supply);
            assert_eq!(None, xyk_pool_data.fee_to);
            println!("Test Successful: pool created with default values");
        }

        #[test]
        fn test_xyk_pool_add_liquidity_should_pass() {
            println!("\n\nStarting Test: Add liquidity to XYK Pool");
            let mut testkit = TestKit::new();
            testkit.initialize_dex();
            testkit.register_asset("XOR#Soramitsu");
            testkit.register_domain("Polkadot");
            testkit.register_asset("DOT#Polkadot");
            let token_pair_id = testkit.create_token_pair("XOR#Soramitsu", "DOT#Polkadot");
            let (xyk_pool_id, _, pool_token_id) = testkit.xyk_pool_create(token_pair_id.clone());
            let account_id = testkit.create_account("Trader@Soramitsu");
            testkit.add_transfer_permission(
                "Trader@Soramitsu",
                &token_pair_id.base_asset_id.to_string(),
            );
            testkit.add_transfer_permission(
                "Trader@Soramitsu",
                &token_pair_id.target_asset_id.to_string(),
            );
            testkit.mint_asset("XOR#Soramitsu", "Trader@Soramitsu", 5000u32);
            testkit.mint_asset("DOT#Polkadot", "Trader@Soramitsu", 7000u32);

            // add minted tokens to the pool from account
            let desired_base: u32 = 5000;
            let desired_target: u32 = 7000;
            let base_min: u32 = 4000;
            let target_min: u32 = 6000;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_id,
                &xyk_pool_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_id.clone(),
                desired_base,
                desired_target,
                base_min,
                target_min,
            )
            .execute(account_id.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            testkit.check_xyk_pool_state(5916, 5000, 7000, 0, &xyk_pool_id);
            testkit.check_xyk_pool_storage_account(5000, 7000, &xyk_pool_id);
            testkit.check_asset_amount("Trader@Soramitsu", "XOR#Soramitsu", 0);
            testkit.check_asset_amount("Trader@Soramitsu", "DOT#Polkadot", 0);
            testkit.check_asset_amount("Trader@Soramitsu", &pool_token_id.to_string(), 4916);
            println!("Test Successful: liquidity added")
        }

        #[test]
        fn test_xyk_pool_optimal_liquidity_should_pass() {
            // zero reserves return desired amounts
            let (amount_a, amount_b) =
                xyk_pool::get_optimal_deposit_amounts(0, 0, 10000, 5000, 10000, 5000)
                    .expect("failed to get optimal asset amounts");
            assert_eq!(amount_a, 10000);
            assert_eq!(amount_b, 5000);
            // add liquidity with same proportions
            let (amount_a, amount_b) =
                xyk_pool::get_optimal_deposit_amounts(10000, 5000, 10000, 5000, 10000, 5000)
                    .expect("failed to get optimal asset amounts");
            assert_eq!(amount_a, 10000);
            assert_eq!(amount_b, 5000);
            // add liquidity with different proportions
            let (amount_a, amount_b) =
                xyk_pool::get_optimal_deposit_amounts(10000, 5000, 5000, 10000, 0, 0)
                    .expect("failed to get optimal asset amounts");
            assert_eq!(amount_a, 5000);
            assert_eq!(amount_b, 2500);
            // add liquidity `b_optimal>b_desired` branch
            let (amount_a, amount_b) =
                xyk_pool::get_optimal_deposit_amounts(10000, 5000, 5000, 2000, 0, 0)
                    .expect("failed to get optimal asset amounts");
            assert_eq!(amount_a, 4000);
            assert_eq!(amount_b, 2000);
        }

        #[test]
        fn test_xyk_pool_quote_should_pass() {
            let amount_b_optimal =
                xyk_pool::quote(2000, 5000, 10000).expect("failed to calculate proportion");
            assert_eq!(amount_b_optimal, 4000);
            let amount_b_optimal =
                xyk_pool::quote(1, 5000, 10000).expect("failed to calculate proportion");
            assert_eq!(amount_b_optimal, 2);
            let result = xyk_pool::quote(0, 5000, 10000).unwrap_err();
            assert_eq!(result, "insufficient amount");
            let result = xyk_pool::quote(1000, 5000, 0).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            let result = xyk_pool::quote(1000, 0, 10000).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
        }

        #[test]
        fn test_xyk_pool_remove_liquidity_should_pass() {
            println!("\n\nStarting Test: Remove liquidity from XYK Pool");
            let mut testkit = TestKit::new();

            // prepare environment
            testkit.initialize_dex();
            testkit.register_asset("XOR#Soramitsu");
            testkit.register_domain("Polkadot");
            testkit.register_asset("DOT#Polkadot");
            let token_pair_id = testkit.create_token_pair("XOR#Soramitsu", "DOT#Polkadot");
            let (xyk_pool_id, _, pool_token_id) = testkit.xyk_pool_create(token_pair_id.clone());
            let account_id = testkit.create_account("Trader@Soramitsu");
            testkit.add_transfer_permission("Trader@Soramitsu", &pool_token_id.to_string());
            testkit.add_transfer_permission("Trader@Soramitsu", "XOR#Soramitsu");
            testkit.add_transfer_permission("Trader@Soramitsu", "DOT#Polkadot");
            testkit.mint_asset("XOR#Soramitsu", "Trader@Soramitsu", 5000u32);
            testkit.mint_asset("DOT#Polkadot", "Trader@Soramitsu", 7000u32);

            // add minted tokens to the pool from account
            let desired_base: u32 = 5000;
            let desired_target: u32 = 7000;
            let base_min: u32 = 4000;
            let target_min: u32 = 6000;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_id,
                &xyk_pool_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_id.clone(),
                desired_base,
                desired_target,
                base_min,
                desired_target,
            )
            .execute(account_id.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            // burn minted pool token to receive pool tokens back
            let pool_tokens: u32 = 4916;
            let base_min: u32 = 0;
            let target_min: u32 = 0;
            println!("Removing liquidity: domain {}, pair {}, pool tokens {}, min received base {}, min received target {}",
                &xyk_pool_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_id.token_pair_id.get_symbol(),
                pool_tokens, base_min, target_min);
            xyk_pool::remove_liquidity(xyk_pool_id.clone(), pool_tokens, base_min, target_min)
                .execute(account_id.clone(), &mut testkit.world_state_view)
                .expect("remove liquidity failed");

            testkit.check_xyk_pool_state(1000, 846, 1184, 0, &xyk_pool_id);
            testkit.check_xyk_pool_storage_account(846, 1184, &xyk_pool_id);
            testkit.check_asset_amount("Trader@Soramitsu", "XOR#Soramitsu", 4154);
            testkit.check_asset_amount("Trader@Soramitsu", "DOT#Polkadot", 5816);
            testkit.check_asset_amount("Trader@Soramitsu", &pool_token_id.to_string(), 0);
            println!("Test Successful: liquidity removed from pool");
        }

        #[test]
        fn test_xyk_pool_swap_assets_in_should_pass() {
            println!("\n\nStarting Test: Swap assets on XYK Pool with desired input amount");
            let mut testkit = TestKit::new();

            // prepare environment
            testkit.initialize_dex();
            testkit.register_asset("XOR#Soramitsu");
            testkit.register_domain("Polkadot");
            testkit.register_asset("DOT#Polkadot");
            let token_pair_id = testkit.create_token_pair("XOR#Soramitsu", "DOT#Polkadot");
            let (xyk_pool_id, _, pool_token_id) = testkit.xyk_pool_create(token_pair_id.clone());
            let account_id = testkit.create_account("Trader@Soramitsu");
            testkit.add_transfer_permission("Trader@Soramitsu", &pool_token_id.to_string());
            testkit.add_transfer_permission("Trader@Soramitsu", "XOR#Soramitsu");
            testkit.add_transfer_permission("Trader@Soramitsu", "DOT#Polkadot");
            testkit.mint_asset("XOR#Soramitsu", "Trader@Soramitsu", 7000u32);
            testkit.mint_asset("DOT#Polkadot", "Trader@Soramitsu", 7000u32);

            // add minted tokens to the pool from account
            let desired_base: u32 = 5000;
            let desired_target: u32 = 7000;
            let base_min: u32 = 4000;
            let target_min: u32 = 6000;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_id,
                &xyk_pool_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_id.clone(),
                desired_base,
                desired_target,
                base_min,
                target_min,
            )
            .execute(account_id.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            let amount_in: u32 = 2000;
            let amount_out_min: u32 = 0;
            println!(
                "Swap exact tokens for tokens: account {}, desired input {}, min received {}",
                &account_id, amount_in, amount_out_min
            );
            xyk_pool::swap_exact_tokens_for_tokens(
                DEXId::new(&testkit.domain_name),
                vec![
                    token_pair_id.base_asset_id.clone(),
                    token_pair_id.target_asset_id.clone(),
                ],
                amount_in,
                amount_out_min,
            )
            .execute(account_id.clone(), &mut testkit.world_state_view)
            .expect("swap exact tokens for tokens failed");

            testkit.check_xyk_pool_state(5916, 7000, 5005, 0, &xyk_pool_id);
            testkit.check_xyk_pool_storage_account(7000, 5005, &xyk_pool_id);
            testkit.check_asset_amount("Trader@Soramitsu", "XOR#Soramitsu", 0);
            testkit.check_asset_amount("Trader@Soramitsu", "DOT#Polkadot", 1995);
            testkit.check_asset_amount("Trader@Soramitsu", &pool_token_id.to_string(), 4916);
            println!("Test Successful: swap performed");
        }

        #[test]
        fn test_xyk_pool_get_target_amount_out_should_pass() {
            // regular input
            let (amount_out, fee_out) =
                xyk_pool::get_target_amount_out(2000, 5000, 5000, 30).unwrap();
            assert_eq!(amount_out, xyk_pool::PairAmount::TargetToken(1425));
            assert_eq!(fee_out, xyk_pool::PairAmount::BaseToken(6));
            // zero inputs
            let result = xyk_pool::get_target_amount_out(0, 5000, 7000, 30).unwrap_err();
            assert_eq!(result, "insufficient input amount");
            let result = xyk_pool::get_target_amount_out(2000, 0, 7000, 30).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            let result = xyk_pool::get_target_amount_out(2000, 5000, 0, 30).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            // max values
            let (amount_out, fee_out) =
                xyk_pool::get_target_amount_out(500000, std::u32::MAX, std::u32::MAX, 30).unwrap();
            assert_eq!(amount_out, xyk_pool::PairAmount::TargetToken(498442));
            assert_eq!(fee_out, xyk_pool::PairAmount::BaseToken(1500));
            let (amount_out, fee_out) =
                xyk_pool::get_target_amount_out(250000, std::u32::MAX / 2, std::u32::MAX / 2, 30)
                    .unwrap();
            assert_eq!(amount_out, xyk_pool::PairAmount::TargetToken(249221));
            assert_eq!(fee_out, xyk_pool::PairAmount::BaseToken(750));
            let (amount_out, fee_out) =
                xyk_pool::get_target_amount_out(std::u32::MAX, std::u32::MAX, std::u32::MAX, 30)
                    .unwrap();
            assert_eq!(amount_out, xyk_pool::PairAmount::TargetToken(2144257583));
            assert_eq!(fee_out, xyk_pool::PairAmount::BaseToken(12884901));
        }

        #[test]
        fn test_xyk_pool_get_base_amount_out_should_pass() {
            // regular input
            let (amount_out, fee_out) =
                xyk_pool::get_base_amount_out(2000, 5000, 5000, 30).unwrap();
            assert_eq!(amount_out, xyk_pool::PairAmount::BaseToken(1424));
            assert_eq!(fee_out, xyk_pool::PairAmount::BaseToken(4));
            // zero inputs
            let result = xyk_pool::get_base_amount_out(0, 5000, 7000, 30).unwrap_err();
            assert_eq!(result, "insufficient input amount");
            let result = xyk_pool::get_base_amount_out(2000, 0, 7000, 30).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            let result = xyk_pool::get_base_amount_out(2000, 5000, 0, 30).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            // max values
            let (amount_out, fee_out) =
                xyk_pool::get_base_amount_out(500000, std::u32::MAX, std::u32::MAX, 30).unwrap();
            assert_eq!(amount_out, xyk_pool::PairAmount::BaseToken(498442));
            assert_eq!(fee_out, xyk_pool::PairAmount::BaseToken(1499));
            let (amount_out, fee_out) =
                xyk_pool::get_base_amount_out(250000, std::u32::MAX / 2, std::u32::MAX / 2, 30)
                    .unwrap();
            assert_eq!(amount_out, xyk_pool::PairAmount::BaseToken(249221));
            assert_eq!(fee_out, xyk_pool::PairAmount::BaseToken(749));
            let (amount_out, fee_out) =
                xyk_pool::get_base_amount_out(std::u32::MAX, std::u32::MAX, std::u32::MAX, 30)
                    .unwrap();
            assert_eq!(amount_out, xyk_pool::PairAmount::BaseToken(2141041197));
            assert_eq!(fee_out, xyk_pool::PairAmount::BaseToken(6442450));
        }

        #[test]
        fn test_xyk_pool_swap_assets_out_should_pass() {
            println!("\n\nStarting Test: Swap assets on XYK Pool with desired output amount");
            let mut testkit = TestKit::new();

            // prepare environment
            testkit.initialize_dex();
            testkit.register_asset("XOR#Soramitsu");
            testkit.register_domain("Polkadot");
            testkit.register_asset("DOT#Polkadot");
            let token_pair_id = testkit.create_token_pair("XOR#Soramitsu", "DOT#Polkadot");
            let (xyk_pool_id, _, pool_token_id) = testkit.xyk_pool_create(token_pair_id.clone());
            let account_id = testkit.create_account("Trader@Soramitsu");
            testkit.add_transfer_permission("Trader@Soramitsu", &pool_token_id.to_string());
            testkit.add_transfer_permission("Trader@Soramitsu", "XOR#Soramitsu");
            testkit.add_transfer_permission("Trader@Soramitsu", "DOT#Polkadot");
            testkit.mint_asset("XOR#Soramitsu", "Trader@Soramitsu", 7000u32);
            testkit.mint_asset("DOT#Polkadot", "Trader@Soramitsu", 7000u32);

            // add minted tokens to the pool from account
            let desired_base: u32 = 5000;
            let desired_target: u32 = 7000;
            let base_min: u32 = 4000;
            let target_min: u32 = 6000;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_id,
                &xyk_pool_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_id.clone(),
                desired_base,
                desired_target,
                base_min,
                target_min,
            )
            .execute(account_id.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            let amount_out: u32 = 1995;
            let amount_in_max: u32 = std::u32::MAX;
            println!(
                "Swap tokens for exact tokens: account {}, desired output {}, max spent {}",
                &account_id, amount_out, amount_in_max
            );
            xyk_pool::swap_tokens_for_exact_tokens(
                DEXId::new(&testkit.domain_name),
                vec![
                    token_pair_id.base_asset_id.clone(),
                    token_pair_id.target_asset_id.clone(),
                ],
                amount_out,
                amount_in_max,
            )
            .execute(account_id.clone(), &mut testkit.world_state_view)
            .expect("swap exact tokens for tokens failed");

            testkit.check_xyk_pool_state(5916, 6999, 5005, 0, &xyk_pool_id);
            testkit.check_xyk_pool_storage_account(6999, 5005, &xyk_pool_id);
            testkit.check_asset_amount("Trader@Soramitsu", "XOR#Soramitsu", 1);
            testkit.check_asset_amount("Trader@Soramitsu", "DOT#Polkadot", 1995);
            testkit.check_asset_amount("Trader@Soramitsu", &pool_token_id.to_string(), 4916);
            println!("Test Successful: swap performed");
        }

        #[test]
        fn test_xyk_pool_get_base_amount_in_should_pass() {
            // regular input
            let (amount_in, amount_out, fee) =
                xyk_pool::get_base_amount_in(2000, 5000, 5000, 30).unwrap();
            assert_eq!(amount_in, 3344);
            assert_eq!(amount_out, xyk_pool::PairAmount::TargetToken(2000));
            assert_eq!(fee, xyk_pool::PairAmount::BaseToken(11));
            // zero inputs
            let result = xyk_pool::get_base_amount_in(0, 5000, 7000, 30).unwrap_err();
            assert_eq!(result, "insufficient output amount");
            let result = xyk_pool::get_base_amount_in(2000, 0, 7000, 30).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            let result = xyk_pool::get_base_amount_in(2000, 5000, 0, 30).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            // max values
            let (amount_in, amount_out, fee) =
                xyk_pool::get_base_amount_in(500000, std::u32::MAX, std::u32::MAX, 30).unwrap();
            assert_eq!(amount_in, 501563);
            assert_eq!(amount_out, xyk_pool::PairAmount::TargetToken(500000));
            assert_eq!(fee, xyk_pool::PairAmount::BaseToken(1505));
            let (amount_in, amount_out, fee) =
                xyk_pool::get_base_amount_in(250000, std::u32::MAX / 2, std::u32::MAX / 2, 30)
                    .unwrap();
            assert_eq!(amount_in, 250782);
            assert_eq!(amount_out, xyk_pool::PairAmount::TargetToken(250000));
            assert_eq!(fee, xyk_pool::PairAmount::BaseToken(753));
            let result =
                xyk_pool::get_base_amount_in(std::u32::MAX, std::u32::MAX, std::u32::MAX, 30)
                    .unwrap_err();
            assert_eq!(result, "can't withdraw full reserve");
        }

        #[test]
        fn test_xyk_pool_get_target_amount_in_should_pass() {
            // regular input
            let (amount_in, amount_out, fee) =
                xyk_pool::get_target_amount_in(2000, 5000, 5000, 30).unwrap();
            assert_eq!(amount_in, 3351);
            assert_eq!(amount_out, xyk_pool::PairAmount::BaseToken(2000));
            assert_eq!(fee, xyk_pool::PairAmount::BaseToken(6));
            // zero inputs
            let result = xyk_pool::get_target_amount_in(0, 5000, 7000, 30).unwrap_err();
            assert_eq!(result, "insufficient output amount");
            let result = xyk_pool::get_target_amount_in(2000, 0, 7000, 30).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            let result = xyk_pool::get_target_amount_in(2000, 5000, 0, 30).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            // max values
            let (amount_in, amount_out, fee) =
                xyk_pool::get_target_amount_in(500000, std::u32::MAX, std::u32::MAX, 30).unwrap();
            assert_eq!(amount_in, 501563);
            assert_eq!(amount_out, xyk_pool::PairAmount::BaseToken(500000));
            assert_eq!(fee, xyk_pool::PairAmount::BaseToken(1504));
            let (amount_in, amount_out, fee) =
                xyk_pool::get_target_amount_in(250000, std::u32::MAX / 2, std::u32::MAX / 2, 30)
                    .unwrap();
            assert_eq!(amount_in, 250782);
            assert_eq!(amount_out, xyk_pool::PairAmount::BaseToken(250000));
            assert_eq!(fee, xyk_pool::PairAmount::BaseToken(752));
            let result =
                xyk_pool::get_target_amount_in(std::u32::MAX, std::u32::MAX, std::u32::MAX, 30)
                    .unwrap_err();
            assert_eq!(result, "can't withdraw full reserve");
        }

        #[test]
        fn test_xyk_pool_two_liquidity_providers_one_trader_should_pass() {
            println!("\n\nStarting Test: Usecase with two liquidity providers and one trader");
            let mut testkit = TestKit::new();
            testkit.initialize_dex();
            testkit.register_asset("XOR#Soramitsu");
            testkit.register_domain("Polkadot");
            testkit.register_asset("DOT#Polkadot");
            testkit.register_domain("Kusama");
            testkit.register_asset("KSM#Kusama");
            let token_pair_a_id = testkit.create_token_pair("XOR#Soramitsu", "DOT#Polkadot");
            let token_pair_b_id = testkit.create_token_pair("XOR#Soramitsu", "KSM#Kusama");
            let (xyk_pool_a_id, _, pool_token_a_id) =
                testkit.xyk_pool_create(token_pair_a_id.clone());
            let (xyk_pool_b_id, _, pool_token_b_id) =
                testkit.xyk_pool_create(token_pair_b_id.clone());
            let account_a_id = testkit.create_account("User A@Soramitsu");
            testkit.add_transfer_permission("User A@Soramitsu", &pool_token_a_id.to_string());
            testkit.add_transfer_permission("User A@Soramitsu", &pool_token_b_id.to_string());
            testkit.add_transfer_permission("User A@Soramitsu", "XOR#Soramitsu");
            testkit.add_transfer_permission("User A@Soramitsu", "DOT#Polkadot");
            testkit.add_transfer_permission("User A@Soramitsu", "KSM#Kusama");
            testkit.mint_asset("XOR#Soramitsu", "User A@Soramitsu", 12000u32);
            testkit.mint_asset("DOT#Polkadot", "User A@Soramitsu", 4000u32);
            testkit.mint_asset("KSM#Kusama", "User A@Soramitsu", 3000u32);
            let account_b_id = testkit.create_account("User B@Soramitsu");
            testkit.add_transfer_permission("User B@Soramitsu", &pool_token_a_id.to_string());
            testkit.add_transfer_permission("User B@Soramitsu", "XOR#Soramitsu");
            testkit.add_transfer_permission("User B@Soramitsu", "DOT#Polkadot");
            testkit.mint_asset("XOR#Soramitsu", "User B@Soramitsu", 500u32);
            testkit.mint_asset("DOT#Polkadot", "User B@Soramitsu", 500u32);
            let account_c_id = testkit.create_account("User C@Soramitsu");
            testkit.add_transfer_permission("User C@Soramitsu", "KSM#Kusama");
            testkit.mint_asset("KSM#Kusama", "User C@Soramitsu", 2000u32);

            testkit.check_xyk_pool_state(0, 0, 0, 0, &xyk_pool_a_id);
            testkit.check_xyk_pool_state(0, 0, 0, 0, &xyk_pool_b_id);
            testkit.check_asset_amount("User A@Soramitsu", "XOR#Soramitsu", 12000);
            testkit.check_asset_amount("User A@Soramitsu", "DOT#Polkadot", 4000);
            testkit.check_asset_amount("User A@Soramitsu", "KSM#Kusama", 3000);
            testkit.check_asset_amount("User B@Soramitsu", "XOR#Soramitsu", 500);
            testkit.check_asset_amount("User B@Soramitsu", "DOT#Polkadot", 500);
            testkit.check_asset_amount("User C@Soramitsu", "KSM#Kusama", 2000);

            let desired_base: u32 = 6000;
            let desired_target: u32 = 4000;
            let base_min: u32 = 0;
            let target_min: u32 = 0;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_a_id,
                &xyk_pool_a_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_a_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_a_id.clone(),
                desired_base,
                desired_target,
                base_min,
                target_min,
            )
            .execute(account_a_id.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            let desired_base: u32 = 6000;
            let desired_target: u32 = 3000;
            let base_min: u32 = 0;
            let target_min: u32 = 0;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_a_id,
                &xyk_pool_b_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_b_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_b_id.clone(),
                desired_base,
                desired_target,
                base_min,
                target_min,
            )
            .execute(account_a_id.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            let desired_base: u32 = 500;
            let desired_target: u32 = 500;
            let base_min: u32 = 0;
            let target_min: u32 = 0;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_b_id,
                &xyk_pool_a_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_a_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_a_id.clone(),
                desired_base,
                desired_target,
                base_min,
                target_min,
            )
            .execute(account_b_id.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            let amount_in: u32 = 2000;
            let amount_out_min: u32 = 0u32;
            println!(
                "Swap exact tokens for tokens: account {}, desired input {}, min received {}",
                &account_c_id, amount_in, amount_out_min
            );
            xyk_pool::swap_exact_tokens_for_tokens(
                DEXId::new("Soramitsu"),
                vec![
                    AssetDefinitionId::from("KSM#Kusama"),
                    AssetDefinitionId::from("XOR#Soramitsu"),
                    AssetDefinitionId::from("DOT#Polkadot"),
                ],
                amount_in,
                amount_out_min,
            )
            .execute(account_c_id.clone(), &mut testkit.world_state_view)
            .expect("swap exact tokens for tokens failed");

            let pool_tokens: u32 = 407;
            let base_min: u32 = 0;
            let target_min: u32 = 0;
            println!("Removing liquidity: domain {}, pair {}, pool tokens {}, min received base {}, min received target {}",
                &xyk_pool_b_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_b_id.token_pair_id.get_symbol(),
                pool_tokens, base_min, target_min);
            xyk_pool::remove_liquidity(xyk_pool_a_id.clone(), pool_tokens, base_min, target_min)
                .execute(account_b_id.clone(), &mut testkit.world_state_view)
                .expect("add liquidity failed");

            testkit.check_xyk_pool_state(4898, 8211, 2927, 0, &xyk_pool_a_id);
            testkit.check_xyk_pool_state(4242, 3607, 5000, 0, &xyk_pool_b_id);
            testkit.check_xyk_pool_storage_account(8211, 2927, &xyk_pool_a_id);
            testkit.check_xyk_pool_storage_account(3607, 5000, &xyk_pool_b_id);
            testkit.check_asset_amount("User A@Soramitsu", "XOR#Soramitsu", 0);
            testkit.check_asset_amount("User A@Soramitsu", "DOT#Polkadot", 0);
            testkit.check_asset_amount("User A@Soramitsu", "KSM#Kusama", 0);
            testkit.check_asset_amount("User A@Soramitsu", &pool_token_a_id.to_string(), 3898);
            testkit.check_asset_amount("User A@Soramitsu", &pool_token_b_id.to_string(), 3242);
            testkit.check_asset_amount("User B@Soramitsu", "XOR#Soramitsu", 682);
            testkit.check_asset_amount("User B@Soramitsu", "DOT#Polkadot", 410);
            testkit.check_asset_amount("User B@Soramitsu", &pool_token_a_id.to_string(), 0);
            testkit.check_asset_amount("User C@Soramitsu", "KSM#Kusama", 0);
            testkit.check_asset_amount("User C@Soramitsu", "DOT#Polkadot", 1163);
            println!("Test Successful: usecase passed");
        }

        #[test]
        fn test_xyk_pool_get_price_should_pass() {
            println!("\n\nStarting Test: Get price of tokens on XYK Pool");
            let mut testkit = TestKit::new();
            testkit.initialize_dex();
            testkit.register_asset("XOR#Soramitsu");
            testkit.register_domain("Polkadot");
            testkit.register_asset("DOT#Polkadot");
            let token_pair_id = testkit.create_token_pair("XOR#Soramitsu", "DOT#Polkadot");
            let (xyk_pool_id, ..) = testkit.xyk_pool_create(token_pair_id.clone());
            let account_id = testkit.create_account("Trader@Soramitsu");
            testkit.add_transfer_permission(
                "Trader@Soramitsu",
                &token_pair_id.base_asset_id.to_string(),
            );
            testkit.add_transfer_permission(
                "Trader@Soramitsu",
                &token_pair_id.target_asset_id.to_string(),
            );
            testkit.mint_asset("XOR#Soramitsu", "Trader@Soramitsu", 5000u32);
            testkit.mint_asset("DOT#Polkadot", "Trader@Soramitsu", 7000u32);

            // add minted tokens to the pool from account
            let desired_base: u32 = 5000;
            let desired_target: u32 = 7000;
            let base_min: u32 = 4000;
            let target_min: u32 = 6000;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_id,
                &xyk_pool_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_id.clone(),
                desired_base,
                desired_target,
                base_min,
                target_min,
            )
            .execute(account_id.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            let path = vec![
                AssetDefinitionId::from("XOR#Soramitsu"),
                AssetDefinitionId::from("DOT#Polkadot"),
            ];
            if let QueryResult::GetPriceForInputTokensOnXYKPool(result) =
                GetPriceForInputTokensOnXYKPool::build_request(
                    DEXId::new(&testkit.domain_name),
                    path.clone(),
                    500,
                )
                .query
                .execute(&mut testkit.world_state_view)
                .expect("failed to get price")
            {
                assert_eq!(result.input_amount, 500);
                assert_eq!(result.output_amount, 635);
                assert_eq!(result.amounts.last().unwrap().fee_output.amount(), 1);
                println!(
                    "Got price: for path {:?}, requested input amount {}, received output amount {}",
                    &path, &result.input_amount, &result.output_amount
                );
            } else {
                panic!("wrong enum variant");
            }
            if let QueryResult::GetPriceForOutputTokensOnXYKPool(result) =
                GetPriceForOutputTokensOnXYKPool::build_request(
                    DEXId::new(&testkit.domain_name),
                    path.clone(),
                    635,
                )
                .query
                .execute(&mut testkit.world_state_view)
                .expect("failed to get price")
            {
                assert_eq!(result.input_amount, 500);
                assert_eq!(result.output_amount, 635);
                assert_eq!(result.amounts.last().unwrap().fee_output.amount(), 2);
                println!(
                    "Got price: for path {:?}, requested output amount {}, received input amount {}",
                    &path, &result.input_amount, &result.output_amount
                );
            } else {
                panic!("wrong enum variant");
            }
            let path = vec![
                AssetDefinitionId::from("DOT#Polkadot"),
                AssetDefinitionId::from("XOR#Soramitsu"),
            ];
            if let QueryResult::GetPriceForInputTokensOnXYKPool(result) =
                GetPriceForInputTokensOnXYKPool::build_request(
                    DEXId::new(&testkit.domain_name),
                    path.clone(),
                    780,
                )
                .query
                .execute(&mut testkit.world_state_view)
                .expect("failed to get price")
            {
                assert_eq!(result.input_amount, 780);
                assert_eq!(result.output_amount, 500);
                assert_eq!(result.amounts.last().unwrap().fee_output.amount(), 1);
                println!(
                    "Got price: for path {:?}, requested input amount {}, received output amount {}",
                    &path, &result.input_amount, &result.output_amount
                );
            } else {
                panic!("wrong enum variant");
            }
            if let QueryResult::GetPriceForOutputTokensOnXYKPool(result) =
                GetPriceForOutputTokensOnXYKPool::build_request(
                    DEXId::new(&testkit.domain_name),
                    path.clone(),
                    500,
                )
                .query
                .execute(&mut testkit.world_state_view)
                .expect("failed to get price")
            {
                assert_eq!(result.input_amount, 780);
                assert_eq!(result.output_amount, 500);
                assert_eq!(result.amounts.last().unwrap().fee_output.amount(), 1);
                println!(
                    "Got price: for path {:?}, requested output amount {}, received input amount {}",
                    &path, &result.input_amount, &result.output_amount
                );
            } else {
                panic!("wrong enum variant");
            }
            println!("Test Successful: prices match")
        }

        #[test]
        fn test_xyk_pool_get_owned_liquidity_should_pass() {
            println!("\n\n Starting Test: Get owned liquidity on XYK Pool");
            let mut testkit = TestKit::new();
            testkit.initialize_dex();
            testkit.register_asset("XOR#Soramitsu");
            testkit.register_domain("Polkadot");
            testkit.register_asset("DOT#Polkadot");
            let token_pair_id = testkit.create_token_pair("XOR#Soramitsu", "DOT#Polkadot");
            let (xyk_pool_id, ..) = testkit.xyk_pool_create(token_pair_id.clone());
            // liquidity provider 1
            let account_id_a = testkit.create_account("LP1@Soramitsu");
            testkit
                .add_transfer_permission("LP1@Soramitsu", &token_pair_id.base_asset_id.to_string());
            testkit.add_transfer_permission(
                "LP1@Soramitsu",
                &token_pair_id.target_asset_id.to_string(),
            );
            testkit.mint_asset("XOR#Soramitsu", "LP1@Soramitsu", 5000u32);
            testkit.mint_asset("DOT#Polkadot", "LP1@Soramitsu", 7000u32);
            // liquidity provider 2
            let account_id_b = testkit.create_account("LP2@Soramitsu");
            testkit
                .add_transfer_permission("LP2@Soramitsu", &token_pair_id.base_asset_id.to_string());
            testkit.add_transfer_permission(
                "LP2@Soramitsu",
                &token_pair_id.target_asset_id.to_string(),
            );
            testkit.mint_asset("XOR#Soramitsu", "LP2@Soramitsu", 5000u32);
            testkit.mint_asset("DOT#Polkadot", "LP2@Soramitsu", 7000u32);

            // add minted tokens to the pool from accounts
            let desired_base: u32 = 5000;
            let desired_target: u32 = 7000;
            let base_min: u32 = 4000;
            let target_min: u32 = 6000;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_id_a,
                &xyk_pool_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_id.clone(),
                desired_base,
                desired_target,
                base_min,
                target_min,
            )
            .execute(account_id_a.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            let desired_base: u32 = 5000;
            let desired_target: u32 = 7000;
            let base_min: u32 = 4000;
            let target_min: u32 = 6000;
            println!("Adding liquidity: account {}, domain {}, pair {}, desired base {}, desired target {}, min base {}, min target {}",
                &account_id_b,
                &xyk_pool_id.token_pair_id.dex_id.domain_name,
                &xyk_pool_id.token_pair_id.get_symbol(),
                desired_base, desired_target, base_min, target_min);
            xyk_pool::add_liquidity(
                xyk_pool_id.clone(),
                desired_base,
                desired_target,
                base_min,
                target_min,
            )
            .execute(account_id_b.clone(), &mut testkit.world_state_view)
            .expect("add liquidity failed");

            if let QueryResult::GetOwnedLiquidityOnXYKPool(result) =
                GetOwnedLiquidityOnXYKPool::build_request(
                    xyk_pool_id.clone(),
                    account_id_a.clone(),
                    None,
                )
                .query
                .execute(&testkit.world_state_view)
                .expect("failed to get owned liquidity")
            {
                // amounts are less for initial provider due to minimum liquidity
                assert_eq!(result.base_asset_amount, 4154);
                assert_eq!(result.target_asset_amount, 5816);
                assert_eq!(result.pool_tokens_after_withdrawal, 0);
                println!("Check owned liquidity: account {}, owns {} of base and {} of target on xyk pool {} in domain {}",
                    &account_id_a, result.base_asset_amount, result.target_asset_amount, &xyk_pool_id.token_pair_id.get_symbol(), &xyk_pool_id.token_pair_id.dex_id.domain_name
                );
            } else {
                panic!("wrong enum variant");
            }
            if let QueryResult::GetOwnedLiquidityOnXYKPool(result) =
                GetOwnedLiquidityOnXYKPool::build_request(
                    xyk_pool_id.clone(),
                    account_id_b.clone(),
                    None,
                )
                .query
                .execute(&testkit.world_state_view)
                .expect("failed to get owned liquidity")
            {
                assert_eq!(result.base_asset_amount, 5000);
                assert_eq!(result.target_asset_amount, 7000);
                assert_eq!(result.pool_tokens_after_withdrawal, 0);
                println!("Check owned liquidity: account {}, owns {} of base and {} of target on xyk pool {} in domain {}",
                    &account_id_b, result.base_asset_amount, result.target_asset_amount, &xyk_pool_id.token_pair_id.get_symbol(), &xyk_pool_id.token_pair_id.dex_id.domain_name
                );
            } else {
                panic!("wrong enum variant");
            }
            println!("Test Successful: data received");
        }
    }
}

/// Query module provides functions for performing dex-related queries.
pub mod query {
    use super::isi::*;
    use super::*;
    use crate::query::*;
    use iroha_derive::*;
    use std::time::SystemTime;

    /// Helper function to get reference to `Domain` by its name.
    pub fn get_domain<'a>(
        domain_name: &str,
        world_state_view: &'a WorldStateView,
    ) -> Result<&'a Domain, String> {
        Ok(world_state_view
            .read_domain(domain_name)
            .ok_or("domain not found")?)
    }

    /// Helper function to get mutable reference to `Domain` by its name.
    pub fn get_domain_mut<'a>(
        domain_name: &str,
        world_state_view: &'a mut WorldStateView,
    ) -> Result<&'a mut Domain, String> {
        Ok(world_state_view
            .domain(domain_name)
            .ok_or("domain not found")?)
    }

    /// Helper function to get reference to DEX by name
    /// of containing domain.
    pub fn get_dex<'a>(
        domain_name: &str,
        world_state_view: &'a WorldStateView,
    ) -> Result<&'a DEX, String> {
        Ok(get_domain(domain_name, world_state_view)?
            .dex
            .as_ref()
            .ok_or("dex not initialized for domain")?)
    }

    /// Helper function to get mutable reference to DEX by name
    /// of containing domain.
    pub fn get_dex_mut<'a>(
        domain_name: &str,
        world_state_view: &'a mut WorldStateView,
    ) -> Result<&'a mut DEX, String> {
        Ok(get_domain_mut(domain_name, world_state_view)?
            .dex
            .as_mut()
            .ok_or("dex not initialized for domain")?)
    }

    /// Helper function to get reference to `TokenPair` by its identifier.
    pub fn get_token_pair<'a>(
        token_pair_id: &<TokenPair as Identifiable>::Id,
        world_state_view: &'a WorldStateView,
    ) -> Result<&'a TokenPair, String> {
        Ok(
            get_dex(&token_pair_id.dex_id.domain_name, world_state_view)?
                .token_pairs
                .get(token_pair_id)
                .ok_or("token pair not found")?,
        )
    }
    /// Helper function to get mutable reference to `TokenPair` by its identifier.
    pub fn get_token_pair_mut<'a>(
        token_pair_id: &<TokenPair as Identifiable>::Id,
        world_state_view: &'a mut WorldStateView,
    ) -> Result<&'a mut TokenPair, String> {
        Ok(
            get_dex_mut(&token_pair_id.dex_id.domain_name, world_state_view)?
                .token_pairs
                .get_mut(token_pair_id)
                .ok_or("token pair not found")?,
        )
    }

    /// Helper function to get reference to `LiquiditySource` by its identifier.
    pub fn get_liquidity_source<'a>(
        liquidity_source_id: &<LiquiditySource as Identifiable>::Id,
        world_state_view: &'a WorldStateView,
    ) -> Result<&'a LiquiditySource, String> {
        Ok(
            get_token_pair(&liquidity_source_id.token_pair_id, world_state_view)?
                .liquidity_sources
                .get(liquidity_source_id)
                .ok_or("liquidity source not found")?,
        )
    }

    /// Helper function to get mutable reference to `LiquiditySource` by its identifier.
    pub fn get_liquidity_source_mut<'a>(
        liquidity_source_id: &<LiquiditySource as Identifiable>::Id,
        world_state_view: &'a mut WorldStateView,
    ) -> Result<&'a mut LiquiditySource, String> {
        Ok(
            get_token_pair_mut(&liquidity_source_id.token_pair_id, world_state_view)?
                .liquidity_sources
                .get_mut(liquidity_source_id)
                .ok_or("liquidity source not found")?,
        )
    }

    /// Get balance of asset for specified account.
    pub fn get_asset_quantity(
        account_id: <Account as Identifiable>::Id,
        asset_definition_id: <AssetDefinition as Identifiable>::Id,
        world_state_view: &WorldStateView,
    ) -> Result<u32, String> {
        let asset_id = AssetId::new(asset_definition_id, account_id.clone());
        Ok(world_state_view
            .read_account(&account_id)
            .ok_or("account not found")?
            .assets
            .get(&asset_id)
            .ok_or("asset not found")?
            .quantity)
    }

    /// Helper function to construct default `QueryRequest` with empty signature.
    fn unsigned_query_request(query: IrohaQuery) -> QueryRequest {
        QueryRequest {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Failed to get System Time.")
                .as_millis()
                .to_string(),
            signature: Option::None,
            query,
        }
    }

    /// Get DEX information.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetDEX {
        /// Domain name to which DEX belongs.
        pub domain_name: <Domain as Identifiable>::Id,
    }
    /// Result of `GetDEX` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetDEXResult {
        /// `DEX` entity.
        pub dex: DEX,
    }

    impl GetDEX {
        /// Build a `GetDEX` query in the form of a `QueryRequest`.
        pub fn build_request(domain_name: <Domain as Identifiable>::Id) -> QueryRequest {
            let query = GetDEX { domain_name };
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetDEX {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let dex = get_dex(&self.domain_name, world_state_view)?;
            Ok(QueryResult::GetDEX(GetDEXResult { dex: dex.clone() }))
        }
    }

    /// Get list of active DEX in the network.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetDEXList;

    /// Result of the `GetDEXList` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetDEXListResult {
        /// List of DEX.
        pub dex_list: Vec<DEX>,
    }

    impl GetDEXList {
        /// Build a `GetDEXList` query in the form of a `QueryRequest`.
        pub fn build_request() -> QueryRequest {
            let query = GetDEXList;
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetDEXList {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let dex_list = query_dex_list(world_state_view).cloned().collect();
            Ok(QueryResult::GetDEXList(GetDEXListResult { dex_list }))
        }
    }

    /// A query to get a list of all active DEX in network.
    fn query_dex_list<'a>(world_state_view: &'a WorldStateView) -> impl Iterator<Item = &DEX> + 'a {
        world_state_view
            .peer
            .domains
            .iter()
            .filter_map(|(_, domain)| domain.dex.as_ref())
    }

    /// Get DEX information.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetTokenPair {
        /// Identifier of TokenPair.
        pub token_pair_id: <TokenPair as Identifiable>::Id,
    }
    /// Result of `GetDEX` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetTokenPairResult {
        /// `TokenPair` information.
        pub token_pair: TokenPair,
    }

    impl GetTokenPair {
        /// Build a `GetTokenPair` query in the form of a `QueryRequest`.
        pub fn build_request(token_pair_id: <TokenPair as Identifiable>::Id) -> QueryRequest {
            let query = GetTokenPair { token_pair_id };
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetTokenPair {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let token_pair = get_token_pair(&self.token_pair_id, world_state_view)?;
            Ok(QueryResult::GetTokenPair(GetTokenPairResult {
                token_pair: token_pair.clone(),
            }))
        }
    }

    /// A query to get a particular `TokenPair` identified by its id.
    pub fn query_token_pair(
        token_pair_id: <TokenPair as Identifiable>::Id,
        world_state_view: &WorldStateView,
    ) -> Option<&TokenPair> {
        get_token_pair(&token_pair_id, world_state_view).ok()
    }

    /// Get list of active Token Pairs for Domain by its name.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetTokenPairList {
        domain_name: <Domain as Identifiable>::Id,
    }

    /// Result of the `GetTokenPairList` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetTokenPairListResult {
        /// List of DEX.
        pub token_pair_list: Vec<TokenPair>,
    }

    impl GetTokenPairList {
        /// Build a `GetTokenPairList` query in the form of a `QueryRequest`.
        pub fn build_request(domain_name: <Domain as Identifiable>::Id) -> QueryRequest {
            let query = GetTokenPairList { domain_name };
            unsigned_query_request(query.into())
        }
    }

    /// Add indirect Token Pairs through base asset
    /// Example: for actual pairs -
    /// BASE:TARGET_A, BASE:TARGET_B, BASE:TARGET_C
    /// query will return -
    /// BASE:TARGET_A, BASE:TARGET_B, BASE:TARGET_C,
    /// TARGET_A:TARGET_B, TARGET_A:TARGET_C, TARGET_B:TARGET_C
    impl Query for GetTokenPairList {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let mut token_pair_list = query_token_pair_list(&self.domain_name, world_state_view)
                .ok_or(format!(
                    "No domain with name: {:?} found in the current world state: {:?}",
                    &self.domain_name, world_state_view
                ))?
                .cloned()
                .collect::<Vec<_>>();

            let target_assets = token_pair_list
                .iter()
                .map(|token_pair| token_pair.id.target_asset_id.clone())
                .collect::<Vec<_>>();
            for token_pair in
                get_permuted_pairs(&target_assets)
                    .iter()
                    .map(|(base_asset, target_asset)| {
                        TokenPair::new(
                            DEXId::new(&base_asset.domain_name),
                            base_asset.clone(),
                            target_asset.clone(),
                        )
                    })
            {
                token_pair_list.push(token_pair);
            }
            Ok(QueryResult::GetTokenPairList(GetTokenPairListResult {
                token_pair_list,
            }))
        }
    }

    /// This function returns all combinations of two elements from given sequence.
    /// Combinations are unique without ordering in pairs, i.e. (A,B) and (B,A) considered the same.
    fn get_permuted_pairs<T: Clone>(sequence: &[T]) -> Vec<(T, T)> {
        let mut result = Vec::new();
        for i in 0..sequence.len() {
            for j in i + 1..sequence.len() {
                result.push((
                    sequence.get(i).unwrap().clone(),
                    sequence.get(j).unwrap().clone(),
                ));
            }
        }
        result
    }

    /// A query to get a list of all active `TokenPair`s of a DEX identified by its domain name.
    fn query_token_pair_list<'a>(
        domain_name: &str,
        world_state_view: &'a WorldStateView,
    ) -> Option<impl Iterator<Item = &'a TokenPair>> {
        let dex = world_state_view.read_domain(domain_name)?.dex.as_ref()?;
        Some(dex.token_pairs.iter().map(|(_, value)| value))
    }

    /// Get count of active Token Pairs in DEX.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetTokenPairCount {
        /// Identifier of DEX.
        pub dex_id: <DEX as Identifiable>::Id,
    }
    /// Result of `GetTokenPairCount` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetTokenPairCountResult {
        /// Count of active Token Pairs in DEX.
        pub count: u64,
    }

    impl GetTokenPairCount {
        /// Build a `GetTokenPairList` query in the form of a `QueryRequest`.
        pub fn build_request(dex_id: <DEX as Identifiable>::Id) -> QueryRequest {
            let query = GetTokenPairCount { dex_id };
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetTokenPairCount {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let dex = get_dex(&self.dex_id.domain_name, world_state_view)?;
            Ok(QueryResult::GetTokenPairCount(GetTokenPairCountResult {
                count: dex.token_pairs.len() as u64,
            }))
        }
    }

    /// Get information about XYK Pool.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetXYKPoolInfo {
        /// Identifier of Liquidity Source.
        pub liquidity_source_id: <LiquiditySource as Identifiable>::Id,
    }

    /// Result of `GetXYKPoolInfo` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetXYKPoolInfoResult {
        /// Information about XYK Pool.
        pub pool_data: XYKPoolData,
    }

    impl GetXYKPoolInfo {
        /// Build a `GetXYKPoolInfo` query in the form of a `QueryRequest`.
        pub fn build_request(
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
        ) -> QueryRequest {
            let query = GetXYKPoolInfo {
                liquidity_source_id,
            };
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetXYKPoolInfo {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let liquidity_source =
                get_liquidity_source(&self.liquidity_source_id, world_state_view)?;
            let pool_data = expect_xyk_pool_data(liquidity_source)?;
            Ok(QueryResult::GetXYKPoolInfo(GetXYKPoolInfoResult {
                pool_data: pool_data.clone(),
            }))
        }
    }

    /// Get fee fraction set to be deduced from swaps.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetFeeOnXYKPool {
        /// Identifier of XYK Pool.
        pub liquidity_source_id: <LiquiditySource as Identifiable>::Id,
    }

    /// Result of `GetFeeOnXYKPool` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetFeeOnXYKPoolResult {
        /// Fee fraction expressed in basis points.
        pub fee: u16,
    }

    impl GetFeeOnXYKPool {
        /// Build a `GetFeeOnXYKPool` query in the form of a `QueryRequest`.
        pub fn build_request(
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
        ) -> QueryRequest {
            let query = GetFeeOnXYKPool {
                liquidity_source_id,
            };
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetFeeOnXYKPool {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let liquidity_source =
                get_liquidity_source(&self.liquidity_source_id, world_state_view)?;
            let pool_data = expect_xyk_pool_data(liquidity_source)?;
            Ok(QueryResult::GetFeeOnXYKPool(GetFeeOnXYKPoolResult {
                fee: pool_data.fee,
            }))
        }
    }

    /// Get protocol fee part set to be deducted from regular fee.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetProtocolFeePartOnXYKPool {
        /// Identifier of XYK Pool.
        pub liquidity_source_id: <LiquiditySource as Identifiable>::Id,
    }

    /// Result of `GetProtocolFeePartOnXYKPool` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetProtocolFeePartOnXYKPoolResult {
        /// Protocol fee part expressed as fraction of regular fee in basis points.
        pub protocol_fee_part: u16,
    }

    impl GetProtocolFeePartOnXYKPool {
        /// Build a `GetProtocolFeePartOnXYKPool` query in the form of a `QueryRequest`.
        pub fn build_request(
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
        ) -> QueryRequest {
            let query = GetProtocolFeePartOnXYKPool {
                liquidity_source_id,
            };
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetProtocolFeePartOnXYKPool {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let liquidity_source =
                get_liquidity_source(&self.liquidity_source_id, world_state_view)?;
            let pool_data = expect_xyk_pool_data(liquidity_source)?;
            Ok(QueryResult::GetProtocolFeePartOnXYKPool(
                GetProtocolFeePartOnXYKPoolResult {
                    protocol_fee_part: pool_data.protocol_fee_part,
                },
            ))
        }
    }

    /// Result of `GetPriceForInputTokensOnXYKPool` and `GetPriceForOutputTokensOnXYKPool` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetPriceOnXYKPoolResult {
        /// Input Asset amount, that is deposited by caller into first pool in chain.
        pub input_amount: u32,
        /// Output Asset amount, that is withdrawn from last pool in chain to caller.
        pub output_amount: u32,
        /// Swap outputs for corresponding pools in chain, does not contain input value
        pub amounts: Vec<xyk_pool::SwapOutput>,
    }

    /// Get amount of
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetPriceForInputTokensOnXYKPool {
        /// Identifier of DEX.
        pub dex_id: <DEX as Identifiable>::Id,
        /// Path of exchange, contains assets amoung which exchange will happen.
        pub path: Vec<<AssetDefinition as Identifiable>::Id>,
        /// Input Asset amount.
        pub input_amount: u32,
    }

    impl GetPriceForInputTokensOnXYKPool {
        /// Build a `GetSpotPriceOnXYKPool` query in the form of a `QueryRequest`.
        pub fn build_request(
            dex_id: <DEX as Identifiable>::Id,
            path: Vec<<AssetDefinition as Identifiable>::Id>,
            input_amount: u32,
        ) -> QueryRequest {
            let query = GetPriceForInputTokensOnXYKPool {
                dex_id,
                path,
                input_amount,
            };
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetPriceForInputTokensOnXYKPool {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let (initial_deposit, amounts) = xyk_pool::get_amounts_out(
                self.dex_id.clone(),
                self.input_amount,
                &self.path,
                world_state_view,
            )?;
            Ok(QueryResult::GetPriceForInputTokensOnXYKPool(
                GetPriceOnXYKPoolResult {
                    input_amount: initial_deposit.input_amount,
                    output_amount: amounts.last().unwrap().asset_output.amount(),
                    amounts,
                },
            ))
        }
    }

    /// Get quantity of first asset in path needed to get 1 unit of last asset in path.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetPriceForOutputTokensOnXYKPool {
        /// Identifier of DEX.
        pub dex_id: <DEX as Identifiable>::Id,
        /// Path of exchange, contains assets amoung which exchange will happen.
        pub path: Vec<<AssetDefinition as Identifiable>::Id>,
        /// Input Asset amount.
        pub output_amount: u32,
    }

    impl GetPriceForOutputTokensOnXYKPool {
        /// Build a `GetSpotPriceOnXYKPool` query in the form of a `QueryRequest`.
        pub fn build_request(
            dex_id: <DEX as Identifiable>::Id,
            path: Vec<<AssetDefinition as Identifiable>::Id>,
            output_amount: u32,
        ) -> QueryRequest {
            let query = GetPriceForOutputTokensOnXYKPool {
                dex_id,
                path,
                output_amount,
            };
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetPriceForOutputTokensOnXYKPool {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let (initial_deposit, amounts) = xyk_pool::get_amounts_in(
                self.dex_id.clone(),
                self.output_amount,
                &self.path,
                world_state_view,
            )?;
            Ok(QueryResult::GetPriceForOutputTokensOnXYKPool(
                GetPriceOnXYKPoolResult {
                    input_amount: initial_deposit.input_amount,
                    output_amount: amounts.last().unwrap().asset_output.amount(),
                    amounts,
                },
            ))
        }
    }

    /// Get quantities of base and target assets that correspond to owned pool tokens.
    #[derive(Clone, Debug, Io, IntoQuery, Encode, Decode)]
    pub struct GetOwnedLiquidityOnXYKPool {
        /// Identifier of XYK Pool.
        pub liquidity_source_id: <LiquiditySource as Identifiable>::Id,
        /// Account holding pool tokens.
        pub account_id: <Account as Identifiable>::Id,
        /// Pool token amount desired to remove.
        pub pool_tokens_amount: Option<u32>,
    }

    /// Result of `GetOwnedLiquidityOnXYKPool` execution.
    #[derive(Clone, Debug, Encode, Decode)]
    pub struct GetOwnedLiquidityOnXYKPoolResult {
        /// Amount of base asset.
        pub base_asset_amount: u32,
        /// Amount of target asset.
        pub target_asset_amount: u32,
        /// Pool tokens remaining after withdrawal.
        pub pool_tokens_after_withdrawal: u32,
        /// Pool tokens remaining before withdrawal.
        pub pool_tokens_before_withdrawal: u32,
    }

    impl GetOwnedLiquidityOnXYKPool {
        /// Build a `GetOwnedLiquidityOnXYKPool` query in the form of a `QueryRequest`.
        pub fn build_request(
            liquidity_source_id: <LiquiditySource as Identifiable>::Id,
            account_id: <Account as Identifiable>::Id,
            pool_tokens_amount: Option<u32>,
        ) -> QueryRequest {
            let query = GetOwnedLiquidityOnXYKPool {
                liquidity_source_id,
                account_id,
                pool_tokens_amount,
            };
            unsigned_query_request(query.into())
        }
    }

    impl Query for GetOwnedLiquidityOnXYKPool {
        #[log]
        fn execute(&self, world_state_view: &WorldStateView) -> Result<QueryResult, String> {
            let liquidity_source =
                get_liquidity_source(&self.liquidity_source_id, world_state_view)?;
            let token_pair_id = &liquidity_source.id.token_pair_id;
            let pool_data = expect_xyk_pool_data(liquidity_source)?;
            let base_asset_balance = get_asset_quantity(
                pool_data.storage_account_id.clone(),
                token_pair_id.base_asset_id.clone(),
                world_state_view,
            )?;
            let target_asset_balance = get_asset_quantity(
                pool_data.storage_account_id.clone(),
                token_pair_id.target_asset_id.clone(),
                world_state_view,
            )?;
            let pool_tokens_balance = get_asset_quantity(
                self.account_id.clone(),
                pool_data.pool_token_asset_definition_id.clone(),
                world_state_view,
            )?;
            let pool_tokens_desired = match self.pool_tokens_amount {
                Some(amount) => {
                    if amount > pool_tokens_balance {
                        return Err(format!(
                            "insufficient pool tokens available: {} > {}",
                            amount, pool_tokens_balance
                        ));
                    }
                    amount
                }
                None => pool_tokens_balance,
            };
            let base_asset_quantity =
                pool_tokens_desired * base_asset_balance / pool_data.pool_token_total_supply;
            let target_asset_quantity =
                pool_tokens_desired * target_asset_balance / pool_data.pool_token_total_supply;
            Ok(QueryResult::GetOwnedLiquidityOnXYKPool(
                GetOwnedLiquidityOnXYKPoolResult {
                    base_asset_amount: base_asset_quantity,
                    target_asset_amount: target_asset_quantity,
                    pool_tokens_after_withdrawal: pool_tokens_balance - pool_tokens_desired,
                    pool_tokens_before_withdrawal: pool_tokens_balance,
                },
            ))
        }
    }
}
