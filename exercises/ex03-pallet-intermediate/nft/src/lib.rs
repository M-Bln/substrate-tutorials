#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod tests;
pub mod types;

use frame_support::ensure;
use std::cmp;
use types::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::sp_runtime;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + scale_info::TypeInfo {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn unique_asset)]
	pub(super) type UniqueAsset<T: Config> =
		StorageMap<_, Blake2_128Concat, UniqueAssetId, UniqueAssetDetails<T, T::MaxLength>>;

	#[pallet::storage]
	#[pallet::getter(fn account)]
	/// The holdings of a specific account for a specific asset.
	pub(super) type Account<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		UniqueAssetId,
		Blake2_128Concat,
		T::AccountId,
		u128,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	/// Nonce for id of the next created asset
	pub(super) type Nonce<T: Config> = StorageValue<_, UniqueAssetId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New unique asset created
		Created {
			creator: T::AccountId,
			asset_id: UniqueAssetId,
		},
		/// Some assets have been burned
		Burned {
			asset_id: UniqueAssetId,
			owner: T::AccountId,
			total_supply: u128,
		},
		/// Some assets have been transferred
		Transferred {
			asset_id: UniqueAssetId,
			from: T::AccountId,
			to: T::AccountId,
			amount: u128,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The asset ID is unknown
		UnknownAssetId,
		/// The signing account does not own any amount of this asset
		NotOwned,
		/// Supply must be positive
		NoSupply,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn mint(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, T::MaxLength>,
			supply: u128,
		) -> DispatchResult {
			let origin_account_id = ensure_signed(origin)?;
			if supply <= 0 {
				return Err(DispatchError::from(Error::<T>::NoSupply));
			}
			let asset_id: UniqueAssetId = Nonce::<T>::get();
			Nonce::<T>::set(asset_id + 1);
			let asset_details = UniqueAssetDetails::<T, T::MaxLength>::new(
				origin_account_id.clone(),
				metadata,
				supply,
			);
			UniqueAsset::<T>::set(asset_id, Some(asset_details));
			Account::<T>::set(asset_id, origin_account_id.clone(), supply);
			Self::deposit_event(Event::<T>::Created {
				creator: origin_account_id,
				asset_id,
			});
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn burn(origin: OriginFor<T>, asset_id: UniqueAssetId, amount: u128) -> DispatchResult {
			// - Ensure the extrinsic origin is a signed transaction.
			let origin = ensure_signed(origin)?;

			// - Mutate the total supply.
			let old_supply = Account::<T>::get(asset_id, origin.clone());
			let burned_amount = cmp::min(amount, old_supply);
			UniqueAsset::<T>::try_mutate(asset_id, |maybe_details| -> DispatchResult {
				let details = maybe_details.as_mut().ok_or(Error::<T>::UnknownAssetId)?;
				details.supply -= burned_amount;
				Ok(())
			})?;
			if old_supply == 0 {
				return Err(DispatchError::from(Error::<T>::NotOwned));
			}
			// - Mutate the account balance.
			Account::<T>::mutate(asset_id, origin.clone(), |balance| {
				*balance = balance.saturating_sub(amount);
			});

			Self::deposit_event(Event::Burned {
				asset_id,
				owner: origin,
				total_supply: old_supply - burned_amount,
			});
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			asset_id: UniqueAssetId,
			amount: u128,
			to: T::AccountId,
		) -> DispatchResult {
			// TODO:
			// - Ensure the extrinsic origin is a signed transaction.
			let origin = ensure_signed(origin)?;

			if let Some(_) = UniqueAsset::<T>::get(asset_id) {
				// - Mutate both account balances.
				let old_supply = Account::<T>::get(asset_id, origin.clone());
				if old_supply <= 0 {
					return Err(DispatchError::from(Error::<T>::NotOwned));
				}
				let transferred_amount = cmp::min(amount, old_supply);

				Account::<T>::mutate(asset_id, origin.clone(), |balance| {
					*balance -= transferred_amount;
				});

				Account::<T>::mutate(asset_id, to.clone(), |balance| {
					*balance += transferred_amount;
				});
				// - Emit a `Transferred` event.
				Self::deposit_event(Event::Transferred {
					asset_id,
					from: origin,
					to,
					amount: transferred_amount,
				});
				Ok(())
			} else {
				Err(DispatchError::Module(sp_runtime::ModuleError {
					index: 1,
					error: [0, 0, 0, 0],
					message: Some("UnknownAssetId"),
				}))
			}
		}
	}
}
