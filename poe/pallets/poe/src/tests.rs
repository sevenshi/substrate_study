use crate::{mock::*, Error, Config, Proofs};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {

		let claim = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		let bounded_claim = BoundedVec::<u8,<Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_eq!(Proofs::<Test>::get(&bounded_claim),
		Some((1,frame_system::Pallet::<Test>::block_number())));
	});
}

#[test]
fn create_claim_failed_when_claim_too_long() {
	new_test_ext().execute_with(|| {

		let claim = vec![0,1,2,3];
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimTooLong	
		);
	});
}

#[test]
fn create_claim_failed_when_claim_alread_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist	
		);
	});
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		let bounded_claim = BoundedVec::<u8,<Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_ok!(PoeModule::revoke_claim(Origin::signed(1),bounded_claim.to_vec()));
	});
}
#[test]
fn revoke_claim_failed_when_claim_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(PoeModule::revoke_claim(Origin::signed(1),vec![1]),Error::<Test>::ClaimNotExist);
	});
}

#[test]
fn revoke_claim_failed_when_not_claim_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		let bounded_claim = BoundedVec::<u8,<Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		assert_noop!(PoeModule::revoke_claim(Origin::signed(2),bounded_claim.to_vec()),Error::<Test>::NotClaimOwner);
	});
}


#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		let bounded_claim = BoundedVec::<u8,<Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		assert_eq!(Proofs::<Test>::get(&bounded_claim),
		Some((1,frame_system::Pallet::<Test>::block_number())));
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1),bounded_claim.to_vec(),2));
		assert_eq!(Proofs::<Test>::get(&bounded_claim),
		Some((2,frame_system::Pallet::<Test>::block_number())));
		
	});
}

#[test]
fn  transfer_claim_failed_when_claim_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(PoeModule::transfer_claim(Origin::signed(1),vec![1],2),Error::<Test>::ClaimNotExist);
	});
}

#[test]
fn  transfer_claim_failed_when_not_claim_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		let bounded_claim = BoundedVec::<u8,<Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1),bounded_claim.to_vec(),2));
		assert_noop!(PoeModule::transfer_claim(Origin::signed(1),bounded_claim.to_vec(),3),Error::<Test>::NotClaimOwner);
	});
}
