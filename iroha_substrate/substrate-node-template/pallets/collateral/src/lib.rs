#![deny(warnings)]
#![cfg_attr(test, feature(proc_macro_hygiene))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::traits::{Currency, ReservableCurrency};
/// The Collateral module according to the specification at
/// https://interlay.gitlab.io/polkabtc-spec/spec/collateral.html
use frame_support::{decl_event, decl_module, decl_storage, ensure, sp_runtime::ModuleId};
// use x_core::Error;

pub type BalanceOf<T> = <<T as Trait>::XOR as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The collateral's module id, used for deriving its sovereign account ID.
const _MODULE_ID: ModuleId = ModuleId(*b"ily/cltl");

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    /// The XOR currency
    type XOR: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Collateral {
        /// ## Storage
        /// Note that account's balances and locked balances are handled
        /// through the Balances module.
        ///
        /// Total locked XOR collateral
        TotalCollateral: BalanceOf<T>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        LockCollateral(AccountId, Balance),
        ReleaseCollateral(AccountId, Balance),
        SlashCollateral(AccountId, AccountId, Balance),
    }
);

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        fn deposit_event() = default;
    }
}

impl<T: Trait> Module<T> {
    /// Total supply of XOR
    pub fn get_total_supply() -> BalanceOf<T> {
        T::XOR::total_issuance()
    }
    /// Total locked XOR collateral
    pub fn get_total_collateral() -> BalanceOf<T> {
        <TotalCollateral<T>>::get()
    }
    /// Increase the locked collateral
    pub fn increase_total_collateral(amount: BalanceOf<T>) {
        let new_collateral = Self::get_total_collateral() + amount;
        <TotalCollateral<T>>::put(new_collateral);
    }
    /// Decrease the locked collateral
    pub fn decrease_total_collateral(amount: BalanceOf<T>) {
        let new_collateral = Self::get_total_collateral() - amount;
        <TotalCollateral<T>>::put(new_collateral);
    }
    /// Balance of an account (wrapper)
    pub fn get_balance_from_account(account: &T::AccountId) -> BalanceOf<T> {
        T::XOR::free_balance(account)
    }
    /// Locked balance of account
    pub fn get_collateral_from_account(account: &T::AccountId) -> BalanceOf<T> {
        T::XOR::reserved_balance(account)
    }

    /// Lock XOR collateral
    ///
    /// # Arguments
    ///
    /// * `sender` - the account locking tokens
    /// * `amount` - to be locked amount of XOR
    pub fn lock_collateral(sender: &T::AccountId, amount: BalanceOf<T>) -> Result<(), Error> {
        T::XOR::reserve(sender, amount).map_err(|_| Error::InsufficientFunds)?;

        Self::increase_total_collateral(amount);

        Self::deposit_event(RawEvent::LockCollateral(sender.clone(), amount));
        Ok(())
    }

    /// Release XOR collateral
    ///
    /// # Arguments
    ///
    /// * `sender` - the account releasing tokens
    /// * `amount` - the to be released amount of XOR
    pub fn release_collateral(sender: &T::AccountId, amount: BalanceOf<T>) -> Result<(), Error> {
        ensure!(
            T::XOR::reserved_balance(&sender) >= amount,
            Error::InsufficientCollateralAvailable
        );
        T::XOR::unreserve(sender, amount);

        Self::decrease_total_collateral(amount);

        Self::deposit_event(RawEvent::ReleaseCollateral(sender.clone(), amount));

        Ok(())
    }

    /// Slash XOR collateral and assign to a receiver. Can only fail if
    /// the sender account has too low collateral.
    ///
    /// # Arguments
    ///
    /// * `sender` - the account being slashed
    /// * `receiver` - the receiver of the amount
    /// * `amount` - the to be slashed amount
    pub fn slash_collateral(
        sender: T::AccountId,
        receiver: T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<(), Error> {
        ensure!(
            T::XOR::reserved_balance(&sender) >= amount,
            Error::InsufficientCollateralAvailable
        );

        // slash the sender's collateral
        let (slashed, _remainder) = T::XOR::slash_reserved(&sender, amount);

        // add slashed amount to receiver and create account if it does not exists
        T::XOR::resolve_creating(&receiver, slashed);

        // reserve the created amount for the receiver
        T::XOR::reserve(&receiver, amount).map_err(|_| Error::InsufficientFunds)?;

        Self::deposit_event(RawEvent::SlashCollateral(sender, receiver, amount));

        Ok(())
    }
}
