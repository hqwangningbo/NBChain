use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;

#[test]
fn create_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Kitties::create(Origin::signed(ALICE)));

		expect_event(KittyEvent::KittyCreated(ALICE, 0, Kitties::get_kitty(0).unwrap()));

		assert_eq!(Kitties::kitty_owner(0), Some(ALICE));
		assert_eq!(Kitties::next_kitty_id(), 1);
		assert_eq!(Balances::free_balance(ALICE), 99_000_000_000_000_000_000);
	});
}

#[test]
fn create_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Kitties::create(Origin::signed(CHARLIE)),
			Error::<Test>::NotEnoughBalanceForCreating
		);
	});
}

#[test]
fn breed_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Kitties::create(Origin::signed(ALICE)));
		assert_ok!(Kitties::create(Origin::signed(ALICE)));

		assert_ok!(Kitties::breed(Origin::signed(ALICE), 0, 1));
		expect_event(KittyEvent::KittyBreed(ALICE, 0, 1));

		assert_eq!(Kitties::kitty_owner(2), Some(ALICE));
		assert_eq!(Kitties::next_kitty_id(), 3);
	});
}

#[test]
fn breed_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Kitties::create(Origin::signed(ALICE)));
		assert_ok!(Kitties::create(Origin::signed(ALICE)));

		assert_noop!(Kitties::breed(Origin::signed(ALICE), 0, 0), Error::<Test>::SameKittyId);
		assert_noop!(
			Kitties::breed(Origin::signed(CHARLIE), 0, 1),
			Error::<Test>::NotEnoughBalanceForBreeding
		);
		assert_noop!(Kitties::breed(Origin::signed(ALICE), 1, 3), Error::<Test>::InvalidKittyId);
	});
}

#[test]
fn transfer_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Kitties::create(Origin::signed(ALICE)));

		assert_ok!(Kitties::transfer(Origin::signed(ALICE), 0, BOB));

		expect_event(KittyEvent::KittyTransferred(ALICE, BOB, 0));

		assert_eq!(Kitties::kitty_owner(0), Some(BOB));
		assert_eq!(Balances::free_balance(ALICE), 100_000_000_000_000_000_000);
		assert_eq!(Balances::free_balance(BOB), 99_000_000_000_000_000_000);
	});
}

#[test]
fn transfer_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Kitties::create(Origin::signed(ALICE)));
		assert_ok!(Kitties::create(Origin::signed(ALICE)));

		assert_noop!(Kitties::transfer(Origin::signed(BOB), 0, ALICE), Error::<Test>::NotOwner);
		assert_noop!(
			Kitties::transfer(Origin::signed(ALICE), 0, CHARLIE),
			Error::<Test>::NotEnoughBalanceForReceiving
		);
	});
}

#[test]
fn sell_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Kitties::create(Origin::signed(ALICE)));

		assert_ok!(Kitties::sell(Origin::signed(ALICE), 0, 10_000_000_000_000_000_000u128));
		expect_event(KittyEvent::KittyListed(ALICE, 0, 10_000_000_000_000_000_000u128));

		assert_eq!(
			Kitties::kitties_list_for_sales(0),
			Some((ALICE, 10_000_000_000_000_000_000u128))
		);
	});
}

#[test]
fn sell_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Kitties::sell(Origin::signed(ALICE), 0, 10_000_000_000_000_000_000u128),
			Error::<Test>::InvalidKittyId
		);
		assert_ok!(Kitties::create(Origin::signed(ALICE)));
		assert_noop!(
			Kitties::sell(Origin::signed(CHARLIE), 0, 10_000_000_000_000_000_000u128),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn buy_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Kitties::create(Origin::signed(ALICE)));

		assert_ok!(Kitties::sell(Origin::signed(ALICE), 0, 10_000_000_000_000_000_000u128));
		assert_ok!(Kitties::buy(Origin::signed(BOB), 0));
		expect_event(KittyEvent::KittyBuyed(ALICE, BOB, 0, 10_000_000_000_000_000_000u128));

		assert_eq!(Balances::free_balance(ALICE), 110_000_000_000_000_000_000);
		assert_eq!(Balances::free_balance(BOB), 89_000_000_000_000_000_000);
		assert_eq!(Kitties::kitty_owner(0), Some(BOB));
	});
}

#[test]
fn buy_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(Kitties::buy(Origin::signed(BOB), 0), Error::<Test>::InvalidKittyId);
		assert_ok!(Kitties::create(Origin::signed(ALICE)));
		assert_noop!(Kitties::buy(Origin::signed(BOB), 0), Error::<Test>::NotForSale);
		assert_ok!(Kitties::sell(Origin::signed(ALICE), 0, 10_000_000_000_000_000_000u128));
		assert_noop!(
			Kitties::buy(Origin::signed(CHARLIE), 0),
			Error::<Test>::NotEnoughBalanceForBuying
		);
	});
}
