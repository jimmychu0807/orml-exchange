use crate as pallet_exchange;
use sp_core::H256;
use frame_system::{mocking};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup, Zero}, testing::Header,
};
use frame_support::{
	parameter_types,
};
use orml_currencies::BasicCurrencyAdapter;
use orml_traits::parameter_type_with_key;

type UncheckedExtrinsic = mocking::MockUncheckedExtrinsic<TestRuntime>;
type AccountId = u64;
type Block = mocking::MockBlock<TestRuntime>;
type BlockNumber = u64;
type Amount = i128;
type Balance = u128;
type CurrencyId = u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

pub const DOT: CurrencyId = 1;
pub const BTC: CurrencyId = 3;

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

// Build genesis storage according to the mock runtime.
// pub fn new_test_ext() -> sp_io::TestExternalities {
// 	frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap().into()
// 	pallet_balances::GenesisConfig::default().build_storage().unwrap().into()
// }

pub struct ExtBuilder {
  // endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
  balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
  fn default() -> Self {
    Self {
      // endowed_accounts: vec![
      //   (ALICE, DOT, 1000_000_000_000_000u128),
      //   (BOB, DOT, 1000_000_000_000_000u128),
      //   (ALICE, BTC, 1000_000_000_000_000u128),
      //   (BOB, BTC, 1000_000_000_000_000u128),
      // ],
      balances: vec![]
    }
  }
}

impl ExtBuilder {
  pub fn build(self) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
      .build_storage::<TestRuntime>()
      .unwrap();

    // orml_tokens::GenesisConfig::<TestRuntime> {
    //   endowed_accounts: self.endowed_accounts,
    // }
    pallet_balances::GenesisConfig::<TestRuntime> {
      balances: vec![],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
  }
}
