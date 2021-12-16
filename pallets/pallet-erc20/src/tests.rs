use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

const ALICE: u128 = 1;
const BOB: u128 = 2;

#[test]
fn init_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(ERC20::init(Origin::signed(ALICE), 10000));
		assert_eq!(ERC20::get_balance(ALICE), 10000);
		expect_event(ERC20Event::Initialized(ALICE));
	});
}

#[test]
fn get_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(ERC20::get_name(), String::from("NB Token").into_bytes());
		assert_eq!(ERC20::get_symbol(), String::from("NB").into_bytes());
		assert_eq!(ERC20::get_decimal(), 18);
		assert_eq!(ERC20::get_owner(), ALICE);
	});
}

#[test]
fn transfer_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(ERC20::init(Origin::signed(ALICE), 1000));
		assert_ok!(ERC20::transfer(Origin::signed(ALICE), BOB, 300));

		assert_eq!(ERC20::get_balance(BOB), 300);
		assert_eq!(ERC20::get_balance(ALICE), 700);
	});
}
