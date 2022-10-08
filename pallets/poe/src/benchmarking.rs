//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Poe;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_support::assert_ok;

benchmarks! {
	create_claim {
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller),b"hello".to_vec())

	revoke_claim {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(
			Poe::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(),b"hello".to_vec())
		);
	}: _(RawOrigin::Signed(caller),b"hello".to_vec())

	transfer_claim {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(
			Poe::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(),b"hello".to_vec())
		);
	}: _(RawOrigin::Signed(caller),account("receiver", 0, 0u32),b"hello".to_vec())

	impl_benchmark_test_suite!(Poe, crate::mock::new_test_ext(), crate::mock::Test);
}
