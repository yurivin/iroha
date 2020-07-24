#![cfg_attr(not(feature = "std"), no_std)]

// extern crate iroha;
extern crate alloc;
/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{self as system, ensure_signed};
use alloc::{
	string::String,
	vec::Vec
};
// use frame_support::{dispatch};
// use core::{convert::TryInto, fmt};
use frame_support::{
	debug,
	//decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, traits::Get,

};
// use parity_scale_codec::{Decode, Encode};
//
// use frame_system::{
// 	self as system, ensure_none, ensure_signed,
// 	offchain::{
// 		AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer, SubmitTransaction,
// 	},
// };
// use sp_core::crypto::KeyTypeId;
// use sp_runtime::{
// 	offchain as rt_offchain,
// 	offchain::storage::StorageValueRef,
// 	transaction_validity::{
// 		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
// 		ValidTransaction,
// 	},
// };
// use sp_std::prelude::*;
// use sp_std::str;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
	// It is important to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(fn something): Option<u32>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Just a dummy event.
		/// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		/// To emit this event, we call the deposit function, from our runtime functions
		SomethingStored(u32, AccountId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		/// Just a dummy entry point.
		/// function that can be called by the external world as an extrinsics call
		/// takes a parameter of the type `AccountId`, stores it, and emits an event
		#[weight = 10_000]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let who = ensure_signed(origin)?;

			// Code to execute when something calls this.
			// For example: the following line stores the passed in u32 in the storage
			Something::put(something);

			// Here we are raising the Something event
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

		/// Another dummy entry point.
		/// takes no parameters, attempts to increment storage value, and possibly throws an error
		#[weight = 10_000]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let _who = ensure_signed(origin)?	;
			let v: Vec<u8> = Vec::new();
			let s = String::from_utf8_lossy(&*v);
			debug::error!("error: {:?}", s);

			match Something::get() {
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					Something::put(new);
					Ok(())
				},
			}
		}
	}
}

// impl<T: Trait> Module<T> {
// 	/// Add a new number to the list.
// 	fn append_or_replace_number(who: Option<T::AccountId>, number: u64) -> DispatchResult {
// 		Numbers::mutate(|numbers| {
// 			// The append or replace logic. The `numbers` vector is at most `NUM_VEC_LEN` long.
// 			let num_len = numbers.len();
//
// 			if num_len < NUM_VEC_LEN {
// 				numbers.push(number);
// 			} else {
// 				numbers[num_len % NUM_VEC_LEN] = number;
// 			}
//
// 			// displaying the average
// 			let num_len = numbers.len();
// 			let average = match num_len {
// 				0 => 0,
// 				_ => numbers.iter().sum::<u64>() / (num_len as u64),
// 			};
//
// 			debug::info!("Current average of numbers is: {}", average);
// 		});
//
// 		// Raise the NewNumber event
// 		Self::deposit_event(RawEvent::NewNumber(who, number));
// 		Ok(())
// 	}
//
// 	fn choose_tx_type(block_number: T::BlockNumber) -> TransactionType {
// 		// Decide what type of transaction to send based on block number.
// 		// Each block the offchain worker will send one type of transaction back to the chain.
// 		// First a signed transaction, then an unsigned transaction, then an http fetch and json parsing.
// 		match block_number.try_into().ok().unwrap() % 3 {
// 			0 => TransactionType::SignedSubmitNumber,
// 			1 => TransactionType::UnsignedSubmitNumber,
// 			2 => TransactionType::HttpFetching,
// 			_ => TransactionType::None,
// 		}
// 	}
//
// 	/// Check if we have fetched github info before. If yes, we use the cached version that is
// 	///   stored in off-chain worker storage `storage`. If no, we fetch the remote info and then
// 	///   write the info into the storage for future retrieval.
// 	fn fetch_if_needed() -> Result<(), Error<T>> {
// 		// Start off by creating a reference to Local Storage value.
// 		// Since the local storage is common for all offchain workers, it's a good practice
// 		// to prepend our entry with the pallet name.
// 		let s_info = StorageValueRef::persistent(b"offchain-demo::gh-info");
// 		let s_lock = StorageValueRef::persistent(b"offchain-demo::lock");
//
// 		// The local storage is persisted and shared between runs of the offchain workers,
// 		// and offchain workers may run concurrently. We can use the `mutate` function, to
// 		// write a storage entry in an atomic fashion.
// 		//
// 		// It has a similar API as `StorageValue` that offer `get`, `set`, `mutate`.
// 		// If we are using a get-check-set access pattern, we likely want to use `mutate` to access
// 		// the storage in one go.
// 		//
// 		// Ref: https://substrate.dev/rustdocs/v2.0.0-rc3/sp_runtime/offchain/storage/struct.StorageValueRef.html
// 		if let Some(Some(gh_info)) = s_info.get::<GithubInfo>() {
// 			// gh-info has already been fetched. Return early.
// 			debug::info!("cached gh-info: {:?}", gh_info);
// 			return Ok(());
// 		}
//
// 		// We are implementing a mutex lock here with `s_lock`
// 		let res: Result<Result<bool, bool>, Error<T>> = s_lock.mutate(|s: Option<Option<bool>>| {
// 			match s {
// 				// `s` can be one of the following:
// 				//   `None`: the lock has never been set. Treated as the lock is free
// 				//   `Some(None)`: unexpected case, treated it as AlreadyFetch
// 				//   `Some(Some(false))`: the lock is free
// 				//   `Some(Some(true))`: the lock is held
//
// 				// If the lock has never been set or is free (false), return true to execute `fetch_n_parse`
// 				None | Some(Some(false)) => Ok(true),
//
// 				// Otherwise, someone already hold the lock (true), we want to skip `fetch_n_parse`.
// 				// Covering cases: `Some(None)` and `Some(Some(true))`
// 				_ => Err(<Error<T>>::AlreadyFetched),
// 			}
// 		});
//
// 		// Cases of `res` returned result:
// 		//   `Err(<Error<T>>)` - lock is held, so we want to skip `fetch_n_parse` function.
// 		//   `Ok(Err(true))` - Another ocw is writing to the storage while we set it,
// 		//                     we also skip `fetch_n_parse` in this case.
// 		//   `Ok(Ok(true))` - successfully acquire the lock, so we run `fetch_n_parse`
// 		if let Ok(Ok(true)) = res {
// 			match Self::fetch_n_parse() {
// 				Ok(gh_info) => {
// 					// set gh-info into the storage and release the lock
// 					s_info.set(&gh_info);
// 					s_lock.set(&false);
//
// 					debug::info!("fetched gh-info: {:?}", gh_info);
// 				}
// 				Err(err) => {
// 					// release the lock
// 					s_lock.set(&false);
// 					return Err(err);
// 				}
// 			}
// 		}
// 		Ok(())
// 	}
//
// 	/// Fetch from remote and deserialize the JSON to a struct
// 	fn fetch_n_parse() -> Result<GithubInfo, Error<T>> {
// 		let resp_bytes = Self::fetch_from_remote().map_err(|e| {
// 			debug::error!("fetch_from_remote error: {:?}", e);
// 			<Error<T>>::HttpFetchingError
// 		})?;
//
// 		let resp_str = str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
// 		// Print out our fetched JSON string
// 		debug::info!("{}", resp_str);
//
// 		// Deserializing JSON to struct, thanks to `serde` and `serde_derive`
// 		let gh_info: GithubInfo =
// 			serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingError)?;
// 		Ok(gh_info)
// 	}
//
// 	/// This function uses the `offchain::http` API to query the remote github information,
// 	///   and returns the JSON response as vector of bytes.
// 	fn fetch_from_remote() -> Result<Vec<u8>, Error<T>> {
// 		let remote_url_bytes = HTTP_REMOTE_REQUEST_BYTES.to_vec();
// 		let user_agent = HTTP_HEADER_USER_AGENT.to_vec();
// 		let remote_url =
// 			str::from_utf8(&remote_url_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
//
// 		debug::info!("sending request to: {}", remote_url);
//
// 		// Initiate an external HTTP GET request. This is using high-level wrappers from `sp_runtime`.
// 		let request = rt_offchain::http::Request::get(remote_url);
//
// 		// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
// 		let timeout = sp_io::offchain::timestamp().add(rt_offchain::Duration::from_millis(3000));
//
// 		// For github API request, we also need to specify `user-agent` in http request header.
// 		//   See: https://developer.github.com/v3/#user-agent-required
// 		let pending = request
// 			.add_header(
// 				"User-Agent",
// 				str::from_utf8(&user_agent).map_err(|_| <Error<T>>::HttpFetchingError)?,
// 			)
// 			.deadline(timeout) // Setting the timeout time
// 			.send() // Sending the request out by the host
// 			.map_err(|_| <Error<T>>::HttpFetchingError)?;
//
// 		// By default, the http request is async from the runtime perspective. So we are asking the
// 		//   runtime to wait here.
// 		// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
// 		//   ref: https://substrate.dev/rustdocs/v2.0.0-rc3/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
// 		let response = pending
// 			.try_wait(timeout)
// 			.map_err(|_| <Error<T>>::HttpFetchingError)?
// 			.map_err(|_| <Error<T>>::HttpFetchingError)?;
//
// 		if response.code != 200 {
// 			debug::error!("Unexpected http request status code: {}", response.code);
// 			return Err(<Error<T>>::HttpFetchingError);
// 		}
//
// 		// Next we fully read the response body and collect it to a vector of bytes.
// 		Ok(response.body().collect::<Vec<u8>>())
// 	}
//
// 	fn signed_submit_number(block_number: T::BlockNumber) -> Result<(), Error<T>> {
// 		let signer = Signer::<T, T::AuthorityId>::all_accounts();
// 		if !signer.can_sign() {
// 			debug::error!("No local account available");
// 			return Err(<Error<T>>::SignedSubmitNumberError);
// 		}
//
// 		// Using `SubmitSignedTransaction` associated type we create and submit a transaction
// 		// representing the call, we've just created.
// 		// Submit signed will return a vector of results for all accounts that were found in the
// 		// local keystore with expected `KEY_TYPE`.
// 		let submission: u64 = block_number.try_into().ok().unwrap() as u64;
// 		let results = signer.send_signed_transaction(|_acct| {
// 			// We are just submitting the current block number back on-chain
// 			Call::submit_number_signed(submission)
// 		});
//
// 		for (acc, res) in &results {
// 			match res {
// 				Ok(()) => {
// 					debug::native::info!(
// 						"off-chain send_signed: acc: {:?}| number: {}",
// 						acc.id,
// 						submission
// 					);
// 				}
// 				Err(e) => {
// 					debug::error!("[{:?}] Failed in signed_submit_number: {:?}", acc.id, e);
// 					return Err(<Error<T>>::SignedSubmitNumberError);
// 				}
// 			};
// 		}
// 		Ok(())
// 	}
//
// 	fn unsigned_submit_number(block_number: T::BlockNumber) -> Result<(), Error<T>> {
// 		let submission: u64 = block_number.try_into().ok().unwrap() as u64;
// 		// Submitting the current block number back on-chain.
// 		let call = Call::submit_number_unsigned(submission);
//
// 		SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(|e| {
// 			debug::error!("Failed in unsigned_submit_number: {:?}", e);
// 			<Error<T>>::UnsignedSubmitNumberError
// 		})
// 	}
// }
