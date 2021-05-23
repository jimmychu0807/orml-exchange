use crate::{self as pallet_exchange, Error};
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

}

#[test]
fn test_take_order() {

}
