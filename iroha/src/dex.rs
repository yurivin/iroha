//! This module contains functionality related to `DEX`.

use crate::prelude::*;
use bigdecimal::BigDecimal;
use iroha_derive::Io;
use parity_scale_codec::{Decode, Encode};
use std::collections::BTreeMap;
use std::str::FromStr;

const DEX_DOMAIN_NAME: &str = "dex";
const DEX_BASE_ASSET: &str = "XOR";
const MINIMUM_LIQUIDITY: u64 = 1000;

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

/// Identification of  a Token Pair. Consists of underlying asset ids.
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

#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub struct LiquiditySourceId {
    token_pair_id: <TokenPair as Identifiable>::Id,
    liquidity_source_type: LiquiditySourceType,
}

impl LiquiditySourceId {
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

#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub enum LiquiditySourceType {
    XYKPool,
    OrderBook,
}

/// `LiquiditySource` represents an exchange pair between two assets in a domain. Assets are
/// identified by their AssetDefinitionId's. Containing DEX is identified by domain name.
#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub enum LiquiditySourceData {
    XYKPool {
        pswap_asset_definition_id: <AssetDefinition as Identifiable>::Id,
        storage_account_id: <Account as Identifiable>::Id,
        pswap_total_supply: String,
        base_asset_amount: String,
        target_asset_amount: String,
        k_last: String,
        // fee_fraction
        // owner_fee_fraction
    },
    OrderBook, // this option currently is to prevent `irrefutable if-let pattern` warning
}

#[derive(Encode, Decode, Ord, PartialOrd, PartialEq, Eq, Clone, Debug, Io)]
pub struct LiquiditySource {
    id: <LiquiditySource as Identifiable>::Id,
    data: LiquiditySourceData,
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
            pswap_total_supply: BigDecimal::from(0).to_string(),
            base_asset_amount: BigDecimal::from(0).to_string(),
            target_asset_amount: BigDecimal::from(0).to_string(),
            k_last: BigDecimal::from(0).to_string(),
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
    use std::ops::Mul;

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
        AddLiquidityToXYKPool(
            <LiquiditySource as Identifiable>::Id,
            String,
            String,
            String,
            String,
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
            PermissionInstruction::CanManageDEX(authority).execute(world_state_view)?;
            let dex = self.object;
            let domain_name = self.destination_id;
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
            PermissionInstruction::CanManageDEX(authority).execute(world_state_view)?;
            let domain_name = self.destination_id.domain_name;
            let token_pair = self.object;
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
            PermissionInstruction::CanManageDEX(authority).execute(world_state_view)?;
            let token_pair_id = self.object;
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
    pub fn create_xyk_pool(
        token_pair_id: <TokenPair as Identifiable>::Id,
        // pswap_asset_definition_id: <AssetDefinition as Identifiable>::Id,
    ) -> Instruction {
        let domain_name = token_pair_id.dex_id.domain_name.clone();
        let asset_name = format!(
            "PSWAP XYK {}-{}",
            &token_pair_id.base_asset.name, &token_pair_id.target_asset.name
        );
        let pswap_asset_definition =
            AssetDefinition::new(AssetDefinitionId::new(&asset_name, &domain_name));
        let account_name = format!(
            "PSWAP XYK {}-{} STORE",
            &token_pair_id.base_asset.name, &token_pair_id.target_asset.name,
        );
        let storage_account = Account::new(&account_name, &domain_name);

        Instruction::If(
            Box::new(Instruction::ExecuteQuery(IrohaQuery::GetTokenPair(
                GetTokenPair {
                    token_pair_id: token_pair_id.clone(),
                },
            ))),
            Box::new(Instruction::Sequence(vec![
                Register {
                    // register storage account
                    object: pswap_asset_definition.clone(),
                    destination_id: domain_name.clone(),
                }
                .into(),
                Register {
                    object: storage_account.clone(),
                    destination_id: domain_name.clone(),
                }
                .into(),
                // register asset definition
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
            PermissionInstruction::CanManageDEX(authority).execute(world_state_view)?;
            let liquidity_source = self.object;
            let token_pair_id = &liquidity_source.id.token_pair_id;
            let token_pair = get_token_pair_mut(token_pair_id, world_state_view)?;
            match token_pair
                .liquidity_sources
                .entry(liquidity_source.id.clone())
            {
                Entry::Occupied(_) => {
                    Err("liquidity source already exists for token pair".to_owned())
                }
                Entry::Vacant(entry) => {
                    // TODO: register pswap token for this pool
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

    pub fn xyk_pool_add_liquidity(
        liquidity_source_id: <LiquiditySource as Identifiable>::Id,
        amount_a_desired: String,
        amount_b_desired: String,
        amount_a_min: String,
        amount_b_min: String,
    ) -> Instruction {
        Instruction::DEX(DEXInstruction::AddLiquidityToXYKPool(
            liquidity_source_id,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
        ))
    }

    fn xyk_pool_add_liquidity_execute(
        liquidity_source_id: <LiquiditySource as Identifiable>::Id,
        amount_a_desired: String,
        amount_b_desired: String,
        amount_a_min: String,
        amount_b_min: String,
        to: <Account as Identifiable>::Id,
        authority: <Account as Identifiable>::Id,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        //let domain_name = &liquidity_source_id.token_pair_id.dex_id.domain_name;
        let liquidity_source =
            get_liquidity_source(&liquidity_source_id, world_state_view)?.clone();
        let token_pair_id = &liquidity_source_id.token_pair_id;
        let dex = get_dex(&token_pair_id.dex_id.domain_name, world_state_view)?;

        let (reserve_a, reserve_b, k, total_supply) = if let LiquiditySourceData::XYKPool {
            pswap_asset_definition_id,
            storage_account_id,
            pswap_total_supply,
            base_asset_amount,
            target_asset_amount,
            k_last,
        } = liquidity_source.data.clone()
        {
            let reserve_a =
                BigDecimal::from_str(&base_asset_amount).map_err(|_| "parsing decimal failed")?;
            let reserve_b =
                BigDecimal::from_str(&target_asset_amount).map_err(|_| "parsing decimal failed")?;
            let (amount_a, amount_b) = xyk_pool_get_optimal_deposit_amounts(
                reserve_a.clone(),
                reserve_b.clone(),
                BigDecimal::from_str(&amount_a_desired).map_err(|_| "parsing decimal failed")?,
                BigDecimal::from_str(&amount_b_desired).map_err(|_| "parsing decimal failed")?,
                BigDecimal::from_str(&amount_a_min).map_err(|_| "parsing decimal failed")?,
                BigDecimal::from_str(&amount_b_min).map_err(|_| "parsing decimal failed")?,
            )?;
            //todo!("transfer target must be pool storage account");
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
            let (reserve_a, reserve_b, k, total_supply) = mint_pswap_with_fee(
                pswap_asset_definition_id.clone(),
                to,
                amount_a,
                amount_b,
                reserve_a,
                reserve_b,
                BigDecimal::from_str(&k_last).map_err(|_| "parsing decimal failed")?,
                None,
                BigDecimal::from_str(&pswap_total_supply).map_err(|_| "parsing decimal failed")?,
                authority,
                world_state_view,
            )?;
            // update values in liquidity pool
            (
                reserve_a.to_string(),
                reserve_b.to_string(),
                k.to_string(),
                total_supply.to_string(),
            )
        } else {
            return Err("wrong liquidity source: adding liquidity to xyk pool".to_owned());
        };
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
        }
        Ok(())
    }

    /// Based on given reserves, desired and minimal amounts to add liquidity, either return
    /// optimal values (needed to preserve reserves proportion) or error if it's not possible
    /// to keep proportion with proposed amounts.
    fn xyk_pool_get_optimal_deposit_amounts(
        reserve_a: BigDecimal,
        reserve_b: BigDecimal,
        amount_a_desired: BigDecimal,
        amount_b_desired: BigDecimal,
        amount_a_min: BigDecimal,
        amount_b_min: BigDecimal,
    ) -> Result<(BigDecimal, BigDecimal), String> {
        Ok(
            if reserve_a == BigDecimal::from(0) && reserve_b == BigDecimal::from(0) {
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
                    // if !(amount_a_optimal <= amount_a_desired) {return Err("assertion".to_owned())}
                    assert!(amount_a_optimal <= amount_a_desired); // TODO: don't use assert
                    if !(amount_a_optimal >= amount_a_min) {
                        return Err("insufficient a amount".to_owned());
                    }
                    (amount_a_optimal, amount_b_desired)
                }
            },
        )
    }

    /// Given some amount of an Asset and pair reserves, returns an equivalent amount of the other Asset.
    fn xyk_pool_quote(
        amount_a: BigDecimal,
        reserve_a: BigDecimal,
        reserve_b: BigDecimal,
    ) -> Result<BigDecimal, String> {
        if !(amount_a > BigDecimal::from(0)) {
            return Err("insufficient amount".to_owned());
        }
        if !(reserve_a > BigDecimal::from(0) && reserve_b > BigDecimal::from(0)) {
            return Err("insufficient liquidity".to_owned());
        }
        Ok(amount_a.mul(reserve_b) / reserve_a) // calculate amount_b via proportion
    }

    fn transfer_from(
        token: <AssetDefinition as Identifiable>::Id,
        from: <Account as Identifiable>::Id,
        to: <Account as Identifiable>::Id,
        value: BigDecimal,
        authority: <Account as Identifiable>::Id,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        let quantity: u32 = 42; // TODO: convertion
        let asset_id = AssetId::new(token, from.clone());
        AccountInstruction::TransferAsset(from, to, Asset::with_quantity(asset_id, quantity))
            .execute(authority, world_state_view)
    }

    /// Mint pswap tokens representing liquidity in pool for depositing account.
    /// returns ()
    fn mint_pswap_with_fee(
        pswap_asset_definition_id: <AssetDefinition as Identifiable>::Id,
        to: <Account as Identifiable>::Id,
        amount_a: BigDecimal,
        amount_b: BigDecimal,
        reserve_a: BigDecimal,
        reserve_b: BigDecimal,
        k_last: BigDecimal,
        fee_to: Option<<Account as Identifiable>::Id>,
        total_supply: BigDecimal,
        authority: <Account as Identifiable>::Id,
        world_state_view: &mut WorldStateView,
    ) -> Result<(BigDecimal, BigDecimal, BigDecimal, BigDecimal), String> {
        let (mut k_last, total_supply) = mint_pswap_fee(
            pswap_asset_definition_id.clone(),
            reserve_a.clone(),
            reserve_b.clone(),
            k_last.clone(),
            fee_to.clone(),
            authority.clone(),
            world_state_view,
        )?;

        let mut liquidity = BigDecimal::from(0);
        if total_supply == BigDecimal::from(0) {
            liquidity = amount_a
                .clone()
                .mul(amount_b.clone())
                .sqrt()
                .ok_or("negative value provided")?
                - BigDecimal::from(MINIMUM_LIQUIDITY);
            //TODO: does minimum liquidity needs to be locked?
            // mint (address(0), MINIMUM_LIQUIDITY)
            todo!("if min liquditity is not locked, total_supply will not change correctly, need to mint to zero address!")
        } else {
            liquidity = (amount_a.clone().mul(total_supply.clone()) / reserve_a)
                .min(amount_b.clone().mul(total_supply.clone()) / reserve_b);
        }
        if !(liquidity > BigDecimal::from(0)) {
            return Err("insufficient liquidity minted".to_owned());
        }
        let total_supply = mint_pswap(
            pswap_asset_definition_id,
            to,
            liquidity,
            total_supply,
            authority,
            world_state_view,
        )?;
        let (mut reserve_a, mut reserve_b) = (amount_a, amount_b);
        if fee_to.is_some() {
            k_last = reserve_a.clone().mul(reserve_b.clone());
        }
        Ok((reserve_a, reserve_b, k_last, total_supply))
    }

    /// returns (k_last, total_supply)
    fn mint_pswap_fee(
        asset_definition_id: <AssetDefinition as Identifiable>::Id,
        reserve_a: BigDecimal,
        reserve_b: BigDecimal,
        k_last: BigDecimal,
        fee_to: Option<<Account as Identifiable>::Id>,
        authority: <Account as Identifiable>::Id,
        world_state_view: &mut WorldStateView,
    ) -> Result<(BigDecimal, BigDecimal), String> {
        let mut total_supply = BigDecimal::from(0);
        if let Some(fee_to) = fee_to {
            if k_last != BigDecimal::from(0) {
                let root_k = reserve_a
                    .mul(reserve_b)
                    .sqrt()
                    .ok_or("negative value provided")?;
                let root_k_last = k_last.sqrt().ok_or("negative value provided")?;
                if root_k > root_k_last {
                    let numerator = total_supply
                        .clone()
                        .mul(root_k.clone() - root_k_last.clone());
                    let demonimator = root_k.mul(BigDecimal::from(5)) + root_k_last;
                    let liquidity = numerator / demonimator;
                    if liquidity > BigDecimal::from(0) {
                        total_supply = mint_pswap(
                            asset_definition_id,
                            fee_to,
                            liquidity,
                            total_supply,
                            authority,
                            world_state_view,
                        )?;
                    }
                }
            }
        } else if k_last != BigDecimal::from(0) {
            return Ok((BigDecimal::from(0), total_supply));
        }
        Ok((k_last, total_supply))
    }

    ///
    /// returns total_supply
    fn mint_pswap(
        asset_definition_id: <AssetDefinition as Identifiable>::Id,
        to: <Account as Identifiable>::Id,
        value: BigDecimal,
        total_supply: BigDecimal,
        authority: <Account as Identifiable>::Id,
        world_state_view: &mut WorldStateView,
    ) -> Result<BigDecimal, String> {
        let quantity: u32 = 42; // TODO: conversion
        let asset_id = AssetId::new(asset_definition_id, to);
        AssetInstruction::MintAsset(quantity, asset_id).execute(authority, world_state_view)?;
        Ok(total_supply + value)
    }

    /// Register new `Account` in domain. This function should
    /// be called from contract which performs necessary permission checks.
    fn register_account_unchecked(
        account: Account,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        let domain = get_domain_mut(&account.id.domain_name, world_state_view)?;
        match domain.accounts.entry(account.id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(account);
                Ok(())
            }
            Entry::Occupied(_) => Err("account already exists".to_owned()),
        }
    }

    /// Register new `Asset` in domain. This function should
    /// be called from contract which performs necessary permission checks.
    fn register_asset_unchecked(
        asset_definition: AssetDefinition,
        world_state_view: &mut WorldStateView,
    ) -> Result<(), String> {
        let domain = get_domain_mut(&asset_definition.id.domain_name, world_state_view)?;
        match domain.asset_definitions.entry(asset_definition.id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(asset_definition);
                Ok(())
            }
            Entry::Occupied(_) => Err("asset definition already exists".to_owned()),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::dex::query::*;
        use crate::peer::PeerId;
        use crate::permission::{permission_asset_definition_id, Permission};
        use crate::query::QueryResult;
        use std::collections::BTreeMap;

        struct TestKit {
            world_state_view: WorldStateView,
            root_account_id: <Account as Identifiable>::Id,
        }

        impl TestKit {
            pub fn new() -> Self {
                let domain_name = "Company".to_string();
                let key_pair = KeyPair::generate().expect("Failed to generate KeyPair.");
                let mut asset_definitions = BTreeMap::new();
                let asset_definition_id = permission_asset_definition_id();
                asset_definitions.insert(
                    asset_definition_id.clone(),
                    AssetDefinition::new(asset_definition_id.clone()),
                );
                let account_id = AccountId::new("root", &domain_name);
                let asset_id = AssetId {
                    definition_id: asset_definition_id,
                    account_id: account_id.clone(),
                };
                let asset = Asset::with_permission(asset_id.clone(), Permission::Anything);
                let mut account = Account::with_signatory(
                    &account_id.name,
                    &account_id.domain_name,
                    key_pair.public_key.clone(),
                );
                account.assets.insert(asset_id.clone(), asset);
                let mut accounts = BTreeMap::new();
                accounts.insert(account_id.clone(), account);
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
                    root_account_id: account_id,
                }
            }
        }

        #[test]
        fn test_initialize_dex_should_pass() {
            let mut testkit = TestKit::new();
            // create dex owner account
            let dex_owner_public_key = KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key;
            let domain_name = Name::from("Company");
            let mut dex_owner_account =
                Account::with_signatory("dex_owner", &domain_name, dex_owner_public_key);

            // create permissions to operate any dex
            let asset_definition = permission_asset_definition_id();
            let asset_id = AssetId {
                definition_id: asset_definition,
                account_id: dex_owner_account.id.clone(),
            };
            let manage_dex_permission =
                Asset::with_permission(asset_id.clone(), Permission::ManageDEX);
            dex_owner_account
                .assets
                .insert(asset_id.clone(), manage_dex_permission);

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

            initialize_dex(&domain_name, dex_owner_account.id.clone())
                .execute(dex_owner_account.id.clone(), world_state_view)
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
            // create dex owner account
            let dex_owner_public_key = KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key;
            let domain_name = Name::from("Company");
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

            assert_eq!(
                initialize_dex(
                    &domain_name,
                    dex_owner_account.id.clone(),
                ).execute(
                    dex_owner_account.id.clone(),
                    world_state_view
                ).unwrap_err(),
                format!("Error: Permission not found., CanManageDEX(Id {{ name: \"{}\", domain_name: \"{}\" }})",
                        "dex_owner", &domain_name)
            );
        }

        #[test]
        fn create_and_delete_token_pair_should_pass() {
            let mut testkit = TestKit::new();
            // create dex owner account
            let dex_owner_public_key = KeyPair::generate()
                .expect("Failed to generate KeyPair.")
                .public_key;
            let domain_name = Name::from("Company");
            let mut dex_owner_account =
                Account::with_signatory("dex_owner", &domain_name, dex_owner_public_key);

            // create permissions to operate any dex
            let asset_definition = permission_asset_definition_id();
            let asset_id = AssetId {
                definition_id: asset_definition,
                account_id: dex_owner_account.id.clone(),
            };
            let manage_dex_permission =
                Asset::with_permission(asset_id.clone(), Permission::ManageDEX);
            dex_owner_account
                .assets
                .insert(asset_id.clone(), manage_dex_permission);

            // get world state view and dex domain
            let world_state_view = &mut testkit.world_state_view;
            let domain = world_state_view
                .read_domain(&domain_name)
                .expect("domain not found")
                .clone();

            // register dex owner account
            let register_account = domain.register_account(dex_owner_account.clone());
            register_account
                .execute(testkit.root_account_id.clone(), world_state_view)
                .expect("failed to register dex owner account");

            // initialize dex in domain
            initialize_dex(&domain_name, dex_owner_account.id.clone())
                .execute(dex_owner_account.id.clone(), world_state_view)
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
            .execute(dex_owner_account.id.clone(), world_state_view)
            .expect("create token pair failed");

            let token_pair_id = TokenPairId::new(
                DEXId::new(&domain_name),
                asset_definition_a.id.clone(),
                asset_definition_b.id.clone(),
            );
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
                .execute(dex_owner_account.id.clone(), world_state_view)
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
    }
}

// /// Iroha World State View module provides extensions for the `WorldStateView` for initializing
// /// a DEX in particular Domain.
// pub mod wsv {
//     use super::*;
//
//     impl WorldStateView {
//         /// Get `DEX` without an ability to modify it.
//         pub fn read_dex(&self, domain_name: &str) -> Option<&DEX> {
//             self.read_peer().domains.get(domain_name)?.dex.as_ref()
//         }
//
//         /// Get `DEX` with an ability to modify it.
//         pub fn dex(&mut self, domain_name: &str) -> Option<&mut DEX> {
//             self.peer().domains.get_mut(domain_name)?.dex.as_mut()
//         }
//     }
// }

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
