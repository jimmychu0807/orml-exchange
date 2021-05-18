#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	pub type OrderId = u64;
	pub type ExecutionId = u64;

	pub enum OrderStatus = {
		PENDING,
		ALIVE,
		EXECUTED,
		CANCELLED,
		INVALID,
	};

	pub enum ExecutionStatus = {
		SUCCEEDED,
		FAILED,
	}

	pub struct Order<T: Config> {
		owner:      T::AccountId,
		from_cid:   T::CurrencyId,
		from_bal:   T::Balance,
		to_cid:     T::CurrencyId,
		to_bal:     T::Balance,
		status:     OrderStatus,
		created_at: T::BlockNumber,
	}

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub(super) type Orders<T> = StorageMap<_, Blake2_128Concat, OrderId, Order>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OrderSubmitted(T::AccountId, T::CurrencyId, T::Balance, T::CurrencyId, T::Balance),
		OrderTaken(T::AccountId, OrderId),
		OrderCancelled(T::AccountId, OrderId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {

	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn submit_order(origin: OriginFor<T>, from_currency, to_currency) -> DispatchResult {
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn take_order(origin: OriginFor<T>) -> DispatchResult {
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_order(origin: OriginFor<T>) -> DispatchResult {
			Ok(())
		}
	}
}
