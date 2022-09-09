use crate::{mock::*, Config, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::BoundedVec;

const ALICE: u64 = 1;
const BOB: u64 = 2;

#[test]
fn create_claim_should_work() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Poe::create_claim(Origin::signed(ALICE), b"hello".to_vec()));
		// Read pallet storage and assert an expected result.
		let claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(b"hello".to_vec())
			.unwrap();
		assert_eq!(Poe::proofs(claim), Some((ALICE, frame_system::Pallet::<Test>::block_number())));
	});
}

#[test]
fn create_claim_should_not_work() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Poe::create_claim(Origin::signed(ALICE), b"hello".to_vec()));
		assert_noop!(
			Poe::create_claim(Origin::signed(ALICE), b"hello".to_vec()),
			Error::<Test>::ProofAlreadyClaimed
		);

		assert_noop!(
			Poe::create_claim(Origin::signed(ALICE), vec![0; 513]),
			Error::<Test>::ClaimToolong
		);
	});
}

#[test]
fn revoke_claim_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Poe::create_claim(Origin::signed(ALICE), b"hello".to_vec()));

		let claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(b"hello".to_vec())
			.unwrap();
		assert_eq!(
			Poe::proofs(claim.clone()),
			Some((ALICE, frame_system::Pallet::<Test>::block_number()))
		);

		// Dispatch a signed extrinsic.
		assert_ok!(Poe::revoke_claim(Origin::signed(ALICE), b"hello".to_vec()));
		assert_eq!(Poe::proofs(claim.clone()), None);
	});
}

#[test]
fn revoke_claim_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Poe::create_claim(Origin::signed(ALICE), b"hello".to_vec()));

		let claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(b"hello".to_vec())
			.unwrap();
		assert_eq!(
			Poe::proofs(claim.clone()),
			Some((ALICE, frame_system::Pallet::<Test>::block_number()))
		);

		assert_noop!(
			Poe::revoke_claim(Origin::signed(ALICE), vec![0; 513]),
			Error::<Test>::ClaimToolong
		);
		assert_noop!(
			Poe::revoke_claim(Origin::signed(ALICE), vec![0; 512]),
			Error::<Test>::ClaimNotExist
		);
		assert_noop!(
			Poe::revoke_claim(Origin::signed(BOB), b"hello".to_vec()),
			Error::<Test>::NotClaimOwner
		);
	});
}

#[test]
fn transfer_claim_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Poe::create_claim(Origin::signed(ALICE), b"hello".to_vec()));

		let claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(b"hello".to_vec())
			.unwrap();
		assert_eq!(
			Poe::proofs(claim.clone()),
			Some((ALICE, frame_system::Pallet::<Test>::block_number()))
		);

		// Dispatch a signed extrinsic.
		assert_ok!(Poe::transfer_claim(Origin::signed(ALICE), BOB, b"hello".to_vec()));
		assert_eq!(
			Poe::proofs(claim.clone()),
			Some((BOB, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn transfer_claim_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Poe::create_claim(Origin::signed(ALICE), b"hello".to_vec()));

		assert_noop!(
			Poe::transfer_claim(Origin::signed(ALICE), BOB, vec![0; 513]),
			Error::<Test>::ClaimToolong
		);
		assert_noop!(
			Poe::transfer_claim(Origin::signed(ALICE), BOB, vec![0; 512]),
			Error::<Test>::ClaimNotExist
		);
		assert_noop!(
			Poe::transfer_claim(Origin::signed(BOB), ALICE, b"hello".to_vec()),
			Error::<Test>::NotClaimOwner
		);
	});
}
