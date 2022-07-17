

//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Kitty;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_support::sp_runtime::SaturatedConversion;
use frame_benchmarking::BenchmarkError::Stop;

benchmarks! {
	create_kitty {
		let price = 10_u128.saturated_into::<BalanceOf<T>>();
		let caller: T::AccountId = whitelisted_caller();
	}: create_kitty(RawOrigin::Signed(caller), price)

  change_owner {
    let caller: T::AccountId = whitelisted_caller();
    let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
    let price = 10_u128.saturated_into::<BalanceOf<T>>();
    <Kitty<T>>::create_kitty(caller_origin, price)?;
    let dna = match <OwnerMap<T>>::get(&caller) {
      Some(vec) => vec[0].clone(),
      _ => return Err(Stop("Can not create a kitty for change_owner benchmarking"))
    };
    let new_owner: T::AccountId = whitelisted_caller() ;
  }: change_owner(RawOrigin::Signed(caller), dna.clone(), new_owner.clone())

	verify {
    let kitties: Vec<DNA> = <OwnerMap<T>>::get(&new_owner).unwrap_or(Vec::<DNA>::new());
    assert_eq!(kitties.len(), 1);
	}

	impl_benchmark_test_suite!(Kitty, crate::mock::new_test_ext(), crate::mock::Test);
}
