#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::DispatchResult;
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransfered(T::AccountId, T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyClaimed,
		ClaimToolong,
		ClaimNotExist,
		NotClaimOwner,
	}

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub(super) type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimToolong)?;
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyClaimed);

			let current_block = <frame_system::Pallet<T>>::block_number();

			Proofs::<T>::insert(&bounded_claim, (sender.clone(), current_block));

			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimToolong)?;

			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(sender == owner, Error::<T>::NotClaimOwner);

			Proofs::<T>::remove(&bounded_claim);

			Self::deposit_event(Event::ClaimRevoked(sender, claim));
			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			receiver: T::AccountId,
			claim: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimToolong)?;
			Proofs::<T>::try_mutate_exists(&bounded_claim, |proofs| {
				let proof = proofs.as_mut().ok_or(Error::<T>::ClaimNotExist)?;

				ensure!(proof.0 == sender, Error::<T>::NotClaimOwner);
				proof.0 = receiver.clone();

				Self::deposit_event(Event::ClaimTransfered(sender, receiver, claim));

				Ok(())
			})
		}
	}
}
