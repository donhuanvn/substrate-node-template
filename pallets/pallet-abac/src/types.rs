use codec::{Decode, Encode};
use frame_support::sp_std::vec::Vec;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

/// Access control attribute.
/// This is only the type of record in pallet runtime storage.
#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, TypeInfo, RuntimeDebug,
)]
pub struct Attr<Moment> {
	pub name: Vec<u8>,
	pub value: Vec<u8>,
	pub updated_time: Moment,
}

/// Access control attribute.
/// This is intended for input paramaters of extrinsic.
#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, TypeInfo, RuntimeDebug,
)]
pub struct AttrInput {
  pub name: Vec<u8>,
  pub value: Vec<u8>,
}

/// Endorsements for an access control attribute
/// This is only the type of record in pallet runtime storage.
#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, TypeInfo, RuntimeDebug,
)]
pub struct Endorsement<BlockNumber, Moment> {
	pub validity: BlockNumber,
	pub endorsed_time: Moment,
}

/// Policy associated with an identity (role as access control object).
/// This is only the type of record in pallet runtime storage.
#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, TypeInfo, RuntimeDebug,
)]
pub struct Policy<AccountId, Moment> {
	pub name: Vec<u8>,
	pub attached_by: AccountId,
	pub attached_time: Moment,
}
