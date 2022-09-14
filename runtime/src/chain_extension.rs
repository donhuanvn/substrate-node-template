use codec::Encode;
use frame_support::{
	log::{error, trace},
	traits::Randomness,
};
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;

/// Contract extension for `FetchRandom`
#[derive(Default)]
pub struct FetchRandomExtension;

/// We're using enums for function IDs because contrary to raw u16 it enables
/// exhaustive matching, which results in cleaner code.
enum FuncId {
	Random(HasSeed),
	ReadAttributes(Attributes),
	ReadDIDs(DIDs),
}

#[derive(Debug)]
enum HasSeed {
	Yes,
	No,
}

#[derive(Debug)]
#[derive(Default)]
struct Attributes;

#[derive(Debug)]
#[derive(Default)]
struct DIDs;

impl TryFrom<u16> for FuncId {
	type Error = DispatchError;

	fn try_from(func_id: u16) -> Result<Self, Self::Error> {
		let id = match func_id {
			// Note: We use the first two bytes of interface selectors as function IDs,
			// While we can use anything here, it makes sense from a convention perspective.
			1100 => Self::Random(HasSeed::No),
			1101 => Self::Random(HasSeed::Yes),
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			},
		};

		Ok(id)
	}
}

fn random<T, E>(func_id: HasSeed, env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_contracts::Config + pallet_randomness_collective_flip::Config,
	<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	let mut env = env.buf_in_buf_out();
	let arg: [u8; 32] = env.read_as()?;
	let random_seed = <pallet_randomness_collective_flip::Pallet<T> as Randomness<
		T::Hash,
		T::BlockNumber,
	>>::random(&arg)
	.0;
	let random_slice = random_seed.encode();
	trace!(target: "runtime", "[ChainExtension]|call|func_id:{:?}", func_id);
	env.write(&random_slice, false, None)
		.map_err(|_| DispatchError::Other("ChainExtension failed to call random"))?;
	Ok(())
}

fn read_attributes<T, E>(
	_attrs: Attributes,
	_env: Environment<E, InitState>,
) -> Result<(), DispatchError>
where
	T: pallet_contracts::Config + pallet_randomness_collective_flip::Config,
	<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	Ok(())
}

fn read_dids<T, E>(_dids: DIDs, _env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_contracts::Config + pallet_randomness_collective_flip::Config,
	<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	E: Ext<T = T>,
{
	Ok(())
}

impl<T> ChainExtension<T> for FetchRandomExtension
where
	T: pallet_contracts::Config + pallet_randomness_collective_flip::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
	fn call<E>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let func_id = FuncId::try_from(env.func_id())?;
		match func_id {
			FuncId::Random(func_id) => random::<T, E>(func_id, env)?,
			FuncId::ReadAttributes(attrs) => read_attributes::<T, E>(attrs, env)?,
			FuncId::ReadDIDs(dids) => read_dids::<T, E>(dids, env)?,
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
