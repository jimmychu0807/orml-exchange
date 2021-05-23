#![cfg_attr(not(feature = "std"), no_std)]

use codec::FullCodec;
use frame_support::pallet_prelude::*;
use core::fmt::{Debug};
use sp_runtime::traits::AtLeast32BitUnsigned;

pub mod arithmetic;

pub trait MultiCurrency<AccountId> {
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	fn free_balance(cid: Self::CurrencyId, user: &AccountId) -> Self::Balance;
	fn transfer(
		cid: Self::CurrencyId,
		from: &AccountId,
		to: &AccountId,
		bal: Self::Balance
	) -> Result<(), ()>;
}

pub trait MultiReservableCurrency<AccountId>: MultiCurrency<AccountId> {
	fn reserve(cid: Self::CurrencyId, user: &AccountId, bal: Self::Balance) -> Result<(), ()>;
	fn repatriate_reserved(
		cid: Self::CurrencyId,
		from: &AccountId,
		to: &AccountId,
		bal: Self::Balance,
		status: BalanceStatus
	) -> Result<(), ()>;
	fn unreserve(cid: Self::CurrencyId, user: &AccountId, bal: Self::Balance);
}


pub enum BalanceStatus {
	Reserved,
	Free
}
