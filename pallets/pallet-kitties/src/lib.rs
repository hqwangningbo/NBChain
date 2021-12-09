#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

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
		//使用随机，获得一个随机 Self::Hash 类型
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

	// Struct Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}

	//kitty gender
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	// kitties ID => kitties 详细信息
	pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	// 账户 => kitties ID列表
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
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
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
		// Kitty计数器时处理算术溢出 u64 2^64-1
		KittyCntOverflow,
		// 一个人有的猫不能超过最大值9999
		ExceedMaxKittyOwned,
		//不能买自己的猫
		BuyerIsKittyOwner,
		//不能转给自己
		TransferToSelf,
		//猫咪不存在
		KittyNotExist,
		//你不是猫咪的主人
		NotKittyOwner,
		//猫咪不在卖
		KittyNotForSale,
		//出价太低
		KittyBidPriceTooLow,
		//没有足够的钱
		NotEnoughBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::mint(&sender, None, None)?;
			log::info!("A kitty is born with ID: {:?}.", kitty_id);
			Self::deposit_event(Event::Created(sender, kitty_id));
			Ok(())
		}
		#[pallet::weight(100)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			//检查猫咪是不是自己的
			ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, <Error<T>>::NotKittyOwner);
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			//设置新的价格
			kitty.price = new_price.clone();
			<Kitties<T>>::insert(&kitty_id, kitty);
			//发出价格设置事件
			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));
			Ok(())
		}
		#[pallet::weight(100)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: T::Hash,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			//检查猫咪是不是自己的
			ensure!(Self::is_kitty_owner(&kitty_id, &from)?, <Error<T>>::NotKittyOwner);

			//猫咪不能转给自己
			ensure!(from != to, <Error<T>>::TransferToSelf);

			// 确保一个人拥有的猫数量少于9999
			let to_owned = <KittiesOwned<T>>::get(&to);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittyOwned::get(),
				<Error<T>>::ExceedMaxKittyOwned
			);
			Self::transfer_kitty_to(&kitty_id, &to)?;
			Self::deposit_event(Event::Transferred(from, to, kitty_id));

			Ok(())
		}
		#[transactional]
		#[pallet::weight(100)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			// 检查买家和卖家不是同一人
			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);

			//检查猫咪是否在出售和判断价格是否合理
			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				Err(<Error<T>>::KittyNotForSale)?;
			}

			//检查买家是否有足够的余额
			ensure!(T::Currency::free_balance(&buyer) >= bid_price, <Error<T>>::NotEnoughBalance);
			//检查买家是否还能接受新的猫咪
			let to_owned = <KittiesOwned<T>>::get(&buyer);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittyOwned::get(),
				<Error<T>>::ExceedMaxKittyOwned
			);
			let seller = kitty.owner.clone();
			//打钱
			T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;
			//转移小猫
			Self::transfer_kitty_to(&kitty_id, &buyer)?;
			// 发出事件
			Self::deposit_event(Event::Bought(buyer, seller, kitty_id, bid_price));
			Ok(())
		}
		#[pallet::weight(100)]
		pub fn breed_kitty(
			origin: OriginFor<T>,
			parent1: T::Hash,
			parent2: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			//两只猫咪必须属于同一个人
			ensure!(Self::is_kitty_owner(&parent1, &sender)?, <Error<T>>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(&parent2, &sender)?, <Error<T>>::NotKittyOwner);

			//融合DNA生产新的DNA
			let new_dna = Self::breed_dna(&parent1, &parent2)?;
			//根据DNA生产新的小猫
			Self::mint(&sender, Some(new_dna), None)?;
			Ok(())
		}
	}

	//创世配置
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>,
	}

	//设置kitties默认值
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// When building a kitty from genesis config, we require the dna and gender to be supplied.
			for (acct, dna, gender) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), Some(gender.clone()));
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn gen_gender() -> Gender {
			//获取随机hash
			let random = T::KittyRandomness::random(&b"nbgender"[..]).0;
			//将hash as_ref()强制转换成u8数组
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}
		pub fn gen_dna() -> [u8; 16] {
			let payload = (
				//获取随机hash
				T::KittyRandomness::random(&b"nbdna"[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			//using_encoded将元组转成切片，然后调用128位hash
			payload.using_encoded(blake2_128)
		}
		pub fn breed_dna(parent1: &T::Hash, parent2: &T::Hash) -> Result<[u8; 16], Error<T>> {
			let dna1 = Self::kitties(parent1).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let dna2 = Self::kitties(parent2).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}
			Ok(new_dna)
		}
		//生成小猫，小知识，如果存在dna那么就会用这个dna，不存在就会调用gen_dna()生成dna
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
			//对猫咪整体进行hash
			let kitty_id = T::Hashing::hash_of(&kitty);

			//猫咪总数加1
			let new_cnt = Self::kitty_cnt().checked_add(1).ok_or(<Error<T>>::KittyCntOverflow)?;

			//把猫咪ID存到调用者猫咪列表中
			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| kitty_vec.try_push(kitty_id))
				.map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			<Kitties<T>>::insert(kitty_id, kitty);
			<KittyCnt<T>>::put(new_cnt);
			Ok(kitty_id)
		}
		//判断某个用户是否拥有某只猫咪
		pub fn is_kitty_owner(kitty_id: &T::Hash, acct: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(<Error<T>>::KittyNotExist),
			}
		}
		pub fn balance_of(account: &T::AccountId) -> BalanceOf<T> {
			T::Currency::free_balance(account)
		}

		#[transactional]
		pub fn transfer_kitty_to(kitty_id: &T::Hash, to: &T::AccountId) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			let prev_owner = kitty.owner.clone();
			// 找到原本拥有者kitty_id的索引，然后删除它
			<KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == *kitty_id) {
					owned.swap_remove(ind);
					return Ok(());
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::KittyNotExist)?;

			// 设置新主人
			kitty.owner = to.clone();
			// 重置价格
			kitty.price = None;
			// 插入喵咪总列表
			<Kitties<T>>::insert(kitty_id, kitty);
			//插入主人的猫咪列表中
			<KittiesOwned<T>>::try_mutate(to, |vec| vec.try_push(*kitty_id))
				.map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;
			Ok(())
		}
	}
}
