#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::inherent::Vec;
use frame_support::pallet_prelude::{OptionQuery, *};
use frame_system::pallet_prelude::*;
use frame_support::sp_std::fmt;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	pub type Id = u32;
	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Students <T: Config> {
		pub name: Vec<u8>,
		pub age: u8,
		pub gender: Gender,
		pub account: T::AccountId,
	}

	impl<T: Config> fmt::Debug for Students<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Students")
			.field("name", &self.name)
			.field("age", &self.age)
			.field("gender", &self.gender)
			.field("account", &self.account)
			.finish()
		}
	}

	#[derive(TypeInfo, Encode, Decode, Clone, PartialEq, Debug)]
	pub enum Gender {
		MALE,
		FEMALE,
	}

	impl Default for Gender {
		fn default() -> Self {
			Gender::MALE
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn student_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type StudentId<T> = StorageValue<_, Id>;

	#[pallet::storage]
	#[pallet::getter(fn student)]
	pub(super) type Student<T: Config> = StorageMap<_, Blake2_128Concat, Id, Students<T>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		StudentStored(Vec<u8>, u8),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		NameMustNotBeBlank,
		TooYoung,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_student(origin: OriginFor<T>, name: Vec<u8>, age: u8) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			ensure!(name.len() > 0, Error::<T>::NameMustNotBeBlank);
			ensure!(age > 20, Error::<T>::TooYoung);

			let gender = Self::generate_gender(name.clone())?;

			let student = Students { 
				name: name.clone(), 
				age: age, 
				gender: gender.clone(), 
				account: who, 
			};

			// let current_id = Self::studend_id();
			// let current_id = StudentId::<T>::get();
			let current_id = <StudentId<T>>::get();
			let mut current_id = current_id.unwrap_or(0);

			// log::info!("Current id: {}", &current_id);
			// log::info!("Gender: {:?}", &gender);
			// log::info!("Student: {:?}", &student);

			// Student::<T>::insert(current_id, student);
			<Student<T>>::insert(current_id, student);
			current_id += 1;
			<StudentId<T>>::put(current_id);

			// Emit an event.
			Self::deposit_event(Event::StudentStored(name, age));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

// helper function
impl<T> Pallet<T> {
	fn generate_gender(name: Vec<u8>) -> Result<Gender, Error<T>> {
		let res = if name.len() % 2 == 0 { Gender::FEMALE } else { Gender::MALE };
		Ok(res)
	}
}
