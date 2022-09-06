use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use super::*;

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(KittiesModule::create(Origin::signed(1)));
	});
}

#[test]
fn create_failed_when_kittiescount_overflow() {
	new_test_ext().execute_with(|| {
		KittiesCount::<Test>::put(u32::max_value());
		let account_id: u64 = 1;
		assert_noop!(KittiesModule::create(Origin::signed(account_id)), Error::<Test>::KittiesCountOverflow);
	});
}

#[test]
fn create_error_when_max_kitty_owned() {
	new_test_ext().execute_with(|| {

		
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Ensure the expected error is thrown when no value is present.
		//assert_noop!(KittiesModule::create(Origin::signed(1)),Error::<Test>::ExceedMaxKittyOwned);
		assert_noop!(KittiesModule::create(Origin::signed(1)),Error::<Test>::ExceedMaxKittyOwned);
	});
}

#[test]
fn create_failed_when_not_enough_balance_for_staking() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 3;
		assert_noop!(KittiesModule::create(Origin::signed(account_id)), Error::<Test>::NotEnoughBalanceForStaking);

	});
}
