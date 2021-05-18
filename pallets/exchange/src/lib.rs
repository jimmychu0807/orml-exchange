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
	use orml_traits::{MultiCurrency, MultiReservableCurrency};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: MultiReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	pub type OrderId = u64;
	pub type ExecutionId = u64;

	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	type BalanceOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
	pub enum OrderStatus {
		PENDING,
		ALIVE,
		EXECUTED,
		CANCELLED,
		INVALID,
	}

	#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
	pub enum ExecutionStatus {
		SUCCEEDED,
		FAILED,
	}

	#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
	pub struct Order<T: Config> {
		owner:      T::AccountId,
		from_cid:   CurrencyIdOf<T>,
		from_bal:   BalanceOf<T>,
		to_cid:     CurrencyIdOf<T>,
		to_bal:     BalanceOf<T>,
		status:     OrderStatus,
		created_at: T::BlockNumber,
	}

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub(super) type Orders<T> = StorageMap<_, Blake2_128Concat, OrderId, Order<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OrderSubmitted(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
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
		pub fn submit_order(
			origin: OriginFor<T>,
			from_cid: CurrencyIdOf<T>,
			from_bal: BalanceOf<T>,
			to_cid: CurrencyIdOf<T>,
			to_bal: BalanceOf<T>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Emitting event
			Self::deposit_event(Event::OrderSubmitted(who, from_cid, from_bal, to_cid, to_bal));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn take_order(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//Emitting event
			Self::deposit_event(Event::OrderTaken(who, 0));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_order(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//Emitting event
			Self::deposit_event(Event::OrderCancelled(who, 0));
			Ok(())
		}
	}
}
