use crate::PriceOracle;

use frame_support::parameter_types;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Get, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

frame_support::construct_runtime!(
	pub enum TestRuntime where
	Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
	System: frame_system,
	Balances: pallet_balances,

	PalletToMock: crate::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const ValueToMint: u64 = 17890;
}

pub const ALICE: u64 = 1;
//pub const BOB: u64 = 2;

impl frame_system::Config for TestRuntime {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = u64;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = Event;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type Origin = Origin;
	type PalletInfo = PalletInfo;
	type SS58Prefix = SS58Prefix;
	type SystemWeightInfo = ();
	type Version = ();
}

pub const EXISTENTIAL_DEPOSIT: u64 = 0;

parameter_types! {
	pub const ExistentialDeposit: u64 = EXISTENTIAL_DEPOSIT;
	pub const MaxLocks: u32 = 1000000;
	pub const MaxReserves: u32 = 1000000;
}

impl pallet_balances::Config for TestRuntime {
	type AccountStore = System;
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

// type ValueToMint<T> = PhantomData<T>;
// impl<T> sp_runtime::traits::Get<BalanceOf<T>> for ValueToMint<T> {
//     fn get() -> BalanceOf<T> {
// 	return 1789;
//     }
// }

type SomePriceOracle = ();
impl PriceOracle for SomePriceOracle {
	type Error = ();
	fn get_price() -> Result<u64, Self::Error> {
		Ok(17)
	}
}

//TestRuntime::Balances::

impl crate::Config for TestRuntime {
	type Event = Event;
	type Currency = Balances;
	type ValueToMint = ValueToMint;
	type SomePriceOracle = SomePriceOracle;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();
	let genesis_config = pallet_balances::GenesisConfig::<TestRuntime> {
		balances: vec![(1, 1000)],
	};
	genesis_config.assimilate_storage(&mut t).unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	// In order to emit events the block number must be more than 0
	ext.execute_with(|| System::set_block_number(1));
	ext
}
