#![cfg_attr(not(feature = "std"), no_std)]
use crate::types::*;

pub use pallet::*;

pub mod types;

use frame_support::pallet_prelude::*;
use frame_support::sp_std::vec::Vec;
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info] // Only for DEVELOPMENT or DEMONSTRATION
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_did::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::storage]
	#[pallet::getter(fn attribute_of)]
	pub(super) type AttributeOf<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Attribute>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_policy)]
	pub(super) type Policy<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<PolicyField>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AttributeAdded(T::AccountId, T::AccountId, Vec<u8>, Vec<u8>),
		AttributeRemoved(T::AccountId, T::AccountId, Vec<u8>),
		PolicyAdded(T::AccountId, T::AccountId, Vec<u8>),
		PolicyRemoved(T::AccountId, T::AccountId, Vec<u8>),
		CheckAccessCompleted(T::AccountId, T::AccountId, T::AccountId, Vec<u8>, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		NotHasPermission,
		InvalidAttribute,
		NotExistPolicy,
		ExistPolicy,
		InvalidPolicyField,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn add_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
			value: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::has_permission(&identity, &who)?;

			// Check if the name exists or not
			if <AttributeOf<T>>::contains_key(&identity) {
				let mut exist = false;
				<AttributeOf<T>>::mutate(&identity, |attrs| {
					let position: Option<usize> = attrs.iter().position(|a| *a.name == name);
					match position {
						Some(_) => {
							exist = true;
						},
						_ => {
							(*attrs).push(Attribute { name: name.clone(), value: value.clone() });
						},
					};
				});
				if exist {
					return Err(Error::<T>::InvalidAttribute.into());
				}
			} else {
				// Add for the first time
				let mut attrs = Vec::<Attribute>::new();
				attrs.push(Attribute { name: name.clone(), value: value.clone() });
				<AttributeOf<T>>::insert(&identity, attrs);
			}

			Self::deposit_event(Event::AttributeAdded(who, identity, name, value));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::has_permission(&identity, &who)?;

			// Check if the name exists or not
			if <AttributeOf<T>>::contains_key(&identity) {
				let mut exist = false;
				<AttributeOf<T>>::mutate(&identity, |attrs| {
					let position: Option<usize> = attrs.iter().position(|a| *a.name == name);
					match position {
						Some(i) => {
							(*attrs).remove(i);
							exist = true;
						},
						_ => {},
					};
				});
				if !exist {
					return Err(Error::<T>::InvalidAttribute.into());
				}
			} else {
				return Err(Error::<T>::InvalidAttribute.into());
			}

			Self::deposit_event(Event::AttributeRemoved(who, identity, name));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn add_policy(
			origin: OriginFor<T>,
			object_identity: T::AccountId,
			policy_name: Vec<u8>, // Todo
			policy: Vec<PolicyField>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::has_permission(&object_identity, &who)?;

			if <Policy<T>>::contains_key(&object_identity) {
				return Err(Error::<T>::ExistPolicy.into());
			}

			<Policy<T>>::insert(&object_identity, policy);

			Self::deposit_event(Event::PolicyAdded(who, object_identity, policy_name));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_policy(
			origin: OriginFor<T>,
			object_identity: T::AccountId,
			policy_name: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::has_permission(&object_identity, &who)?;

			if !<Policy<T>>::contains_key(&object_identity) {
				return Err(Error::<T>::NotExistPolicy.into());
			}

			<Policy<T>>::remove(&object_identity);

			Self::deposit_event(Event::PolicyRemoved(who, object_identity, policy_name));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn check_access(
			origin: OriginFor<T>,
			subject_identity: T::AccountId,
			object_identity: T::AccountId,
			required_operation: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if !<AttributeOf<T>>::contains_key(&subject_identity)
				|| !<AttributeOf<T>>::contains_key(&object_identity)
			{
				return Err(Error::<T>::InvalidAttribute.into());
			}

			if !<Policy<T>>::contains_key(&object_identity) {
				return Err(Error::<T>::NotExistPolicy.into());
			}

			let subject_attrs = <AttributeOf<T>>::get(&subject_identity);
			// let object_attrs = <AttributeOf<T>>::get(&object_identity);
			let policy = <Policy<T>>::get(&object_identity).unwrap();

			if Self::validate_policy_with_attrs(&policy, &subject_attrs) {
				Self::deposit_event(Event::CheckAccessCompleted(
					who,
					subject_identity,
					object_identity,
					required_operation,
					b"Allow".to_vec(),
				));
			} else {
				Self::deposit_event(Event::CheckAccessCompleted(
					who,
					subject_identity,
					object_identity,
					required_operation,
					b"Deny".to_vec(),
				));
			}
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn has_permission(identity: &T::AccountId, manipulator: &T::AccountId) -> DispatchResult {
		let owner = <pallet_did::Pallet<T>>::identity_owner(&identity);
		match owner == *manipulator {
			true => Ok(()),
			false => Err(Error::<T>::NotHasPermission.into()),
		}
	}

	fn validate_policy_field_with_attrs(field: &PolicyField, attrs: &Vec<Attribute>) -> bool {
		let attr = attrs.iter().find(|a| (**a).name == field.name);

		match attr {
			Some(a) => {
				if field.value.iter().find(|v| **v == (*a).value).is_some() {
					return true;
				}
			},
			None => (),
		}

		false
	}

	fn validate_policy_with_attrs(policy: &Vec<PolicyField>, attrs: &Vec<Attribute>) -> bool {
		for field in policy.iter() {
			if Self::validate_policy_field_with_attrs(&field, &attrs) == false {
				return false;
			}
		}
		true
	}
}
