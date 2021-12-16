#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::all)]

use codec::Codec;
use sp_std::prelude::Vec;

pub use pallet_erc20::ERC20Info;

sp_api::decl_runtime_apis! {
	pub trait ERC20Api<AccountId> where
		AccountId: Codec
	{
		fn get_erc20_info(owner:AccountId) -> Option<ERC20Info<AccountId>>;
	}
}
