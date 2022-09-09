#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, Randomness, ReservableCurrency};
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::ExistenceRequirement;
	use frame_support::PalletId;
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One};

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type KittyIndex: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaxEncodedLen
			+ TypeInfo
			+ Decode
			+ Encode
			+ From<u32>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		#[pallet::constant]
		type CreatKittyDeposit: Get<BalanceOf<Self>>;
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_kitty)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn get_account_to_kitties)]
	pub type AccountToKitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::KittyIndex>>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_list_for_sales)]
	pub type ListForSale<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, (T::AccountId, BalanceOf<T>)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex, Kitty),
		KittyBreed(T::AccountId, T::KittyIndex, T::KittyIndex),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
		KittyListed(T::AccountId, T::KittyIndex, BalanceOf<T>),
		KittyBuyed(T::AccountId, T::AccountId, T::KittyIndex, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		NotOwner,
		SameKittyId,
		NotForSale,
		NotEnoughBalanceForBuying,
		NotEnoughBalanceForCreating,
		NotEnoughBalanceForBreeding,
		NotEnoughBalanceForReceiving,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id();

			let dna = Self::random_value(&who);
			let kitty = Kitty(dna);

			let free_balance = T::Currency::free_balance(&who);
			ensure!(
				free_balance >= T::CreatKittyDeposit::get(),
				Error::<T>::NotEnoughBalanceForCreating
			);
			T::Currency::reserve(&who, T::CreatKittyDeposit::get())?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			//存入之前做检查
			NextKittyId::<T>::set(
				kitty_id.checked_add(&One::one()).ok_or(Error::<T>::InvalidKittyId)?,
			);
			Self::add_account_to_kitties(who.clone(), kitty_id)?;
			// Emit an event.
			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// check kitty id
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);

			let kitty_1 = Self::get_kitty(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			let free_balance = T::Currency::free_balance(&who);
			ensure!(
				free_balance >= T::CreatKittyDeposit::get(),
				Error::<T>::NotEnoughBalanceForBreeding
			);
			T::Currency::reserve(&who, T::CreatKittyDeposit::get())?;

			// get next id
			let kitty_id = Self::next_kitty_id();

			// selector for breeding
			let selector = Self::random_value(&who);

			let mut data = [0u8; 16];
			for i in 0..kitty_1.0.len() {
				// 0 choose kitty2, and 1 choose kitty1
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			}
			let new_kitty = Kitty(data);

			<Kitties<T>>::insert(kitty_id, &new_kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			NextKittyId::<T>::set(
				kitty_id.checked_add(&One::one()).ok_or(Error::<T>::InvalidKittyId)?,
			);
			Self::add_account_to_kitties(who.clone(), kitty_id)?;

			Self::deposit_event(Event::KittyBreed(who, kitty_id_1, kitty_id_2));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::get_kitty(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);

			let free_balance = T::Currency::free_balance(&new_owner);
			ensure!(
				free_balance >= T::CreatKittyDeposit::get(),
				Error::<T>::NotEnoughBalanceForReceiving
			);
			T::Currency::reserve(&new_owner, T::CreatKittyDeposit::get())?;

			T::Currency::unreserve(&who, T::CreatKittyDeposit::get());

			<KittyOwner<T>>::insert(kitty_id, new_owner.clone());
			Self::add_account_to_kitties(new_owner.clone(), kitty_id)?;
			Self::remove_account_to_kitties(who.clone(), kitty_id)?;
			Self::deposit_event(Event::KittyTransferred(who, new_owner, kitty_id));

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn sell(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(Some(who.clone()) == KittyOwner::<T>::get(kitty_id), Error::<T>::NotOwner);

			ListForSale::<T>::mutate_exists(kitty_id, |p| *p = Some((who.clone(), price)));
			KittyOwner::<T>::insert(kitty_id, Self::account_id());
			Self::add_account_to_kitties(Self::account_id(), kitty_id)?;
			Self::remove_account_to_kitties(who.clone(), kitty_id)?;

			Self::deposit_event(Event::KittyListed(who, kitty_id, price));

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			let (owner, amount) = ListForSale::<T>::get(kitty_id).ok_or(Error::<T>::NotForSale)?;

			let buyer_balance = T::Currency::free_balance(&buyer);
			ensure!(
				buyer_balance >= amount + T::CreatKittyDeposit::get(),
				Error::<T>::NotEnoughBalanceForBuying
			);
			T::Currency::reserve(&buyer, T::CreatKittyDeposit::get())?;
			T::Currency::unreserve(&owner, T::CreatKittyDeposit::get());

			T::Currency::transfer(&buyer, &owner, amount, ExistenceRequirement::KeepAlive)?;

			ListForSale::<T>::remove(kitty_id);
			KittyOwner::<T>::insert(kitty_id, buyer.clone());
			Self::add_account_to_kitties(buyer.clone(), kitty_id)?;
			Self::remove_account_to_kitties(Self::account_id(), kitty_id)?;

			Self::deposit_event(Event::KittyBuyed(owner, buyer, kitty_id, amount));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// get a random 256.
		pub fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128)
		}

		pub fn account_id() -> T::AccountId {
			<T as Config>::PalletId::get().into_account_truncating()
		}

		pub fn add_account_to_kitties(
			who: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let empty_vec: Vec<T::KittyIndex> = Vec::new();
			if Self::get_account_to_kitties(who.clone()) == None {
				AccountToKitties::<T>::insert(who.clone(), empty_vec);
			}
			AccountToKitties::<T>::mutate(who.clone(), |kitties| -> Result<(), Error<T>> {
				match kitties {
					Some(kitty_list) if !kitty_list.contains(&kitty_id) => {
						kitty_list.push(kitty_id.clone());
						Ok(())
					},
					_ => Err(Error::<T>::InvalidKittyId),
				}
			})?;
			Ok(())
		}
		pub fn remove_account_to_kitties(
			who: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			AccountToKitties::<T>::mutate(who.clone(), |kitties| -> Result<(), Error<T>> {
				match kitties {
					Some(kitty_list) if kitty_list.contains(&kitty_id) => {
						kitty_list.retain(|x| x.clone() != kitty_id);
						Ok(())
					},
					_ => Err(Error::<T>::InvalidKittyId),
				}
			})?;
			Ok(())
		}
	}
}
