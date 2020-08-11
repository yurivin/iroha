#![deny(warnings)]
#![cfg_attr(test, feature(proc_macro_hygiene))]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
extern crate mocktopus;

// #[cfg(test)]
// use mocktopus::macros::mockable;

use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive, ReservableCurrency};
/// # XOR Treasury implementation
/// The Treasury module according to the specification at
/// https://interlay.gitlab.io/XOR-spec/spec/treasury.html
// Substrate
use frame_support::{decl_event, decl_module, decl_error, decl_storage, dispatch::DispatchResult, ensure};
use sp_runtime::ModuleId;
use system::ensure_signed;
// use x_core::Error;

pub type BalanceOf<T> = <<T as Trait>::XOR as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The treasury's module id, used for deriving its sovereign account ID.
const _MODULE_ID: ModuleId = ModuleId(*b"ily/trsy");

/// The pallet's configuration trait.
/// Instantiation of this pallet requires the existence of a module that
/// implements Currency and ReservableCurrency. The Balances module can be used
/// for this. The Balances module then gives functions for total supply, balances
/// of accounts, and any function defined by the Currency and ReservableCurrency
/// traits.
pub trait Trait: system::Trait {
    /// The XOR currency
    type XOR: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        InsufficientLockedFunds,
        InsufficientFunds,
    }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Treasury {
        /// ## Storage
        /// Note that account's balances and locked balances are handled
        /// through the Balances module.
        ///
        /// Total locked PolkaDOT
        TotalLocked: BalanceOf<T>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        Transfer(AccountId, AccountId, Balance),
        Mint(AccountId, Balance),
        Lock(AccountId, Balance),
        Burn(AccountId, Balance),
    }
);

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your pallet
        fn deposit_event() = default;

        /// Transfer an amount of XOR (without fees)
        ///
        /// # Arguments
        ///
        /// * `origin` - sender of the transaction
        /// * `receiver` - receiver of the transaction
        /// * `amount` - amount of XOR
        #[weight = 1000]
        fn transfer(origin, receiver: T::AccountId, amount: BalanceOf<T>)
            -> DispatchResult
        {
            let sender = ensure_signed(origin)?;

            T::XOR::transfer(&sender, &receiver, amount, KeepAlive)
                .map_err(|_| <Error<T>>::InsufficientFunds)?;

            Self::deposit_event(RawEvent::Transfer(sender, receiver, amount));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Total supply of XOR
    pub fn get_total_supply() -> BalanceOf<T> {
        T::XOR::total_issuance()
    }
    /// Balance of an account (wrapper)
    pub fn get_balance_from_account(account: T::AccountId) -> BalanceOf<T> {
        T::XOR::free_balance(&account)
    }
    /// Locked balance of an account (wrapper)
    pub fn get_locked_balance_from_account(account: T::AccountId) -> BalanceOf<T> {
        T::XOR::reserved_balance(&account)
    }
    /// Increase the supply of locked XOR
    pub fn increase_total_locked(amount: BalanceOf<T>) {
        let new_locked = <TotalLocked<T>>::get() + amount;
        <TotalLocked<T>>::put(new_locked);
    }
    /// Decrease the supply of locked XOR
    pub fn decrease_total_locked(amount: BalanceOf<T>) {
        let new_locked = <TotalLocked<T>>::get() - amount;
        <TotalLocked<T>>::put(new_locked);
    }
    /// Mint new tokens
    ///
    /// # Arguments
    ///
    /// * `requester` - XOR user requesting new tokens
    /// * `amount` - to be issued amount of XOR
    pub fn mint(requester: T::AccountId, amount: BalanceOf<T>) {
        // adds the amount to the total balance of tokens
        let minted_tokens = T::XOR::issue(amount);
        // adds the added amount to the requester's balance
        T::XOR::resolve_creating(&requester, minted_tokens);

        Self::deposit_event(RawEvent::Mint(requester, amount));
    }

    /// Lock XOR tokens to burn them. Note: this removes them from the
    /// free balance of XOR and adds them to the locked supply of XOR.
    ///
    /// # Arguments
    ///
    /// * `redeemer` - the account redeeming tokens
    /// * `amount` - to be locked amount of XOR
    pub fn lock(redeemer: T::AccountId, amount: BalanceOf<T>) -> Result<(), Error<T>> {
        T::XOR::reserve(&redeemer, amount).map_err(|_| <Error<T>>::InsufficientFunds)?;

        // update total locked balance
        Self::increase_total_locked(amount);

        Self::deposit_event(RawEvent::Lock(redeemer, amount));
        Ok(())
    }

    /// Burn previously locked XOR tokens
    ///
    /// # Arguments
    ///
    /// * `redeemer` - the account redeeming tokens
    /// * `amount` - the to be burned amount of XOR
    pub fn burn(redeemer: T::AccountId, amount: BalanceOf<T>) -> Result<(), Error<T>> {
        ensure!(
            T::XOR::reserved_balance(&redeemer) >= amount,
            <Error<T>>::InsufficientLockedFunds
        );

        // burn the tokens from the locked balance
        Self::decrease_total_locked(amount);

        // burn the tokens for the redeemer
        // remainder should always be 0 and is checked above
        let (_burned_tokens, _remainder) = T::XOR::slash_reserved(&redeemer, amount);

        Self::deposit_event(RawEvent::Burn(redeemer, amount));

        Ok(())
    }
}
