#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		inherent::Vec
	};
	use frame_system::pallet_prelude::*;

	pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
	pub type ClassDataOf<T> = <T as orml_nft::Config>::ClassData;

	pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
	pub type TokenDataOf<T> = <T as orml_nft::Config>::TokenData;

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ItemMinted(T::AccountId, (T::ClassId, T::TokenId), Vec<u8>),
		ItemTransferred(T::AccountId, T::AccountId, (T::ClassId, T::TokenId))
	}

	#[pallet::error]
	pub enum Error<T> {
		NotAuthorized,
		MintItemError,
		NotItemOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn mint_item(
			origin: OriginFor<T>,
			cid: ClassIdOf<T>,
			metadata: Vec<u8>,
			data: TokenDataOf<T>,
		) -> DispatchResult {
			// ensure root
			ensure_root(origin.clone()).map_err(|_| Error::<T>::NotAuthorized)?;

			// To get the user
			let who = ensure_signed(origin)?;
			let tid = orml_nft::Pallet::<T>::mint(&who, cid, metadata.clone(), data)
				.map_err(|_| Error::<T>::MintItemError)?;

			Self::deposit_event(Event::ItemMinted(who, (cid, tid), metadata));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_item(
			origin: OriginFor<T>,
			to: T::AccountId,
			token: (ClassIdOf<T>, TokenIdOf<T>)
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// check: if the origin is the owner of the token, implicitly checking the existence of the
			//   class and token ID
			if !orml_nft::Pallet::<T>::is_owner(&who, token.clone()) {
				Err(Error::<T>::NotItemOwner)?
			}

			// execute: actualize the transfer
			orml_nft::Pallet::<T>::transfer(&who, &to, token.clone())?;
			Self::deposit_event(Event::ItemTransferred(who, to, token));
			Ok(())
		}
	}
}
