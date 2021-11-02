#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::{
		sp_runtime::traits::Hash,
		traits::{tokens::ExistenceRequirement, Currency, Randomness},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_io::hashing::blake2_128;

	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		//使用事件
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//使用余额相关
		type Currency: Currency<Self::AccountId>;
		//使用随机数
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
		//定义一个常量
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;
	}
	//kitties拥有者账户
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	//kitties的价格
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	// kitties ID=》kitties 详细信息
	pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	// 账户=》kitties ID列表
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxKittyOwned>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	//已经创建了多少个小猫，跟踪总数
	pub(super) type KittyCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//创建[sender, kitty_id]
		Created(T::AccountId, T::Hash),
		//价格被设置[sender, kitty_id, new_price]
		PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
		//猫咪被转走[from, to, kitty_id]
		Transferred(T::AccountId, T::AccountId, T::Hash),
		//猫咪被买[buyer, seller, kitty_id, bid_price]
		Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Handles arithemtic overflow when incrementing the Kitty counter.
		KittyCntOverflow,
		/// An account cannot own more Kitties than `MaxKittyCount`.
		ExceedMaxKittyOwned,
		/// Buyer cannot be the owner.
		BuyerIsKittyOwner,
		/// Cannot transfer a kitty to its owner.
		TransferToSelf,
		/// Handles checking whether the Kitty exists.
		KittyNotExist,
		/// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.
		NotKittyOwner,
		/// Ensures the Kitty is for sale.
		KittyNotForSale,
		/// Ensures that the buying price is greater than the asking price.
		KittyBidPriceTooLow,
		/// Ensures that an account has enough funds to purchase a Kitty.
		NotEnoughBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?; // <- add this line
			let kitty_id = Self::mint(&sender, None, None)?; // <- add this line
												 // Logging to the console
			log::info!("A kitty is born with ID: {:?}.", kitty_id); // <- add this line

			Self::deposit_event(Event::Created(sender, kitty_id));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}
		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}
		// Helper to mint a Kitty.
		pub fn mint(
			owner: &T::AccountId,
			dna: Option<[u8; 16]>,
			gender: Option<Gender>,
		) -> Result<T::Hash, Error<T>> {
			let kitty = Kitty::<T> {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone(),
			};

			let kitty_id = T::Hashing::hash_of(&kitty);

			// Performs this operation first as it may fail
			let new_cnt = Self::kitty_cnt().checked_add(1).ok_or(<Error<T>>::KittyCntOverflow)?;

			// Performs this operation first because as it may fail
			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| kitty_vec.try_push(kitty_id))
				.map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			<Kitties<T>>::insert(kitty_id, kitty);
			<KittyCnt<T>>::put(new_cnt);
			Ok(kitty_id)
		}
	}
}
