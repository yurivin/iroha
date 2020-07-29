//! This module contains functionality related to `DEX`.

use crate::permission::*;
use crate::prelude::*;
use integer_sqrt::*;
use iroha_derive::Io;
use parity_scale_codec::{Decode, Encode};
use std::cmp;
use std::collections::BTreeMap;

const PSWAP_ASSET_NAME: &str = "PSWAP";
const STORAGE_ACCOUNT_NAME: &str = "STORE";
const DEX_BASE_ASSET: &str = "XOR";
const MINIMUM_LIQUIDITY: u32 = 1000;

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
}

impl DEX {
    /// Default `DEX` entity constructor.
    pub fn new(domain_name: &str, owner_account_id: <Account as Identifiable>::Id) -> Self {
        DEX {
            id: DEXId::new(domain_name),
            owner_account_id,
            token_pairs: BTreeMap::new(),
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
    pub base_asset: <AssetDefinition as Identifiable>::Id,
    /// Target token of exchange.
    pub target_asset: <AssetDefinition as Identifiable>::Id,
}

impl TokenPairId {
    /// Default Token Pair identifier constructor.
    pub fn new(
        dex_id: <DEX as Identifiable>::Id,
        base_asset: <AssetDefinition as Identifiable>::Id,
        target_asset: <AssetDefinition as Identifiable>::Id,
    ) -> Self {
        TokenPairId {
            dex_id,
            base_asset,
            target_asset,
        }
    }
    /// Symbol representation of the Token Pair.
    pub fn get_symbol(&self) -> String {
        // TODO: elaborate the format
        format!(
            "{}-{}",
            self.base_asset.to_string(),
            self.target_asset.to_string(),
        )
    }
}

/// `TokenPair` represents an exchange pair between two assets in a domain. Assets are
/// identified by their AssetDefinitionId's. Containing DEX is identified by domain name.
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
pub struct TokenPair {
    /// An Identification of the `TokenPair`, holds pair of token Ids.
    pub id: <TokenPair as Identifiable>::Id,
    /// Precision of the exchange rate, measured in a number of decimal places.
    pub precision: u8,
    /// Fraction of price by which it can change.
    pub price_step: u32,
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
        precision: u8,
        price_step: u32,
    ) -> Self {
        TokenPair {
            id: TokenPairId::new(dex_id, base_asset, target_asset),
            precision,
            price_step,
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
#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub enum LiquiditySourceType {
    /// X*Y=K liquidity pool.
    XYKPool,
    /// Regular order book.
    OrderBook,
}

/// `LiquiditySource` represents an exchange pair between two assets in a domain. Assets are
/// identified by their AssetDefinitionId's. Containing DEX is identified by domain name.
#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub enum LiquiditySourceData {
    /// Data representing state of the XYK liquidity pool.
    XYKPool {
        /// Asset definition of liquidity token belongin to given pool.
        pswap_asset_definition_id: <AssetDefinition as Identifiable>::Id,
        /// Account that is used to store exchanged tokens, i.e. actual liquidity.
        storage_account_id: <Account as Identifiable>::Id,
        /// Amount of active liquidity tokens.
        pswap_total_supply: u32,
        /// Amount of base tokens in the pool (currently stored in storage account).
        base_asset_amount: u32,
        /// Amount of target tokens in the pool (currently stored in storage account).
        target_asset_amount: u32,
        /// K (constant product) value, updated by latest liquidity operation.
        k_last: u32,
    },
    /// Data representing state of the Order Book.
    OrderBook, // this option currently is to prevent `irrefutable if-let pattern` warning
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
        pswap_asset_definition_id: <AssetDefinition as Identifiable>::Id,
        storage_account_id: <Account as Identifiable>::Id,
    ) -> Self {
        let data = LiquiditySourceData::XYKPool {
            pswap_asset_definition_id,
            storage_account_id,
            pswap_total_supply: 0,
            base_asset_amount: 0,
            target_asset_amount: 0,
            k_last: 0,
        };
        let id = LiquiditySourceId::new(token_pair_id, LiquiditySourceType::XYKPool);
        LiquiditySource { id, data }
    }
}

/// Iroha Special Instructions module provides helper-methods for `Peer` for operating DEX,
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
        /// Variant of instruction to add liquidity source for existing `TokenPair`.
        AddLiquiditySource(LiquiditySource, <TokenPair as Identifiable>::Id),
        /// Variant of instruction to deposit tokens to liquidity pool.
        /// `LiquiditySource` <-- Amount Base Desired, Amount Target Desired, Amount Base Min, Amount Target Min
        AddLiquidityToXYKPool(<LiquiditySource as Identifiable>::Id, u32, u32, u32, u32),
        /// Variant of instruction to mint permissions needeed for trading on dex for account.
        /// TODO: this ISI is for debug purposes and should be deleted later
        ActivateXYKPoolTraderAccount(
            <LiquiditySource as Identifiable>::Id,
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
                DEXInstruction::AddLiquiditySource(liquidity_source, token_pair_id) => {
                    Add::new(liquidity_source.clone(), token_pair_id.clone())
                        .execute(authority, world_state_view)
                }
                DEXInstruction::AddLiquidityToXYKPool(
                    liquidity_source_id,
                    amount_a_desired,
                    amount_b_desired,
                    amount_a_min,
                    amount_b_min,
                ) => xyk_pool_add_liquidity_execute(
                    liquidity_source_id.clone(),
                    amount_a_desired.clone(),
                    amount_b_desired.clone(),
                    amount_a_min.clone(),
                    amount_b_min.clone(),
                    authority.clone(), // TODO: change this
                    authority,
                    world_state_view,
                ),
                DEXInstruction::ActivateXYKPoolTraderAccount(liquidity_source_id, account_id) => {
                    activate_trader_account_execute(
                        liquidity_source_id.clone(),
                        account_id.clone(),
                        authority,
                        world_state_view,
                    )
                }
            }
        }
    }

    /// Constructor of `Register<Domain, DEX>` ISI.
    ///
    /// Initializes DEX for the domain.
    pub fn initialize_dex(
        domain_name: &str,
        owner_account_id: <Account as Identifiable>::Id,
    ) -> Register<Domain, DEX> {
        Register {
            object: DEX::new(domain_name, owner_account_id),
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
            PermissionInstruction::CanManageDEX(authority, Some(domain_name.clone()))
                .execute(world_state_view)?;
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
            object: TokenPair::new(dex_id.clone(), base_asset, target_asset, 0, 0),
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
            let domain = get_domain(&domain_name, world_state_view)?;
            let base_asset_definition = &token_pair.id.base_asset;
            let target_asset_definition = &token_pair.id.target_asset;
            if !domain.asset_definitions.contains_key(base_asset_definition) {
                return Err(format!(
                    "base asset definition: {:?} not found",
                    base_asset_definition
                ));
            }
            if &base_asset_definition.name != DEX_BASE_ASSET {
                return Err(format!(
                    "base asset definition is incorrect: {}",
                    base_asset_definition
                ));
            }
            if !domain
                .asset_definitions
                .contains_key(target_asset_definition)
            {
                return Err(format!(
                    "target asset definition: {:?} not found",
                    target_asset_definition
                ));
            }
            if base_asset_definition.domain_name != target_asset_definition.domain_name {
                return Err("assets in token pair must be in same domain".to_owned());
            }
            if base_asset_definition.name == target_asset_definition.name {
                return Err("assets in token pair must be different".to_owned());
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

    /// Constructor of `Add<DEX, LiquiditySource>` ISI.
    ///
    /// Add new XYK Liquidity Pool for DEX with given `TokenPair`.
    pub fn create_xyk_pool(token_pair_id: <TokenPair as Identifiable>::Id) -> Instruction {
        let domain_name = token_pair_id.dex_id.domain_name.clone();
        let asset_name = format!("{} XYK {}", PSWAP_ASSET_NAME, token_pair_id.get_symbol());
        let pswap_asset_definition =
            AssetDefinition::new(AssetDefinitionId::new(&asset_name, &domain_name));
        let account_name = format!(
            "{} XYK {}",
            STORAGE_ACCOUNT_NAME,
            token_pair_id.get_symbol()
        );
        let storage_account = Account::new(&account_name, &domain_name);
        // TODO: add checking for existing xyk pool for pair
        Instruction::If(
            Box::new(Instruction::ExecuteQuery(IrohaQuery::GetTokenPair(
                GetTokenPair {
                    token_pair_id: token_pair_id.clone(),
                },
            ))),
            Box::new(Instruction::Sequence(vec![
                // register storage account for pool
                Register {
                    object: pswap_asset_definition.clone(),
                    destination_id: domain_name.clone(),
                }
                .into(),
                // register asset definition for pswap
                Register {
                    object: storage_account.clone(),
                    destination_id: domain_name.clone(),
                }
                .into(),
                // create xyk pool for pair
                Add {
                    object: LiquiditySource::new_xyk_pool(
                        token_pair_id.clone(),
                        pswap_asset_definition.id.clone(),
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
            Instruction::DEX(DEXInstruction::AddLiquiditySource(
                instruction.object,
                instruction.destination_id,
            ))
        }
    }

    /// Constructor if `AddLiquidityToXYKPool` ISI.
    pub fn xyk_pool_add_liquidity(
        liquidity_source_id: <LiquiditySource as Identifiable>::Id,
        amount_a_desired: u32,
        amount_b_desired: u32,
        amount_a_min: u32,
        amount_b_min: u32,
    ) -> Instruction {
        Instruction::DEX(DEXInstruction::AddLiquidityToXYKPool(
            liquidity_source_id,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
        ))
    }

    /// Core logic of `AddLiquidityToXYKPool` ISI, called by its `execute` function.
    ///
    /// `liquidity_source_id` - should be xyk pool.
    /// `amount_a_desired` - desired base asset quantity (maximum) to be deposited.
    /// `amount_b_desired` - desired target asset quantity (maximum) to be deposited.
    /// `amount_a_min` - lower bound for base asset quantity to be deposited.
    /// `amount_b_min` - lower bound for target asset quantity to be deposited.
    /// `to` - account to receive liquidity tokens for deposit.
    /// `authority` - permorms the operation, actual tokens are withdrawn from this account.
    fn xyk_pool_add_liquidity_execute(
        liquidity_source_id: <LiquiditySource as Identifiable>::Id,
        amount_a_desired: u32,
        amount_b_desired: u32,
        amount_a_min: u32,
        amount_b_min: u32,
        to: <Account as Identifiable>::Id,
        authority: <Account as Identifiable>::Id,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        let liquidity_source = get_liquidity_source(&liquidity_source_id, world_state_view)?;
        let token_pair_id = &liquidity_source_id.token_pair_id;

        // TODO: consider wrapping data into struct contained in enum
        if let LiquiditySourceData::XYKPool {
            pswap_asset_definition_id,
            storage_account_id,
            pswap_total_supply,
            base_asset_amount,
            target_asset_amount,
            k_last,
        } = liquidity_source.data.clone()
        {
            let reserve_a = base_asset_amount;
            let reserve_b = target_asset_amount;
            // calculate appropriate deposit quantities to preserve pool proportions
            let (amount_a, amount_b) = xyk_pool_get_optimal_deposit_amounts(
                reserve_a.clone(),
                reserve_b.clone(),
                amount_a_desired,
                amount_b_desired,
                amount_a_min,
                amount_b_min,
            )?;
            // deposit tokens into the storage account
            transfer_from(
                token_pair_id.base_asset.clone(),
                authority.clone(),
                storage_account_id.clone(),
                amount_a.clone(),
                authority.clone(),
                world_state_view,
            )?;
            transfer_from(
                token_pair_id.target_asset.clone(),
                authority.clone(),
                storage_account_id.clone(),
                amount_b.clone(),
                authority.clone(),
                world_state_view,
            )?;
            // mint pswap for sender based on deposited amount
            let (reserve_a, reserve_b, k, total_supply) = mint_pswap_with_fee(
                pswap_asset_definition_id.clone(),
                to,
                amount_a,
                amount_b,
                reserve_a,
                reserve_b,
                k_last,
                None,
                pswap_total_supply,
                world_state_view,
            )?;
            // update state of the pool
            if let LiquiditySourceData::XYKPool {
                pswap_total_supply,
                base_asset_amount,
                target_asset_amount,
                k_last,
                ..
            } = &mut get_liquidity_source_mut(&liquidity_source_id, world_state_view)?.data
            {
                *pswap_total_supply = total_supply;
                *base_asset_amount = reserve_a;
                *target_asset_amount = reserve_b;
                *k_last = k;
            };
            Ok(())
        } else {
            Err("wrong liquidity source: adding liquidity to xyk pool".to_owned())
        }
    }

    /// Based on given reserves, desired and minimal amounts to add liquidity, either return
    /// optimal values (needed to preserve reserves proportion) or error if it's not possible
    /// to keep proportion with proposed amounts.
    fn xyk_pool_get_optimal_deposit_amounts(
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
            let amount_b_optimal = xyk_pool_quote(
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
                let amount_a_optimal =
                    xyk_pool_quote(amount_b_desired.clone(), reserve_b, reserve_a)?;
                assert!(amount_a_optimal <= amount_a_desired); // TODO: consider not using assert
                if !(amount_a_optimal >= amount_a_min) {
                    return Err("insufficient a amount".to_owned());
                }
                (amount_a_optimal, amount_b_desired)
            }
        })
    }

    /// Given some amount of an asset and pair reserves, returns an equivalent amount of the other Asset.
    fn xyk_pool_quote(amount_a: u32, reserve_a: u32, reserve_b: u32) -> Result<u32, String> {
        if !(amount_a > 0u32) {
            return Err("insufficient amount".to_owned());
        }
        if !(reserve_a > 0u32 && reserve_b > 0u32) {
            return Err("insufficient liquidity".to_owned());
        }
        Ok((amount_a * reserve_b) / reserve_a) // calculate amount_b via proportion
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

    /// Mint pswap tokens representing liquidity in pool for depositing account.
    /// returns (reserve_a, reserve_b, k_last, total_supply)
    fn mint_pswap_with_fee(
        pswap_asset_definition_id: <AssetDefinition as Identifiable>::Id,
        to: <Account as Identifiable>::Id,
        amount_a: u32,
        amount_b: u32,
        reserve_a: u32,
        reserve_b: u32,
        k_last: u32,
        fee_to: Option<<Account as Identifiable>::Id>,
        total_supply: u32,
        world_state_view: &mut WorldStateView,
    ) -> Result<(u32, u32, u32, u32), String> {
        let (mut k_last, total_supply) = mint_pswap_fee(
            pswap_asset_definition_id.clone(),
            reserve_a.clone(),
            reserve_b.clone(),
            k_last.clone(),
            fee_to.clone(),
            total_supply.clone(),
            world_state_view,
        )?;

        let liquidity;
        let total_supply = if total_supply == 0u32 {
            liquidity = (amount_a * amount_b).integer_sqrt() - MINIMUM_LIQUIDITY;
            //TODO: does minimum liquidity need to be locked?
            // mint (address(0), MINIMUM_LIQUIDITY), equivalent to just raising total_supply:
            MINIMUM_LIQUIDITY
        } else {
            liquidity = cmp::min(
                (amount_a * total_supply) / reserve_a,
                (amount_b * total_supply) / reserve_b,
            );
            total_supply
        };
        if !(liquidity > 0u32) {
            return Err("insufficient liquidity minted".to_owned());
        }
        let total_supply = mint_pswap(
            pswap_asset_definition_id,
            to,
            liquidity,
            total_supply,
            world_state_view,
        )?;
        let (reserve_a, reserve_b) = (amount_a, amount_b);
        if fee_to.is_some() {
            k_last = reserve_a * reserve_b;
        }
        Ok((reserve_a, reserve_b, k_last, total_supply))
    }

    /// returns (k_last, total_supply)
    fn mint_pswap_fee(
        asset_definition_id: <AssetDefinition as Identifiable>::Id,
        reserve_a: u32,
        reserve_b: u32,
        k_last: u32,
        fee_to: Option<<Account as Identifiable>::Id>,
        mut total_supply: u32,
        world_state_view: &mut WorldStateView,
    ) -> Result<(u32, u32), String> {
        if let Some(fee_to) = fee_to {
            if k_last != 0u32 {
                let root_k = (reserve_a * reserve_b).integer_sqrt();
                let root_k_last = k_last.integer_sqrt();
                if root_k > root_k_last {
                    let numerator = total_supply * (root_k - root_k_last);
                    let demonimator = 5 * root_k + root_k_last;
                    let liquidity = numerator / demonimator;
                    if liquidity > 0u32 {
                        total_supply = mint_pswap(
                            asset_definition_id,
                            fee_to,
                            liquidity,
                            total_supply.clone(),
                            world_state_view,
                        )?;
                    }
                }
            }
        } else if k_last != 0u32 {
            return Ok((0u32, total_supply));
        }
        Ok((k_last, total_supply))
    }

    /// returns (total_supply)
    fn mint_pswap(
        asset_definition_id: <AssetDefinition as Identifiable>::Id,
        to: <Account as Identifiable>::Id,
        value: u32,
        total_supply: u32,
        world_state_view: &mut WorldStateView,
    ) -> Result<u32, String> {
        let asset_id = AssetId::new(asset_definition_id, to);
        mint_asset_unchecked(asset_id, value, world_state_view)?;
        Ok(total_supply + value)
    }

    fn mint_asset_unchecked(
        asset_id: <Asset as Identifiable>::Id,
        quantity: u32,
        //authority: <Account as Identifiable>::Id,
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

    /// Mint permissions required to participate in trades
    ///
    /// TODO: this function has unchecked operations and exists solely for debug purposes
    fn activate_trader_account_execute(
        liquidity_source_id: <LiquiditySource as Identifiable>::Id,
        account_id: <Account as Identifiable>::Id,
        _authority: <Account as Identifiable>::Id,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        match liquidity_source_id.liquidity_source_type.clone() {
            LiquiditySourceType::XYKPool => {
                let domain_name = account_id.domain_name.clone();
                let permission_asset_definition_id = permission_asset_definition_id();
                let asset_id = AssetId {
                    definition_id: permission_asset_definition_id.clone(),
                    account_id: account_id.clone(),
                };
                let liquidity_source =
                    get_liquidity_source(&liquidity_source_id, world_state_view)?;
                if let LiquiditySourceData::XYKPool {
                    pswap_asset_definition_id,
                    ..
                } = &liquidity_source.data
                {
                    let asset = Asset::with_permissions(
                        asset_id.clone(),
                        &[
                            Permission::TransferAsset(
                                None,
                                Some(liquidity_source_id.token_pair_id.base_asset.clone()),
                            ),
                            Permission::TransferAsset(
                                None,
                                Some(liquidity_source_id.token_pair_id.target_asset.clone()),
                            ),
                            Permission::TransferAsset(
                                None,
                                Some(pswap_asset_definition_id.clone()),
                            ),
                        ],
                    );
                    let domain = get_domain_mut(&domain_name, world_state_view)?;
                    domain
                        .accounts
                        .get_mut(&account_id)
                        .ok_or("failed to find account")?
                        .assets
                        .insert(asset_id, asset);
                }
            }
            _ => {}
        }
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
        }

        impl TestKit {
            pub fn new() -> Self {
                let domain_name = "Company".to_string();
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
                }
            }
        }

        #[test]
        fn test_initialize_dex_should_pass() {
            let mut testkit = TestKit::new();
            let domain_name = testkit.domain_name.clone();

            // get world state view and dex domain
            let world_state_view = &mut testkit.world_state_view;

            initialize_dex(&domain_name, testkit.dex_owner_account_id.clone())
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
                assert_eq!(&dex_list_result.dex_list, &[dex_query_result.clone()])
            } else {
                panic!("wrong enum variant returned for GetDEXList");
            }
        }

        #[test]
        fn test_initialize_dex_should_fail_with_permission_not_found() {
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
            let register_account = domain.register_account(dex_owner_account.clone());
            register_account
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register dex owner account");

            assert!(initialize_dex(&domain_name, dex_owner_account.id.clone(),)
                .execute(dex_owner_account.id.clone(), world_state_view)
                .unwrap_err()
                .contains("Error: Permission not found."));
        }

        #[test]
        fn test_create_and_delete_token_pair_should_pass() {
            let mut testkit = TestKit::new();
            let domain_name = testkit.domain_name.clone();

            // get world state view and dex domain
            let world_state_view = &mut testkit.world_state_view;
            let domain = world_state_view
                .read_domain(&domain_name)
                .expect("domain not found")
                .clone();

            // initialize dex in domain
            initialize_dex(&domain_name, testkit.dex_owner_account_id.clone())
                .execute(testkit.dex_owner_account_id.clone(), world_state_view)
                .expect("failed to initialize dex");

            // register assets in domain
            let asset_definition_a = AssetDefinition::new(AssetDefinitionId::new("XOR", "Company"));
            domain
                .register_asset(asset_definition_a.clone())
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register asset");
            let asset_definition_b = AssetDefinition::new(AssetDefinitionId::new("DOT", "Company"));
            domain
                .register_asset(asset_definition_b.clone())
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register asset");

            create_token_pair(
                asset_definition_a.id.clone(),
                asset_definition_b.id.clone(),
                &domain_name,
            )
            .execute(testkit.dex_owner_account_id.clone(), world_state_view)
            .expect("create token pair failed");

            let token_pair_id = TokenPairId::new(
                DEXId::new(&domain_name),
                asset_definition_a.id.clone(),
                asset_definition_b.id.clone(),
            );
            // TODO: rewrite into iroha query calls
            let token_pair = query_token_pair(token_pair_id.clone(), world_state_view)
                .expect("failed to query token pair");
            assert_eq!(&token_pair_id, &token_pair.id);

            if let QueryResult::GetTokenPairList(token_pair_list_result) =
                GetTokenPairList::build_request(domain_name.clone())
                    .query
                    .execute(world_state_view)
                    .expect("failed to query token pair list")
            {
                assert_eq!(
                    &token_pair_list_result.token_pair_list,
                    &[token_pair.clone()]
                )
            } else {
                panic!("wrong enum variant returned for GetTokenPairList");
            }

            let token_pair_count = query_token_pair_count(&domain_name, world_state_view)
                .expect("failed to query token pair count");
            assert_eq!(token_pair_count, 1);

            remove_token_pair(token_pair_id.clone())
                .execute(testkit.dex_owner_account_id.clone(), world_state_view)
                .expect("remove token pair failed");

            if let QueryResult::GetTokenPairList(token_pair_list_result) =
                GetTokenPairList::build_request(domain_name.clone())
                    .query
                    .execute(world_state_view)
                    .expect("failed to query token pair list")
            {
                assert!(&token_pair_list_result.token_pair_list.is_empty());
            } else {
                panic!("wrong enum variant returned for GetTokenPairList");
            }

            let token_pair_count = query_token_pair_count(&domain_name, world_state_view)
                .expect("failed to query token pair count");
            assert_eq!(token_pair_count, 0);
        }

        #[test]
        fn test_xyk_pool_add_liquidity_should_pass() {
            let mut testkit = TestKit::new();
            let domain_name = testkit.domain_name.clone();

            // get world state view and dex domain
            let world_state_view = &mut testkit.world_state_view;
            let domain = world_state_view
                .read_domain(&domain_name)
                .expect("domain not found")
                .clone();

            // initialize dex in domain
            initialize_dex(&domain_name, testkit.dex_owner_account_id.clone())
                .execute(testkit.dex_owner_account_id.clone(), world_state_view)
                .expect("failed to initialize dex");

            // register assets in domain
            let asset_definition_a = AssetDefinition::new(AssetDefinitionId::new("XOR", "Company"));
            domain
                .register_asset(asset_definition_a.clone())
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register asset");
            let asset_definition_b = AssetDefinition::new(AssetDefinitionId::new("DOT", "Company"));
            domain
                .register_asset(asset_definition_b.clone())
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register asset");

            // create account with permissions to transfer
            let key_pair = KeyPair::generate().expect("Failed to generate KeyPair.");
            let account_id = AccountId::new("User", &domain_name);
            let mut account = Account::with_signatory(
                &account_id.name,
                &account_id.domain_name,
                key_pair.public_key.clone(),
            );
            let permission_asset_definition_id = permission_asset_definition_id();
            let permission_asset_id = AssetId {
                definition_id: permission_asset_definition_id.clone(),
                account_id: account_id.clone(),
            };
            let permission_asset = Asset::with_permissions(
                permission_asset_id.clone(),
                &[
                    Permission::TransferAsset(None, Some(asset_definition_a.id.clone())),
                    Permission::TransferAsset(None, Some(asset_definition_b.id.clone())),
                ],
            );
            account.assets.insert(permission_asset_id, permission_asset);
            domain
                .register_account(account)
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to create account");

            // mint tokens to account
            let asset_a_id = AssetId::new(asset_definition_a.id.clone(), account_id.clone());
            let asset_b_id = AssetId::new(asset_definition_b.id.clone(), account_id.clone());
            Mint::new(5000u32, asset_a_id.clone())
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("mint asset failed");
            Mint::new(7000u32, asset_b_id.clone())
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("mint asset failed");

            // register pair for exchange assets
            create_token_pair(
                asset_definition_a.id.clone(),
                asset_definition_b.id.clone(),
                &domain_name,
            )
            .execute(testkit.dex_owner_account_id.clone(), world_state_view)
            .expect("create token pair failed");

            let token_pair_id = TokenPairId::new(
                DEXId::new(&domain_name),
                asset_definition_a.id.clone(),
                asset_definition_b.id.clone(),
            );

            // initialize xyk pool for token pair
            create_xyk_pool(token_pair_id.clone())
                .execute(testkit.dex_owner_account_id.clone(), world_state_view)
                .expect("create xyk pool failed");

            // replicate generated id's for checking
            let xyk_pool_id =
                LiquiditySourceId::new(token_pair_id.clone(), LiquiditySourceType::XYKPool);
            let storage_account_id_test = AccountId::new(
                &format!(
                    "{} XYK {}",
                    STORAGE_ACCOUNT_NAME,
                    &token_pair_id.get_symbol()
                ),
                &domain_name,
            );
            let pswap_asset_definition_id_test = AssetDefinitionId::new(
                &format!("{} XYK {}", PSWAP_ASSET_NAME, &token_pair_id.get_symbol()),
                &domain_name,
            );

            // add minted tokens to the pool from account
            xyk_pool_add_liquidity(xyk_pool_id.clone(), 5000, 7000, 4000, 6000)
                .execute(account_id.clone(), world_state_view)
                .expect("add liquidity failed");

            // check state of XYK Pool
            if let QueryResult::GetTokenPair(token_pair_result) =
                GetTokenPair::build_request(token_pair_id.clone())
                    .query
                    .execute(world_state_view)
                    .expect("failed to query token pair")
            {
                let token_pair = token_pair_result.token_pair;
                let xyk_pool = token_pair
                    .liquidity_sources
                    .get(&xyk_pool_id)
                    .expect("failed to find xyk pool in token pair");
                if let LiquiditySourceData::XYKPool {
                    pswap_asset_definition_id,
                    storage_account_id,
                    pswap_total_supply,
                    base_asset_amount,
                    target_asset_amount,
                    k_last,
                } = xyk_pool.data.clone()
                {
                    assert_eq!(&pswap_asset_definition_id, &pswap_asset_definition_id_test);
                    assert_eq!(&storage_account_id, &storage_account_id_test);
                    assert!(pswap_total_supply > 0u32);
                    assert_eq!(base_asset_amount, 5000);
                    assert_eq!(target_asset_amount, 7000);
                    assert!(k_last == 0u32);
                } else {
                    panic!("wrong data type of liquidity source")
                }
            } else {
                panic!("wrong enum variant returned for GetTokenPair");
            }

            // check depositing account to have decreased base/target tokens and minted liquidity tokens
            if let QueryResult::GetAccount(account_result) =
                GetAccount::build_request(account_id.clone())
                    .query
                    .execute(world_state_view)
                    .expect("failed to query token pair")
            {
                let account = account_result.account;
                let base_asset = account
                    .assets
                    .get(&asset_a_id)
                    .expect("failed to get base asset");
                let target_asset = account
                    .assets
                    .get(&asset_b_id)
                    .expect("failed to get target asset");
                assert_eq!(base_asset.quantity.clone(), 0);
                assert_eq!(target_asset.quantity.clone(), 0);
                let pswap_asset_id =
                    AssetId::new(pswap_asset_definition_id_test.clone(), account_id.clone());
                let pswap_asset = account
                    .assets
                    .get(&pswap_asset_id)
                    .expect("failed to get pswap asset");
                assert!(pswap_asset.quantity > 0);
            } else {
                panic!("wrong enum variant returned for GetAccount");
            }

            // check storage account to have increased base/target tokens
            if let QueryResult::GetAccount(account_result) =
                GetAccount::build_request(storage_account_id_test.clone())
                    .query
                    .execute(world_state_view)
                    .expect("failed to query token pair")
            {
                let storage_asset_a_id = AssetId::new(
                    asset_definition_a.id.clone(),
                    storage_account_id_test.clone(),
                );
                let storage_asset_b_id = AssetId::new(
                    asset_definition_b.id.clone(),
                    storage_account_id_test.clone(),
                );
                let account = account_result.account;
                let base_asset = account
                    .assets
                    .get(&storage_asset_a_id)
                    .expect("failed to get base asset");
                let target_asset = account
                    .assets
                    .get(&storage_asset_b_id)
                    .expect("failed to get target asset");
                assert_eq!(base_asset.quantity.clone(), 5000);
                assert_eq!(target_asset.quantity.clone(), 7000);
            } else {
                panic!("wrong enum variant returned for GetAccount");
            }
        }

        #[test]
        fn test_mint_pswap_should_pass() {
            let mut testkit = TestKit::new();
            let world_state_view = &mut testkit.world_state_view;
            let domain_name = testkit.domain_name.clone();
            let domain = world_state_view
                .domain(&domain_name)
                .expect("domain not found")
                .clone();
            let account_public_key = KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key;
            let account = Account::with_signatory("user", &domain_name, account_public_key);
            let register_account = domain.register_account(account.clone());
            register_account
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register account");

            // register assets
            let base_asset_definition_id = AssetDefinitionId::new("XOR", &domain_name);
            let target_asset_definition_id = AssetDefinitionId::new("DOT", &domain_name);
            let dex_id = DEXId::new(&domain_name);
            let token_pair_id =
                TokenPairId::new(dex_id, base_asset_definition_id, target_asset_definition_id);
            let asset_name = format!("{} XYK {}", PSWAP_ASSET_NAME, token_pair_id.get_symbol());
            let pswap_asset_definition_id = AssetDefinitionId::new(&asset_name, &domain_name);
            domain
                .register_asset(AssetDefinition::new(pswap_asset_definition_id.clone()))
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register asset");

            // set initial total supply to 100
            let total_supply = 100u32;
            let total_supply = mint_pswap(
                pswap_asset_definition_id.clone(),
                account.id.clone(),
                100u32,
                total_supply,
                world_state_view,
            )
            .expect("failed to mint pswap");
            // after minting 100 pswap, total supply should be 200
            assert_eq!(total_supply.clone(), 200u32);

            if let QueryResult::GetAccount(account_result) =
                GetAccount::build_request(account.id.clone())
                    .query
                    .execute(world_state_view)
                    .expect("failed to query token pair")
            {
                let account = account_result.account;
                let pswap_asset_id =
                    AssetId::new(pswap_asset_definition_id.clone(), account.id.clone());
                let pswap_asset = account
                    .assets
                    .get(&pswap_asset_id)
                    .expect("failed to get pswap asset");
                // account should contain 100 pswap
                assert_eq!(pswap_asset.quantity, 100);
            } else {
                panic!("wrong enum variant returned for GetAccount");
            }
        }

        #[test]
        fn test_xyk_pool_optimal_liquidity_should_pass() {
            // zero reserves return desired amounts
            let (amount_a, amount_b) =
                xyk_pool_get_optimal_deposit_amounts(0, 0, 10000, 5000, 10000, 5000)
                    .expect("failed to get optimal asset amounts");
            assert_eq!(amount_a, 10000);
            assert_eq!(amount_b, 5000);
            // add liquidity with same proportions
            let (amount_a, amount_b) =
                xyk_pool_get_optimal_deposit_amounts(10000, 5000, 10000, 5000, 10000, 5000)
                    .expect("failed to get optimal asset amounts");
            assert_eq!(amount_a, 10000);
            assert_eq!(amount_b, 5000);
            // add liquidity with different proportions
            let (amount_a, amount_b) =
                xyk_pool_get_optimal_deposit_amounts(10000, 5000, 5000, 10000, 0, 0)
                    .expect("failed to get optimal asset amounts");
            assert_eq!(amount_a, 5000);
            assert_eq!(amount_b, 2500);
            // add liquidity `b_optimal>b_desired` branch
            let (amount_a, amount_b) =
                xyk_pool_get_optimal_deposit_amounts(10000, 5000, 5000, 2000, 0, 0)
                    .expect("failed to get optimal asset amounts");
            assert_eq!(amount_a, 4000);
            assert_eq!(amount_b, 2000);
        }

        #[test]
        fn test_xyk_pool_quote_should_pass() {
            let amount_b_optimal =
                xyk_pool_quote(2000, 5000, 10000).expect("failed to calculate proportion");
            assert_eq!(amount_b_optimal, 4000);
            let amount_b_optimal =
                xyk_pool_quote(1, 5000, 10000).expect("failed to calculate proportion");
            assert_eq!(amount_b_optimal, 2);
            let result = xyk_pool_quote(0, 5000, 10000).unwrap_err();
            assert_eq!(result, "insufficient amount");
            let result = xyk_pool_quote(1000, 5000, 0).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
            let result = xyk_pool_quote(1000, 0, 10000).unwrap_err();
            assert_eq!(result, "insufficient liquidity");
        }

        // TODO: tests with multiple consecutive AddLiquidity
        // TODO: tests for AddLiquidity with feeOn
    }
}

/// Query module provides functions for performing dex-related queries.
pub mod query {
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
            QueryRequest {
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Failed to get System Time.")
                    .as_millis()
                    .to_string(),
                signature: Option::None,
                query: query.into(),
            }
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
            QueryRequest {
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Failed to get System Time.")
                    .as_millis()
                    .to_string(),
                signature: Option::None,
                query: query.into(),
            }
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
            QueryRequest {
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Failed to get System Time.")
                    .as_millis()
                    .to_string(),
                signature: Option::None,
                query: query.into(),
            }
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
            QueryRequest {
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Failed to get System Time.")
                    .as_millis()
                    .to_string(),
                signature: Option::None,
                query: query.into(),
            }
        }
    }

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
            // Add indirect Token Pairs through base asset
            // Example: for actual pairs -
            // BASE:TARGET_A, BASE:TARGET_B, BASE:TARGET_C
            // query will return -
            // BASE:TARGET_A, BASE:TARGET_B, BASE:TARGET_C,
            // TARGET_A:TARGET_B, TARGET_A:TARGET_C, TARGET_B:TARGET_C
            let target_assets = token_pair_list
                .iter()
                .map(|token_pair| token_pair.id.target_asset.clone())
                .collect::<Vec<_>>();
            for token_pair in
                get_permuted_pairs(&target_assets)
                    .iter()
                    .map(|(base_asset, target_asset)| {
                        TokenPair::new(
                            DEXId::new(&base_asset.domain_name),
                            base_asset.clone(),
                            target_asset.clone(),
                            0,
                            0,
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

    /// A query to get a particular `TokenPair` identified by its id.
    pub fn query_token_pair(
        token_pair_id: <TokenPair as Identifiable>::Id,
        world_state_view: &WorldStateView,
    ) -> Option<&TokenPair> {
        get_token_pair(&token_pair_id, world_state_view).ok()
    }

    /// A query to get a list of all active `TokenPair`s of a DEX identified by its domain name.
    fn query_token_pair_list<'a>(
        domain_name: &str,
        world_state_view: &'a WorldStateView,
    ) -> Option<impl Iterator<Item = &'a TokenPair>> {
        let dex = world_state_view.read_domain(domain_name)?.dex.as_ref()?;
        Some(dex.token_pairs.iter().map(|(_, value)| value))
    }

    /// A query to get a number of `TokenPair`s of a DEX identified by its domain name.
    pub fn query_token_pair_count(
        domain_name: &str,
        world_state_view: &WorldStateView,
    ) -> Option<usize> {
        let dex = world_state_view.read_domain(domain_name)?.dex.as_ref()?;
        Some(dex.token_pairs.len())
    }
}
