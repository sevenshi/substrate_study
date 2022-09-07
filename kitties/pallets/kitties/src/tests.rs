use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

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
		assert_noop!(
			KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::KittiesCountOverflow
		);
	});
}

#[test]
fn create_error_when_max_kitty_owned() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		//assert_noop!(KittiesModule::create(Origin::signed(1)),
		// Error::<Test>::ExceedMaxKittyOwned);
		assert_noop!(KittiesModule::create(Origin::signed(1)), Error::<Test>::ExceedMaxKittyOwned);
	});
}

#[test]
fn create_failed_when_not_enough_balance_for_staking() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 3;
		assert_noop!(
			KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::NotEnoughBalanceForStaking
		);
	});
}

#[test]
fn breed_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_ok!(KittiesModule::breed(Origin::signed(1), 0, 1));
	});
}

#[test]
fn breed_error_when_same_index() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_noop!(KittiesModule::breed(Origin::signed(1), 1, 1), Error::<Test>::SameParentIndex);
	});
}

#[test]
fn breed_error_when_not_enough_balance_for_staking() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::breed(Origin::signed(2), 0, 1));
		assert_noop!(
			KittiesModule::breed(Origin::signed(3), 0, 1),
			Error::<Test>::NotEnoughBalanceForStaking
		);
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_ok!(KittiesModule::transfer(Origin::signed(1), 0, 2));
		assert_eq!(KittiesOwner::<Test>::get(2).contains(&0u32), true);
	});
}

#[test]
fn transfer_error_when_not_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_noop!(KittiesModule::transfer(Origin::signed(2), 0, 1), Error::<Test>::NotOwner);
	});
}

#[test]
fn transfer_error_when_not_exists() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), 1, 3),
			Error::<Test>::InvalidKittyIndex
		);
	});
}

#[test]
fn transfer_error_when_not_enough_balance_for_staking() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), 0, 3),
			Error::<Test>::NotEnoughBalanceForStaking
		);
	});
}

#[test]
fn set_price_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_ok!(KittiesModule::set_price(Origin::signed(1), 0, Some(10000u128)));
	});
}

#[test]
fn set_price_error_when_not_exists() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_noop!(
			KittiesModule::set_price(Origin::signed(1), 1, Some(10000u128)),
			Error::<Test>::InvalidKittyIndex
		);
	});
}

#[test]
fn buy_kitty_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_ok!(KittiesModule::set_price(Origin::signed(1), 0, Some(10000u128)));
		assert_ok!(KittiesModule::buy_kitty(Origin::signed(2), 0));
		assert_eq!(KittiesOwner::<Test>::get(2).contains(&0u32), true);
	});
}

#[test]
fn buy_kitty_error_when_not_for_sale() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_noop!(
			KittiesModule::buy_kitty(Origin::signed(2), 0),
			Error::<Test>::KittyNotForSale
		);
	});
}

#[test]
fn buy_kitty_error_when_not_enough_balance() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// Transfer AccountID 1 to AccountID 2, KittyIndex = 0
		assert_ok!(KittiesModule::set_price(Origin::signed(1), 0, Some(10000u128)));
		assert_noop!(
			KittiesModule::buy_kitty(Origin::signed(3), 0),
			Error::<Test>::NotEnoughBalanceForBuying
		);
	});
}
