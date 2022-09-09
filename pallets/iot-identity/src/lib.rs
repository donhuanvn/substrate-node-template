#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
  use frame_support::sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_identity_count)]
	pub type IdentityCount<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IdentityAdded(u32, Vec<u8>, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		IdentityLimitationReached,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_identity(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let current_count = Self::get_identity_count().unwrap_or(0);
			<IdentityCount<T>>::put(current_count + 1);
			Self::deposit_event(Event::IdentityAdded(current_count + 1, name, who));
      Ok(())
		}

    #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
    pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
      let _who = ensure_signed(origin)?;

      match Self::get_identity_count() {
        None => return Err(Error::<T>::NoneValue.into()),
        Some(old) => {
          let new = old.checked_add(1).ok_or(Error::<T>::IdentityLimitationReached)?;
          <IdentityCount<T>>::put(new);
          Ok(())
        },
      }
    }
	}
}
