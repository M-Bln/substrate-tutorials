use super::mock::*;
use crate::{pallet, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{traits::Dispatchable, DispatchError::Module, ModuleError, RuntimeDebug};

#[test]
fn set_value_ok() {
	new_test_ext().execute_with(|| {
		assert_ok!(Flipper::set_value(Origin::signed(ALICE), false));
		assert_eq!(Flipper::value(), Some(false));
	});
}

#[test]
fn set_value_err_already_set() {
	new_test_ext().execute_with(|| {
		assert_ok!(Flipper::set_value(Origin::signed(ALICE), false));
		assert_eq!(
			Flipper::set_value(Origin::signed(ALICE), false),
			Err(Module(ModuleError {
				index: 1,
				error: [1, 0, 0, 0],
				message: Some("AlreadySet")
			}))
		);
	});
	//new_test_ext().execute_with(|| todo!("Verify if the function returns the expected error."));
}

#[test]
fn flip_value_ok() {
	new_test_ext().execute_with(|| {
		assert_ok!(Flipper::set_value(Origin::signed(ALICE), false));
		assert_eq!(Flipper::value(), Some(false));
		assert_ok!(Flipper::flip_value(Origin::signed(ALICE)));
		assert_eq!(Flipper::value(), Some(true));
	});
	//.execute_with(|| todo!("Ensure the good behaviour of the flip_value() function."));
}

#[test]
fn flip_value_ko() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			Flipper::flip_value(Origin::signed(ALICE)),
			Err(Module(ModuleError {
				index: 1,
				error: [0, 0, 0, 0,],
				message: Some("NoneValue",),
			},),)
		);
	});
	//.execute_with(|| todo!("write a scenario that triggers an error in flip_value()"));
}
