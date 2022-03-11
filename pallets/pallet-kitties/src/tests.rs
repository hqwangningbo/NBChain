use crate::mock::*;
use frame_support::metadata::StorageEntryModifier::Default;
use frame_support::traits::tokens::Balance;
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};

const ALICE: u128 = 1;
const BOB: u128 = 2;

#[test]
fn create_kitties_should_work() {
	new_test_ext().execute_with(|| {
		// (1) create kitty
		assert_ok!(Kitties::create_kitty(Origin::signed(ALICE)));
		let hash = Kitties::kitties_owned(ALICE)[0];
		expect_event(KittiesEvent::Created(ALICE, hash));
	});
}

#[test]
fn get_balance_test() {
	new_test_ext().execute_with(|| {
		let balance = Kitties::balance_of(&ALICE);
		assert_eq!(balance, 100_000_000_000_000_000_000);
	})
}
