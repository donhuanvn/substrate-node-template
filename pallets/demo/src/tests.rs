use crate::{mock::*, Error, Gender};
use frame_support::{assert_noop, assert_ok};

#[test]
fn correct_error_for_creating_student_of_blank_name() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let blank_name: Vec<u8> = b"".to_vec();
		let age = 21;
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(DemoModule::create_student(origin, blank_name, age), Error::<Test>::NameMustNotBeBlank);
	});
}

#[test]
fn correct_error_for_creating_student_of_too_young() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = b"Alice".to_vec();
		let undering_age = 19;
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(DemoModule::create_student(origin, name, undering_age), Error::<Test>::TooYoung);
	});
}

#[test]
fn it_stores_correctly_data_for_creating_student() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(1);
		let name: Vec<u8> = b"Alice".to_vec();
		let age = 21;
		
		assert_ok!(DemoModule::create_student(origin.clone(), name.clone(), age.clone()));

		let student_id = DemoModule::student_id();
		assert_eq!(student_id, Some(1));

		let student = DemoModule::student(0);
		assert!(student.is_some());
		
		let student = student.unwrap();
		assert_eq!(student.name, name);
		assert_eq!(student.age, age);
		assert_eq!(student.gender, Gender::MALE);
		assert_eq!(student.account, 1);
	});
}

