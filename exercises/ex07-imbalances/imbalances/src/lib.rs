#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

pub use pallet::*;

use sp_std::vec::Vec;

use frame_support::{
	sp_runtime::traits::{CheckedConversion, CheckedMul},
	traits::{Currency, Imbalance, SignedImbalance, TryDrop},
	transactional,
};

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NegativeBalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{ExistenceRequirement, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + scale_info::TypeInfo {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type TreasuryAccount: Get<Self::AccountId>;
		#[pallet::constant]
		type TreasuryFlatCut: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		AccountDoesNotExist,
		ImbalanceOffsetFailed,
		WithdrawalFailed,
		Overflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn mint_to(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			beneficiary: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Here we add some tokens to the chain total_issuance
			// If we do nothing more, those tokens will be removed when the `NegativeImbalance`
			// contained in the `amount_to_distribute` variable will be drop
			let amount_to_distribute = T::Currency::issue(amount);
			match T::Currency::deposit_into_existing(&beneficiary, amount_to_distribute.peek()) {
				Ok(_) => Ok(()),
				Err(_) => Err(DispatchError::Module(
					frame_support::sp_runtime::ModuleError {
						index: 2,
						error: [0, 0, 0, 0],
						message: Some("AccountDoesNotExist"),
					},
				)),
			}
			//amount_to_distribute.offset(amount_distributed);
			//amount_to_distribute.drop_zero();
			// TODO
			// We want to compensate this imbalance by increasing `benefeciary` balance by the
			// corresponding amount

			//			Ok(())
		}

		#[pallet::weight(0)]
		pub fn slash(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			target: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Todo: slash target
			let (amount_slashed, _) = T::Currency::slash(&target, amount);

			// Todo: give 1/3 of the slashed amount to the treasury and burn the rest
			// Hint: use the `ration` method
			// Hint: TreasuryAccount is defined as on l35 as a Config constant
			let (to_treasury, to_burn) = amount_slashed.ration(1, 2);
			T::Currency::burn(to_burn.peek());
			T::Currency::deposit_creating(&T::TreasuryAccount::get(), to_treasury.peek());
			// match T::Currency::deposit_into_existing(&T::TreasuryAccount::get(), to_treasury.peek()) {
			// 	Ok(_) => Ok(()),
			// 	Err(_) => Err(DispatchError::Module(frame_support::sp_runtime::ModuleError { index: 2, error: [0, 0, 0, 0], message: Some("AccountDoesNotExist") })),
			// }
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn sack(
			origin: OriginFor<T>,
			sacked_accounts: Vec<T::AccountId>,
			beneficiary: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;
			let mut to_beneficiary =
				<T::Currency as Currency<T::AccountId>>::NegativeImbalance::zero();
			let mut to_treasury =
				<T::Currency as Currency<T::AccountId>>::NegativeImbalance::zero();
			for sacked_account in sacked_accounts {
				match T::Currency::make_free_balance_be(
					&sacked_account,
					T::Currency::minimum_balance(),
				) {
					SignedImbalance::Negative(amount_slashed) => {
						let (treasury_cut, beneficiary_cut) =
							amount_slashed.split(T::TreasuryFlatCut::get());
						to_beneficiary = to_beneficiary.merge(beneficiary_cut);
						to_treasury = to_treasury.merge(treasury_cut);
					},
					_ => {
						return Err(DispatchError::Module(
							frame_support::sp_runtime::ModuleError {
								index: 2,
								error: [0, 0, 0, 0],
								message: Some("AccounttDoesNotExist"),
							},
						));
					},
				}
				// let to_slash = T::Currency::free_balance(&sacked_account) T::Currency::minimum_balance();
				// match T::Currency::withdraw(&sacked_account, total_issuance, WithdrawReasons::FEE, ExistenceRequirement::KeepAlive) {
				//     Ok(amount_slashed) => {
				// 	let (treasury_cut, beneficiary_cut) = amount_slashed.split(T::TreasuryFlatCut::get());
				// 	to_beneficiary = to_beneficiary.merge(beneficiary_cut);
				// 	to_treasury = to_treasury.merge(treasury_cut);
				//     },
				//     Err(_) => {return Err(DispatchError::Module(frame_support::sp_runtime::ModuleError { index: 2, error: [0, 0, 0, 0], message: Some("AccounttDoesNotExist") })); },
				// }
			}
			T::Currency::deposit_creating(&T::TreasuryAccount::get(), to_treasury.peek());
			//T::Currency::deposit_creating(&beneficiary, to_beneficiary.peek());

			match T::Currency::deposit_into_existing(&beneficiary, to_beneficiary.peek()) {
				Ok(_) => Ok(()),
				Err(_) => Err(DispatchError::Module(
					frame_support::sp_runtime::ModuleError {
						index: 2,
						error: [0, 0, 0, 0],
						message: Some("AccountDoesNotExist"),
					},
				)),
			}
			// Todo:
			// Take as much as possible from each account in `sacked_accounts`,
			// without removing them from existence
			// and give it all to beneficiary
			// except for the TreasuryFlatCut amount, that goes to the treasury for each sacked
			// account Hint: there is a `split` method implemented on imbalances
		}
	}
}
