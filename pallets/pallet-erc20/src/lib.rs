#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::inherent::Vec;
pub use pallet::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Clone, Eq, PartialEq, Encode, Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ERC20Info<AccountId> {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimal: u8,
	pub owner: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
	use crate::ERC20Info;
	use frame_support::{dispatch::DispatchResultWithPostInfo, inherent::Vec, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		//初始化
		Initialized(T::AccountId),
		//转账
		Transfer(T::AccountId, T::AccountId, u32),
		//授权
		Approval(T::AccountId, T::AccountId, u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		//已经初始化
		AlreadyInitialized,
		//未初始化
		UnInitialized,
		//资金不足
		InsufficientFunds,
		//未授权
		Unauthorized,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	//余额map  账户=>余额
	#[pallet::storage]
	#[pallet::getter(fn get_balance)]
	pub(super) type Balances<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	//已授权map 账户=>(已授权账户,授权金额)
	#[pallet::storage]
	#[pallet::getter(fn get_allowed_info)]
	pub(super) type Allowed<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (T::AccountId, u32), ValueQuery>;

	//name value
	#[pallet::storage]
	#[pallet::getter(fn get_name)]
	pub(super) type Name<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	//symbol value
	#[pallet::storage]
	#[pallet::getter(fn get_symbol)]
	pub(super) type Symbol<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	//TotalSupply value
	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub(super) type TotalSupply<T: Config> = StorageValue<_, u32, ValueQuery>;

	//decimal
	#[pallet::storage]
	#[pallet::getter(fn get_decimal)]
	pub(super) type Decimal<T: Config> = StorageValue<_, u8, ValueQuery>;

	//owner
	#[pallet::storage]
	#[pallet::getter(fn get_owner)]
	pub(super) type Owner<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub name: Vec<u8>,
		pub symbol: Vec<u8>,
		pub decimal: u8,
		pub owner: T::AccountId,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				name: Default::default(),
				symbol: Default::default(),
				decimal: Default::default(),
				owner: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<Name<T>>::put(&self.name);
			<Symbol<T>>::put(&self.symbol);
			<Decimal<T>>::put(&self.decimal);
			<Owner<T>>::put(&self.owner);
		}
	}

	//是否初始化
	#[pallet::storage]
	#[pallet::getter(fn is_init)]
	pub type Init<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//初始化，发行amount数额的token，并将存入初始化人的余额
		#[pallet::weight(1_000)]
		pub fn init(origin: OriginFor<T>, amount: u32) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(!Self::is_init(), <Error<T>>::AlreadyInitialized);
			ensure!(Self::get_owner() == sender, <Error<T>>::Unauthorized);
			<TotalSupply<T>>::put(amount);
			<Balances<T>>::insert(&sender, amount);
			<Init<T>>::put(true);
			Self::deposit_event(Event::Initialized(sender));
			Ok(().into())
		}
		//转移token
		#[pallet::weight(1_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			amount: u32,
		) -> DispatchResultWithPostInfo {
			ensure!(Self::is_init(), <Error<T>>::UnInitialized);
			let sender = ensure_signed(origin)?;
			let sender_balance = Self::get_balance(&sender);
			ensure!(
				Self::is_sufficient_funds(sender_balance, amount),
				<Error<T>>::InsufficientFunds
			);
			Self::base_transfer(&sender, &recipient, amount);
			Self::deposit_event(Event::Transfer(sender, recipient, amount));
			Ok(().into())
		}
		//被授权人转移授权人的token
		#[pallet::weight(1_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			sender: T::AccountId,
			recipient: T::AccountId,
			amount: u32,
		) -> DispatchResultWithPostInfo {
			ensure!(Self::is_init(), <Error<T>>::UnInitialized);
			let caller = ensure_signed(origin)?;
			let (spender, spender_allowed_balance) = Self::get_allowed_info(&sender);
			ensure!(caller == spender, <Error<T>>::Unauthorized);
			ensure!(
				Self::is_sufficient_funds(spender_allowed_balance, amount),
				<Error<T>>::InsufficientFunds
			);
			<Allowed<T>>::insert(&sender, (spender.clone(), spender_allowed_balance - amount));
			Self::base_transfer(&sender, &recipient, amount);
			Self::deposit_event(Event::Transfer(sender, recipient, amount));
			Ok(().into())
		}
		//授权给别人使用自己的token
		#[pallet::weight(1_000)]
		pub fn approve(
			origin: OriginFor<T>,
			spender: T::AccountId,
			amount: u32,
		) -> DispatchResultWithPostInfo {
			ensure!(Self::is_init(), <Error<T>>::UnInitialized);
			let approver = ensure_signed(origin)?;
			let approver_balance = Self::get_balance(&approver);
			ensure!(
				Self::is_sufficient_funds(approver_balance, amount),
				<Error<T>>::InsufficientFunds
			);
			<Allowed<T>>::insert(&approver, (spender.clone(), amount));
			Self::deposit_event(Event::Approval(approver, spender, amount));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn is_sufficient_funds(sender_balance: u32, amount: u32) -> bool {
			sender_balance >= amount
		}
		fn base_transfer(
			sender: &<T as frame_system::Config>::AccountId,
			recipient: &<T as frame_system::Config>::AccountId,
			amount: u32,
		) {
			let sender_balance = Self::get_balance(sender);
			let recipient_balance = Self::get_balance(recipient);
			<Balances<T>>::insert(sender, sender_balance - amount);
			<Balances<T>>::insert(recipient, recipient_balance + amount);
		}
		pub fn get_erc20_info() -> Option<ERC20Info<T::AccountId>> {
			let name = <Name<T>>::get();
			let symbol = <Symbol<T>>::get();
			let decimal = <Decimal<T>>::get();
			let owner = <Owner<T>>::get();
			Some(ERC20Info {
				name: name.clone(),
				symbol: symbol.clone(),
				decimal: decimal.clone(),
				owner: owner.clone(),
			})
		}
	}
}
