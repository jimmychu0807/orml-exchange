#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		dispatch::DispatchResult,
		inherent::Vec,
		storage::{with_transaction, TransactionOutcome},
	};
	use frame_system::pallet_prelude::*;
	use orml_traits::{
		MultiCurrency, MultiReservableCurrency, BalanceStatus,
		arithmetic::Zero,
	};

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

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	type BalanceOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
	pub enum OrderStatus {
		Pending,
		Alive,
		Executed,
		Cancelled,
		Invalid,
	}

	#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
	pub struct Order<T: Config> {
		owner:      T::AccountId,
		from_cid:   CurrencyIdOf<T>,
		from_bal:   BalanceOf<T>,
		to_cid:     CurrencyIdOf<T>,
		to_bal:     BalanceOf<T>,
		status:     OrderStatus,
		executed_with: Option<T::AccountId>,
		created_at: T::BlockNumber,
		cancelled_at: Option<T::BlockNumber>,
		executed_at: Option<T::BlockNumber>,
	}

	impl<T: Config> Order<T> {
		pub fn new_alive_order(
			owner: 			&T::AccountId,
			from_cid: 	CurrencyIdOf<T>,
			from_bal:   BalanceOf<T>,
			to_cid:     CurrencyIdOf<T>,
			to_bal:     BalanceOf<T>,
		) -> Self {
			Self {
				from_cid, from_bal, to_cid, to_bal,
				owner: owner.clone(),
				status: OrderStatus::Alive,
				executed_with: None,
				created_at: <frame_system::Pallet<T>>::block_number(),
				cancelled_at: None,
				executed_at: None,
			}
		}
	}

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub(super) type Orders<T> = StorageMap<_, Blake2_128Concat, OrderId, Order<T>>;

	#[pallet::storage]
	#[pallet::getter(fn user_orders)]
	pub(super) type UserOrders<T> = StorageMap<_, Blake2_128Concat, AccountOf<T>, Vec<OrderId>>;

	#[pallet::storage]
	#[pallet::getter(fn next_order_id)]
	pub(super) type NextOrderId<T> = StorageValue<_, OrderId, ValueQuery, DefaultNextOrderId>;

	#[pallet::type_value]
	pub(super) fn DefaultNextOrderId() -> OrderId { 0 }

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
		CannotTransferOnFrom,
		CannotTransferOnTo,
		NextOrderIdOverflow,
		NotEnoughBalance,
		NotOrderOwner,
		OrderCannotBeCancelled,
		OrderFromBalZero,
		OrderNotAvailableToExecute,
		OrderNotExist,
		OrderToBalZero,
		SameToFromCurrency,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn submit_order(
			origin: OriginFor<T>,
			from_cid: CurrencyIdOf<T>,
			from_bal: BalanceOf<T>,
			to_cid: CurrencyIdOf<T>,
			to_bal: BalanceOf<T>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// CHECK: `from_cid` and `to_cid` must be different
			ensure!(from_cid != to_cid, <Error::<T>>::SameToFromCurrency);

			// CHECK: No bal is zero
			ensure!(from_bal > Zero::zero(), <Error::<T>>::OrderFromBalZero);
			ensure!(to_bal > Zero::zero(), <Error::<T>>::OrderToBalZero);

			let order_id = Self::next_order_id();

			// CHECK: Arithmetic should use `check_*` to avoid overflow and panic
			<NextOrderId::<T>>::try_mutate(|oid| -> DispatchResult {
				*oid = oid.checked_add(1).ok_or(<Error::<T>>::NextOrderIdOverflow)?;
				Ok(())
			})?;

			T::Currency::reserve(from_cid, &who, from_bal).map_err(|_| <Error::<T>>::NotEnoughBalance)?;

			// Write to Orders
			<Orders::<T>>::insert(order_id, Order::new_alive_order(&who, from_cid, from_bal, to_cid, to_bal));

			// Write to UserOrders
			<UserOrders::<T>>::append(&who, order_id);

			// Emitting event
			Self::deposit_event(Event::OrderSubmitted(who, from_cid, from_bal, to_cid, to_bal));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn take_order(origin: OriginFor<T>, oid: OrderId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Orders::<T>>::try_mutate(oid, |order_opt| {
				if let Some(order) = order_opt {
					// CHECK: order_status
					if let OrderStatus::Alive = order.status {
						// CHECK: User has what the order to_cid and to_bal
						if T::Currency::free_balance(order.to_cid, &who) >= order.to_bal {

							// the actual transaction
							with_transaction(|| {
								// repatriate from user A (from_cid, from_bal) to user B
								if let Err(_e) = T::Currency::repatriate_reserved(order.from_cid, &order.owner, &who, order.from_bal, BalanceStatus::Reserved) {
									return TransactionOutcome::Rollback(Err(Error::<T>::CannotTransferOnFrom))
								}

								// transfer from user B (to_cid, to_bal) to user A
								if let Err(_e) = T::Currency::transfer(order.to_cid, &who, &order.owner, order.to_bal) {
									return TransactionOutcome::Rollback(Err(Error::<T>::CannotTransferOnTo))
								}

								// update storage
								order.executed_with = Some(who.clone());
								order.executed_at = Some(<frame_system::Pallet<T>>::block_number());
								order.status = OrderStatus::Executed;
								TransactionOutcome::Commit(Ok(()))
							}) // -- end of `with_transaction()` --
						} else {
							Err(Error::<T>::NotEnoughBalance)
						}
					} else {
						Err(Error::<T>::OrderNotAvailableToExecute)
					}
				} else {
					Err(<Error::<T>>::OrderNotExist)
				}
			})?;

			//Emitting event
			Self::deposit_event(Event::OrderTaken(who, oid));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn cancel_order(origin: OriginFor<T>, oid: OrderId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// CHECK: it is the owner of the order
			ensure!(Self::order_owned_by(&oid, &who), <Error::<T>>::NotOrderOwner);

			// CHECK: the order is indeed existed && status is either PENDING/ALIVE, if not return error.
			<Orders::<T>>::try_mutate(&oid, |order_opt| {
				if let Some(order) = order_opt {
					if let OrderStatus::Pending | OrderStatus::Alive = order.status {
						// DO: set the status of the order to cancelled
						order.status = OrderStatus::Cancelled;
						order.cancelled_at = Some(<frame_system::Pallet<T>>::block_number());

						// DO: unreserve the fund for the user
						T::Currency::unreserve(order.from_cid, &who, order.from_bal);

						Ok(())
					} else {
						Err(<Error::<T>>::OrderCannotBeCancelled)
					}
				} else {
					Err(<Error::<T>>::OrderNotExist)
				}
			})?;

			//Emitting event
			Self::deposit_event(Event::OrderCancelled(who, oid));
			Ok(())
		}
	} // -- End of `#[pallet::call]` --

	// Other functions defined here
	impl<T: Config> Pallet<T> {
		pub fn order_owned_by(oid: &OrderId, who: &T::AccountId) -> bool {
			match <UserOrders::<T>>::get(who) {
				Some(vec) => vec.contains(oid),
				None => false
			}
		}

	}
}
