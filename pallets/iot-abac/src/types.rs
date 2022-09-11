use codec::{Decode, Encode};
use frame_support::sp_std::vec::Vec;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo,
)]
pub struct Attribute {
	pub name: Vec<u8>,
	pub value: Vec<u8>,
}

#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo,
)]
pub struct PolicyField {
	pub name: Vec<u8>,
	pub value: Vec<Vec<u8>>,
}
