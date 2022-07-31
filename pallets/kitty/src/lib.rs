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
use frame_support::sp_std::fmt;
use frame_support::traits::Currency;
use frame_support::traits::Randomness;
use frame_support::traits::Time;
use frame_system::pallet_prelude::*;
use frame_support::sp_runtime::SaturatedConversion;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type Moment<T> = <<T as Config>::Time as frame_support::traits::Time>::Moment;

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
		created_time: Moment<T>,
	}

	impl<T: Config> fmt::Debug for Kitty<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Kitty")
				.field("dna", &self.dna)
				.field("owner", &self.owner)
				.field("price", &self.price)
				.field("gender", &self.gender)
				.field("created_time", &self.created_time)
				.finish()
		}
	}

	#[derive(TypeInfo, Encode, Decode, Clone, Debug)]
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
		type Time: Time;
		#[pallet::constant]
		type MaxLength: Get<u32>; // references: https://docs.substrate.io/reference/how-to-guides/basics/configure-runtime-constants/
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>; // https://docs.substrate.io/main-docs/build/randomness/
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
		KittyPerOwnerTooLarge,
	}

	//----------------------------------------------------------------------------
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub owners: Vec<(T::AccountId, u32)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { owners: Vec::<(T::AccountId, u32)>::new() }
		}
	}

	#[cfg(feature = "std")]
	impl<T: Config> GenesisConfig<T> {
		
		fn generate_random_kitty(owner: &T::AccountId, random_dna_seed: u32) -> Kitty<T> {
			let mut dna = Vec::<u8>::with_capacity(32);
			for _ in 0..31 {
				dna.push(0u8);
			}
			dna.push(random_dna_seed as u8);
			
			// Automatically create gender from dna
			let gender = match dna.len() {
				0 => Gender::FEMALE,
				_ => Gender::MALE,
			};

			Kitty {
				dna: dna.clone(),
				owner: owner.clone(),
				price: 10_u128.saturated_into::<BalanceOf<T>>(),
				gender,
				created_time: 0_u128.saturated_into::<Moment<T>>(),
			}
		}

		fn generate_random_kitties(owner: &T::AccountId, count: &u32) -> Vec<Kitty<T>> {
			let mut kitties: Vec<Kitty<T>> = Vec::<Kitty<T>>::new();
	
			let current_storage_count = <KittyCount<T>>::get().unwrap_or(0);
			let mut generated_count = 0u32;

			while generated_count < *count {
				let seed = current_storage_count + generated_count;
				let new_kitty = Self::generate_random_kitty(owner, seed);
				log::info!("New kittiy: {:?}", new_kitty);
				kitties.push(new_kitty);
				generated_count += 1;
			}

			kitties
		}
	}
	
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// write length of the initial vector to KittyCount
			let mut initial_count = 0;
			for (_, count) in self.owners.iter() {
				initial_count += *count;
			}
			KittyCount::<T>::put(initial_count);

			// create kitties and store them in KittyMap
			for (owner, count) in self.owners.iter() {
				let kitties: Vec<Kitty<T>> = Self::generate_random_kitties(owner, count);
				// log::info!("New kittiy: {:?}", kitties);
				for k in kitties.iter() {
					<KittyMap<T>>::insert(&k.dna, k);
				}
				let dna_vec: Vec<DNA> = kitties.iter().map(|k| k.dna.clone()).collect();
				<OwnerMap<T>>::insert(owner, dna_vec);
			}
		}
	}

	//----------------------------------------------------------------------------

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(52_108_000 + T::DbWeight::get().reads(4) + T::DbWeight::get().writes(4))]
		pub fn create_kitty(origin: OriginFor<T>, price: BalanceOf<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			let current_count = <KittyCount<T>>::get().unwrap_or(0);
			let subject = Self::generate_subject(current_count);
			let (random_value, _) = T::Randomness::random(&subject);

			let dna = random_value.encode();

			// Make sure the owner has ability to own one more kitty.
			let current_len = match <OwnerMap<T>>::get(&who) {
				Some(vec) => vec.len(),
				_ => 0 as usize,
			};
			ensure!(current_len < T::MaxLength::get() as usize, Error::<T>::KittyPerOwnerTooLarge);

			// Automatically create gender from dna
			let gender = match dna.len() {
				0 => Gender::FEMALE,
				_ => Gender::MALE,
			};

			let new_kitty: Kitty<T> = Kitty {
				dna: dna.clone(),
				owner: who.clone(),
				price,
				gender,
				created_time: T::Time::now(),
			};

			log::info!("New kittiy: {:?}", new_kitty);

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

		#[pallet::weight(38_392_000 + T::DbWeight::get().reads(2) + T::DbWeight::get().writes(2))]
		pub fn change_owner(
			origin: OriginFor<T>,
			dna: DNA,
			new_owner: T::AccountId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Make sure the new owner has ability to own one more kitty.
			let current_len = match <OwnerMap<T>>::get(&new_owner) {
				Some(vec) => vec.len(),
				_ => 0 as usize,
			};
			ensure!(current_len < T::MaxLength::get() as usize, Error::<T>::KittyPerOwnerTooLarge);

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
impl<T: Config> Pallet<T> {
	fn generate_new_owner(dna: DNA) -> Option<Vec<DNA>> {
		let mut new_vector = Vec::<DNA>::new();
		new_vector.push(dna);
		Some(new_vector)
	}

	fn generate_subject(number_kitty: u32) -> Vec<u8> {
		let mut subject = "kitty".as_bytes().to_vec();
		subject.extend(number_kitty.encode());
		subject
	}
}

