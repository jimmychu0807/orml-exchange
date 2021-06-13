#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_std::prelude::*;
use sp_runtime::{RuntimeDebug};

use crate::{AccountId};

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ItemType {
	Weapon,
	HeadGear,
	Armor,
	Shoe
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ItemClassData {
	pub item_type: ItemType,
	pub info: Vec<u8>,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ItemTokenData {
	pub name: Vec<u8>,
	pub attack: u64,
	pub defense: u64,
	pub speed: u64,
	pub dexterity: u64,
}

pub const MAX_CLASS_METADATA: u32  = 1024;
pub const MAX_TOKEN_METADATA: u32  = 1024;

// This is returning the testnet genesis config
pub fn items_genesis(owner: &AccountId) ->
	Vec<(
		AccountId,
		Vec<u8>,
		ItemClassData,
		Vec<(AccountId, Vec<u8>, ItemTokenData)>
	)>
{
	vec![(
		owner.clone(),
		b"sword".to_vec(),
		ItemClassData {
			item_type: ItemType::Weapon,
			info: b"Sword".to_vec(),
		},
		vec![(
			owner.clone(),
			b"iron_sword".to_vec(),
			ItemTokenData {
				name: b"Iron Sword".to_vec(),
				attack: 7,
				defense: 0,
				speed: 0,
				dexterity: 0
			}
		), (
			owner.clone(),
			b"steel_sword".to_vec(),
			ItemTokenData {
				name: b"Steel Sword".to_vec(),
				attack: 14,
				defense: 0,
				speed: 0,
				dexterity: 2
			}
		)]
	), (
		owner.clone(),
		b"hat".to_vec(),
		ItemClassData {
			item_type: ItemType::HeadGear,
			info: b"Hat".to_vec(),
		},
		vec![]
	)]
}
