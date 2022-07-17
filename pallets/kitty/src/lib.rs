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
use frame_support::traits::Currency;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	pub type DNA = Vec<u8>;
	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: DNA,
		owner: T::AccountId,
		price: BalanceOf<T>,
		gender: Gender,
	}

	#[derive(TypeInfo, Encode, Decode)]
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
		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn kitty_count)]
	pub type KittyCount<T> = StorageValue<_, u32, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_map)]
	pub(super) type KittyMap<T: Config> =
		StorageMap<_, Blake2_128Concat, DNA, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner_map)]
	pub(super) type OwnerMap<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<DNA>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyStored(DNA, BalanceOf<T>),
		KittyOwnerChanged(DNA, T::AccountId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		SomethingError,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, dna: DNA, price: BalanceOf<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			log::info!("total balance: {:?}", T::Currency::total_balance(&who));

			// Automatically create gender from dna
			let gender = match dna.len() {
				0 => Gender::FEMALE,
				_ => Gender::MALE,
			};

			let new_kitty: Kitty<T> = Kitty { dna: dna.clone(), owner: who.clone(), price, gender };

			// Store new kitty into KittyMap
			<KittyMap<T>>::insert(dna.clone(), new_kitty);

			// Update new count of kitty.
			let mut count = <KittyCount<T>>::get().unwrap_or(0);
			count += 1;
			<KittyCount<T>>::put(count);

			// Update OwnerMap at the pair of the key of this account id.
			<OwnerMap<T>>::mutate(who.clone(), |query: &mut Option<Vec<DNA>>| {
				match query {
					Some(vector) => vector.push(dna.clone()),
					None => *query = Self::generate_new_owner(dna.clone()),
				};
			});

			// Emit an event.
			Self::deposit_event(Event::KittyStored(dna, price));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_owner(
			origin: OriginFor<T>,
			dna: DNA,
			new_owner: T::AccountId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update kitty's owner in KittyMap
			<KittyMap<T>>::mutate(dna.clone(), |query: &mut Option<Kitty<T>>| match query {
				Some(kitty) => kitty.owner = new_owner.clone(),
				None => (),
			});

			// Update kitty vectors of related owners in OwnerMap
			// Remove the kitty from the vector of the current owner.
			<OwnerMap<T>>::mutate_exists(who.clone(), |query: &mut Option<Vec<DNA>>| match query {
				Some(vector) => {
					*vector = vector.drain(..).filter(|d| *d != dna).collect();
					if vector.len() == 0 {
						*query = None;
					}
				},
				None => (),
			});
			// Add the kitty to the vector of the new owner.
			<OwnerMap<T>>::mutate(new_owner.clone(), |query: &mut Option<Vec<DNA>>| match query {
				Some(vector) => vector.push(dna.clone()),
				None => *query = Self::generate_new_owner(dna.clone()),
			});

			// Emit an event.
			Self::deposit_event(Event::KittyOwnerChanged(dna, who, new_owner));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

// helper function
impl<T> Pallet<T> {
	fn generate_new_owner(dna: DNA) -> Option<Vec<DNA>> {
		let mut new_vector = Vec::<DNA>::new();
		new_vector.push(dna);
		Some(new_vector)
	}
}
