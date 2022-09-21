#![cfg_attr(not(feature = "std"), no_std)]
pub mod types;
use crate::types::*;

pub use pallet::*;

use frame_support::{pallet_prelude::*, sp_std::vec::Vec, traits::Time};
use frame_system::pallet_prelude::*;

const VEC_MAX_LENGTH: usize = 64;

type Moment<T> = <<T as Config>::Time as Time>::Moment;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info] // Only for DEVELOPMENT or DEMONSTRATION
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_did::Config + pallet_contracts::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Time: Time;
	}

	#[pallet::storage]
	#[pallet::getter(fn attr_of)]
	pub(super) type AttrOf<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		Vec<u8>,
		Attr<Moment<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn endorsement_of)]
	pub(super) type EndorsementOf<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, T::AccountId>,
		),
		Endorsement<T::BlockNumber, Moment<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn policy_of)]
	pub(super) type PolicyOf<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		Policy<T::AccountId, Moment<T>>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AttributesSet(T::AccountId, T::AccountId, Vec<AttrInput>),
		AttributesCleared(T::AccountId, T::AccountId, Vec<Vec<u8>>),
		AttributesEndorsed(T::AccountId, T::AccountId, T::AccountId, Vec<Vec<u8>>, T::BlockNumber),
		AttributesUnendorsed(T::AccountId, T::AccountId, T::AccountId, Vec<Vec<u8>>),
		PolicyAttached(T::AccountId, T::AccountId, T::AccountId, T::AccountId, Vec<u8>),
		PolicyDetached(T::AccountId, T::AccountId, T::AccountId, T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		NotOwner,
		NotHasPermission,
		InputOverflowed,
		InvalidAttributes,
		InvalidPolicy,
		AttributeClearingFailed,
		AttributeSettingFailed,
		AttributeEndorsementFailed,
		PolicyAttachmentFailed,
		PolicyDetachmentFailed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn set_attributes(
			origin: OriginFor<T>,
			identity: T::AccountId,
			list_of_attrs: Vec<AttrInput>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(list_of_attrs.len() <= VEC_MAX_LENGTH, Error::<T>::InputOverflowed);
			for attr in list_of_attrs.iter() {
				ensure!(
					attr.name.len() <= VEC_MAX_LENGTH && attr.value.len() <= VEC_MAX_LENGTH,
					Error::<T>::InputOverflowed
				);
			}
		
			Self::is_owner(&identity, &who)?;

			if Self::check_attr_keys_duplication(&list_of_attrs) == true {
				return Err(Error::<T>::InvalidAttributes.into());
			}

			// Write to storage item-by-item and overwrite if exists.
			for attr in list_of_attrs.iter() {
				let new_attr: Attr<Moment<T>> = Attr {
					name: attr.name.clone(),
					value: attr.value.clone(),
					updated_time: <T as Config>::Time::now(),
				};
				// Check if it should insert or update.
				if <AttrOf<T>>::contains_key(&identity, &attr.name) {
					let mut changed: bool = false; // it will remove endorsement if the attribute value changed.
					<AttrOf<T>>::mutate(&identity, &attr.name, |a| {
						changed = a.as_ref().unwrap().value != attr.value;
						*a = Some(new_attr);
					});
					if changed {
						Self::remove_endorsements_per_attribute(&identity, &attr.name);
					}
				} else {
					<AttrOf<T>>::insert(&identity, &attr.name, new_attr);
				}
			}

			Self::deposit_event(Event::AttributesSet(who, identity, list_of_attrs));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn clear_attributes(
			origin: OriginFor<T>,
			identity: T::AccountId,
			list_of_attr_keys: Vec<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(list_of_attr_keys.len() <= VEC_MAX_LENGTH, Error::<T>::InputOverflowed);
			for attr_key in list_of_attr_keys.iter() {
				ensure!(attr_key.len() <= VEC_MAX_LENGTH, Error::<T>::InputOverflowed);
			}

			Self::is_owner(&identity, &who)?;

			// Don't accept any non-existing key.
			for key in list_of_attr_keys.iter() {
				if !<AttrOf<T>>::contains_key(&identity, &key) {
					return Err(Error::<T>::InvalidAttributes.into());
				}
			}

			// Write to storage key-by-key
			for key in list_of_attr_keys.iter() {
				<AttrOf<T>>::remove(&identity, &key);
			}

			Self::deposit_event(Event::AttributesCleared(who, identity, list_of_attr_keys));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn endorse_attributes(
			origin: OriginFor<T>,
			identity: T::AccountId,
			target_identity: T::AccountId,
			list_of_attr_keys: Vec<Vec<u8>>,
			valid_for: Option<T::BlockNumber>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			ensure!(list_of_attr_keys.len() <= VEC_MAX_LENGTH, Error::<T>::InputOverflowed);
			for attr_key in list_of_attr_keys.iter() {
				ensure!(attr_key.len() <= VEC_MAX_LENGTH, Error::<T>::InputOverflowed);
			}

			Self::is_owner(&identity, &who)?;

			// Don't accept any non-existing key.
			let all_existing = Self::check_attributes_existing(&target_identity, &list_of_attr_keys);
			if !all_existing {
				return Err(Error::<T>::InvalidAttributes.into());
			}

			let now_block_number = <frame_system::Pallet<T>>::block_number();
			let now_timestamp = <T as Config>::Time::now();
			let validity = match valid_for {
					Some(v) => v + now_block_number,
					None => u32::MAX.into()
			};

			for key in list_of_attr_keys.iter() {
				let new_endorsement = Endorsement {
					validity: validity.clone(),
					endorsed_time: now_timestamp.clone()
				};

				let endorsement_key = (&target_identity, &key, &identity);
				if <EndorsementOf<T>>::contains_key(&endorsement_key) {
					<EndorsementOf<T>>::mutate(endorsement_key, |e| {
						*e = Some(new_endorsement);
					})
				} else {
					<EndorsementOf<T>>::insert(endorsement_key, new_endorsement);
				}
			}

			Self::deposit_event(Event::AttributesEndorsed(
				who,
				identity,
				target_identity,
				list_of_attr_keys,
				validity
			));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn unendorse_attributes(
			origin: OriginFor<T>,
			identity: T::AccountId,
			target_identity: T::AccountId,
			list_of_attr_keys: Vec<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::is_owner(&identity, &who)?;

			// Don't accept any non-existing key.
			let all_existing = Self::check_attributes_existing(&identity, &list_of_attr_keys);
			if !all_existing {
				return Err(Error::<T>::InvalidAttributes.into());
			}

			for key in list_of_attr_keys.iter() {
				Self::remove_endorsements_per_attribute(&identity, key);
			}

			Self::deposit_event(Event::AttributesUnendorsed(
				who,
				identity,
				target_identity,
				list_of_attr_keys,
			));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn attach_policy(
			origin: OriginFor<T>,
			identity: T::AccountId,
			object: T::AccountId,
			policy: T::AccountId,
			name: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::deposit_event(Event::PolicyAttached(who, identity, object, policy, name));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn detach_policy(
			origin: OriginFor<T>,
			identity: T::AccountId,
			object: T::AccountId,
			policy: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::deposit_event(Event::PolicyDetached(who, identity, object, policy, Vec::new()));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Validates if the AccountId 'actual_owner' owns the identity.
	fn is_owner(identity: &T::AccountId, actual_owner: &T::AccountId) -> DispatchResult {
		let owner = Self::identity_owner(identity);
		match owner == *actual_owner {
			true => Ok(()),
			false => Err(Error::<T>::NotOwner.into()),
		}
	}

	/// Validates if the AccountId 'actual_owner' owns the identity.
	fn identity_owner(identity: &T::AccountId) -> T::AccountId {
		match pallet_did::Pallet::<T>::owner_of(identity) {
			Some(id) => id,
			None => identity.clone(),
		}
	}

	fn is_delegate_for_policy_attachment(
		object: &T::AccountId,
		attacher: &T::AccountId,
	) -> DispatchResult {
		Ok(())
	}

	fn is_address_of_smart_contract(address: &T::AccountId) -> DispatchResult {
		Ok(())
	}

	fn check_attr_keys_duplication(list_of_attrs: &Vec<AttrInput>) -> bool {
		for attr in list_of_attrs.iter() {
			let dup = list_of_attrs.iter().filter(|a| *a.name == attr.name).count();
			if dup > 1 {
				return true;
			}
		}
		false
	}

	fn check_attributes_existing(
		identity: &T::AccountId, 
		list_of_attr_keys: &Vec<Vec<u8>>
	) -> bool {
		for key in list_of_attr_keys.iter() {
			if !<AttrOf<T>>::contains_key(&identity, &key) {
				return false
			}
		}
		true
	}

	fn remove_endorsements_per_attribute(
		identity: &T::AccountId,
		attr_key: &Vec<u8>,
	) {
		// Assume that the attribute exists, so no checking code is present.
		loop {
			// Delete one item at once
			let result = <EndorsementOf<T>>::clear_prefix((identity, attr_key), 1, None);
			// Redo until no one exist
			if result.maybe_cursor.is_none() {
				break;
			}
		}
	}
}
