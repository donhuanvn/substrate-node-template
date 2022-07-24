//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Kitty;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_support::sp_runtime::SaturatedConversion;

benchmarks! {
	create_kitty {
		let dna : Vec<u8> = b"dna".to_vec();
		let price = 10_u128.saturated_into::<BalanceOf<T>>();
		let caller: T::AccountId = whitelisted_caller();
	}: create_kitty(RawOrigin::Signed(caller), dna, price)

	verify {
		assert_eq!(KittyCount::<T>::get(), Some(1));
	}

	impl_benchmark_test_suite!(Kitty, crate::mock::new_test_ext(), crate::mock::Test);
}
