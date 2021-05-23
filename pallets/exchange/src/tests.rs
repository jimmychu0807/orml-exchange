use crate::{self as pallet_exchange, *};
use crate::tests::sp_api_hidden_includes_construct_runtime::hidden_include::traits::GenesisBuild;

use sp_core::H256;
use frame_system::{mocking};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup, Zero}, testing::Header,
};
use frame_support::{
	assert_noop, assert_ok,
	parameter_types,
};
use orml_currencies::BasicCurrencyAdapter;
use orml_traits::{
	MultiCurrency,
	parameter_type_with_key
};

// --- Constructing the mock runtime ---

type UncheckedExtrinsic = mocking::MockUncheckedExtrinsic<TestRuntime>;
type AccountId = u64;
type Block = mocking::MockBlock<TestRuntime>;
type BlockNumber = u64;
type Amount = i128;
type Balance = u128;
type CurrencyId = u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;

pub const NATIVE: CurrencyId = 0;
pub const DOT: CurrencyId = 1;
pub const BTC: CurrencyId = 2;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for TestRuntime {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for TestRuntime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_id: CurrencyId| -> Balance {
		Zero::zero()
	};
}

impl orml_tokens::Config for TestRuntime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type ExistentialDeposits = ExistentialDeposits;
	// TODO: investigate the proper OnDust setup
	// type OnDust = orml_tokens::TransferDust<Runtime, Balance>;
	type OnDust = ();
	type MaxLocks = MaxLocks;
	type WeightInfo = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = 0;
}

impl orml_currencies::Config for TestRuntime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<TestRuntime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}

impl pallet_exchange::Config for TestRuntime {
	type Event = Event;
	type Currency = Currencies;
}

frame_support::construct_runtime!(
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		Currencies: orml_currencies::{Pallet, Call, Event<T>},
		Exchange: pallet_exchange::{Pallet, Call, Storage, Event<T>},
	}
);

// --- Finish constructing the mock runtime ---

// Build genesis storage according to the mock runtime.
pub const ENDOWED_AMT: u128 = 1000;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();

	orml_tokens::GenesisConfig::<TestRuntime> {
		endowed_accounts: vec![
			(ALICE, NATIVE, ENDOWED_AMT),
			(ALICE, DOT, ENDOWED_AMT),
			(BOB, BTC, ENDOWED_AMT),
		]
	}.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn events() -> Vec<Event> {
	let evt = System::events()
		.into_iter()
		.map(|evt| evt.event)
		.collect::<Vec<_>>();

	System::reset_events();
	evt
}

fn last_event() -> Event {
	events().pop().expect("Should have one event")
}

// -- actual test cases --

#[test]
fn test_submit_order_should_fail() {
	new_test_ext().execute_with(|| {
		// Test `Error::<T>::SameToFromCurrency`
		assert_noop!(
			Exchange::submit_order(Origin::signed(ALICE), DOT, 1, DOT, 2),
			Error::<TestRuntime>::SameToFromCurrency
		);

		// Test `Error::<T>::OrderWithZeroBal`
		assert_noop!(
			Exchange::submit_order(Origin::signed(ALICE), DOT, 1, BTC, 0),
			Error::<TestRuntime>::OrderWithZeroBal
		);

		// Test `Error::<T>::OrderWithZeroBal`
		assert_noop!(
			Exchange::submit_order(Origin::signed(ALICE), DOT, 0, BTC, 1),
			Error::<TestRuntime>::OrderWithZeroBal
		);

		// Test `Error::<T>::NotEnoughBalance`
		// assert_noop!(
		// 	Exchange::submit_order(Origin::signed(ALICE), BTC, 1, DOT, 1),
		// 	Error::<TestRuntime>::NotEnoughBalance
		// );
	});
}

#[test]
fn test_submit_order_should_succeed() {
	new_test_ext().execute_with(|| {
		assert_ok!(Exchange::submit_order(Origin::signed(ALICE), DOT, 2, BTC, 1));
		assert_eq!(Tokens::free_balance(DOT, &ALICE), ENDOWED_AMT - 2);
		assert_eq!(Tokens::free_balance(BTC, &ALICE), 0);

		// Verify there is an order
		let order = Pallet::<TestRuntime>::orders(0).expect("It should contains an order");
		assert_eq!(order, Order {
			owner: ALICE,
			from_cid: DOT,
			from_bal: 2,
			to_cid: BTC,
			to_bal: 1,
			status: OrderStatus::Alive,
			executed_with: None,
			created_at: 1_u64.into(),
			cancelled_at: None,
			executed_at: None
		});

		// Verify event emitted
		assert_eq!(
			last_event(),
			Event::pallet_exchange(
				crate::Event::OrderSubmitted(ALICE, DOT, 2, BTC, 1)
			)
		);
	});
}

#[test]
fn test_cancel_order() {
	new_test_ext().execute_with(|| {
		// Test `Error::<T>::NotOrderOwner
		assert_noop!(
			Exchange::cancel_order(Origin::signed(ALICE), 0),
			Error::<TestRuntime>::NotOrderOwner
		);

		assert_ok!(Exchange::submit_order(Origin::signed(ALICE), DOT, 2, BTC, 1));

		// Test `Error::<T>::NotOrderOwner
		assert_noop!(
			Exchange::cancel_order(Origin::signed(BOB), 0),
			Error::<TestRuntime>::NotOrderOwner
		);

		// Cancel order successfully
		assert_ok!(Exchange::cancel_order(Origin::signed(ALICE), 0));

		// Amount refunded to Alice
		assert_eq!(Tokens::free_balance(DOT, &ALICE), ENDOWED_AMT);

		// Verify the order is cancelled
		let order = Pallet::<TestRuntime>::orders(0).expect("Order should exist.");
		assert_eq!(order, Order {
			owner: ALICE,
			from_cid: DOT,
			from_bal: 2,
			to_cid: BTC,
			to_bal: 1,
			status: OrderStatus::Cancelled,
			executed_with: None,
			created_at: 1_u64.into(),
			cancelled_at: 1_u64.into(),
			executed_at: None
		});

		// Verify event emitted
		assert_eq!(
			last_event(),
			Event::pallet_exchange(
				crate::Event::OrderCancelled(ALICE, 0)
			)
		);

		// Cannot cancel executed order
		assert_noop!(
			Exchange::cancel_order(Origin::signed(ALICE), 0),
			Error::<TestRuntime>::OrderCannotBeCancelled
		);

	});
}

#[test]
fn test_take_order() {
	new_test_ext().execute_with(|| {
		assert_ok!(Exchange::submit_order(Origin::signed(ALICE), DOT, 2, BTC, 1));

		// Test `Error::<T>::OrderNotExist`
		assert_noop!(
			Exchange::take_order(Origin::signed(BOB), 1),
			Error::<TestRuntime>::OrderNotExist
		);

		// Test `Error::<T>::NotEnoughBalance`
		assert_noop!(
			Exchange::take_order(Origin::signed(CHARLIE), 0),
			Error::<TestRuntime>::NotEnoughBalance
		);

		assert_ok!(Exchange::take_order(Origin::signed(BOB), 0));

		// Verify ALICE and BOB balance has updated
		assert_eq!(Tokens::free_balance(DOT, &ALICE), ENDOWED_AMT - 2);
		assert_eq!(Tokens::free_balance(BTC, &ALICE), 1);
		assert_eq!(Tokens::free_balance(DOT, &BOB), 2);
		assert_eq!(Tokens::free_balance(BTC, &BOB), ENDOWED_AMT - 1);

		// Verify the order status
		let order = Pallet::<TestRuntime>::orders(0).expect("It should contains an order");
		assert_eq!(order, Order {
			owner: ALICE,
			from_cid: DOT,
			from_bal: 2,
			to_cid: BTC,
			to_bal: 1,
			status: OrderStatus::Executed,
			executed_with: Some(BOB),
			created_at: 1_u64.into(),
			cancelled_at: None,
			executed_at: 1_u64.into()
		});

		// Verify emitted
		assert_eq!(
			last_event(),
			Event::pallet_exchange(
				crate::Event::OrderTaken(BOB, 0)
			)
		);

		// Cannot cancel executed order
		assert_noop!(
			Exchange::cancel_order(Origin::signed(ALICE), 0),
			Error::<TestRuntime>::OrderCannotBeCancelled
		);
	});
}
