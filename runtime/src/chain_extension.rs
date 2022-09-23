use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::sp_std::vec::Vec;
use sp_runtime::DispatchError;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use pallet_abac::{
	self,
	Moment,
	types::Endorsement,
};

pub const MAX_STRING_LENGTH: usize = 64; // limitation of both key and value of attributes.

/// Contract extension for `AbacChainExtension`
#[derive(Default)]
pub struct AbacChainExtension;

enum FuncId {
	ReadAttributeValue,
	CheckValidEndorsement,
}

impl TryFrom<u16> for FuncId {
	type Error = DispatchError;

	fn try_from(func_id: u16) -> Result<Self, Self::Error> {
		let id = match func_id {
			0x0001 => Self::ReadAttributeValue,
			0x0002 => Self::CheckValidEndorsement,
			_ => {
				log::error!("Called an unregistered `func_id`: {}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			},
		};
		Ok(id)
	}
}

#[derive(Debug, PartialEq, Encode, Decode, MaxEncodedLen)]
struct ReadAttrInput<AccountId> {
	identity: AccountId,
	attr_name: [u8; MAX_STRING_LENGTH],
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct CheckEndorsementInput<AccountId> {
	identity: AccountId,
	attr_name: [u8; MAX_STRING_LENGTH],
	endorsers: Vec<AccountId>,
}

fn convert_slice_u8_to_vec_u8(input: &[u8; MAX_STRING_LENGTH]) -> Vec<u8> {
	let mut result = Vec::<u8>::with_capacity(MAX_STRING_LENGTH);
	for &c in input.iter() {
		if c == 0u8 {
			break;
		}
		result.push(c.clone());
	}
	result
}

fn read_access_control_attribute_value<T, E>(
	env: Environment<E, InitState>,
) -> Result<RetVal, DispatchError>
where
	E: Ext<T = T>,
	T: pallet_contracts::Config + pallet_abac::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
	// Parse input parameters from function call on the policy smart-contract.
	let mut env = env.buf_in_buf_out();
	log::debug!("read_access_control_attribute_value env.in_len: {:?}", env.in_len());
	let input: ReadAttrInput<T::AccountId> = env.read_as()?;

	let identity = input.identity;
	// Convert attribute key of rust slice [] to attribute key of rust Vec<u8>.
	let key = convert_slice_u8_to_vec_u8(&input.attr_name);
	log::debug!("read_access_control_attribute_value key = {:?}", key);

	// Query the storage of pallet_abac with the above inputs.
	let attr_value = 	match <pallet_abac::Pallet<T>>::attr_of(identity, key) {
		Some(v) => v.value,
		None => Vec::<u8>::new()
	};
	log::debug!("read_access_control_attribute_value attr_value = {:?}", attr_value);

	// Return the result to function on the policy smart-contract.
	let return_slice = attr_value.encode();
	env.write(&return_slice, false, None)
		.map_err(|_| DispatchError::Other("AbacChainExtension failed to read attribute value"))?;

	// Return a status code of successful status.
	Ok(RetVal::Converging(0))
}

fn check_attribute_had_valid_endorsement<T, E>(
	env: Environment<E, InitState>,
) -> Result<RetVal, DispatchError>
where
	E: Ext<T = T>,
	T: pallet_contracts::Config + pallet_abac::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
	// Parse input parameters from function call on the policy smart-contract.
	let mut env = env.buf_in_buf_out();
	log::debug!("check_attribute_had_valid_endorsement env.in_len: {:?}", env.in_len());
	let input: CheckEndorsementInput<T::AccountId> = env.read_as_unbounded(env.in_len())?;

	let identity = input.identity;
	// Convert attribute key of rust slice [] to attribute key of rust Vec<u8>.
	let key = convert_slice_u8_to_vec_u8(&input.attr_name);
	log::debug!("check_attribute_had_valid_endorsement key = {:?}", key);
	let endorsers = input.endorsers;

	let mut valid = false;
	let now_block_number = <frame_system::Pallet<T>>::block_number(); // used to check validity of an endorsement.
	// Query the storage of pallet_abac with each endorser listed in the input.
	for endorser in endorsers.iter() {
		let endorsement: Option<Endorsement<T::BlockNumber, Moment<T>>> =
			<pallet_abac::Pallet<T>>::endorsement_of((&identity, &key, endorser));
		if let Some(e) = endorsement {
			if e.validity > now_block_number {
				valid = true;
				break;
			}
		}
	}
	log::debug!("check_attribute_had_valid_endorsement valid = {:?}", valid);

	// Return the result to function on the policy smart-contract.
	let return_slice = valid.encode();
	env.write(&return_slice, false, None)
		.map_err(|_| DispatchError::Other("AbacChainExtension failed to check valid endorsement of attribute"))?;

	// Return a status code of successful status.
	Ok(RetVal::Converging(0))
}

impl<T> ChainExtension<T> for AbacChainExtension
where
	T: pallet_contracts::Config + pallet_abac::Config,
	<T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
	fn call<E>(
		&mut self,
		env: Environment<E, InitState>,
	) -> pallet_contracts::chain_extension::Result<RetVal>
	where
		E: Ext<T = T>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let func_id = FuncId::try_from(env.func_id());
		if func_id.is_err() {
			return Ok(RetVal::Converging(1)); // Return with error to smart-contract
		}

		let func_id = func_id.unwrap();
		match func_id {
			FuncId::ReadAttributeValue => read_access_control_attribute_value::<T, E>(env),
			FuncId::CheckValidEndorsement => check_attribute_had_valid_endorsement::<T, E>(env),
			// _ => Ok(RetVal::Converging(1)),
		}
	}
}
