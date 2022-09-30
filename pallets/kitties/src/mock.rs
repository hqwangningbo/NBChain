use crate as pallet_kitties;
use frame_support::{
	traits::{ConstU16, ConstU64},
	PalletId,
};
use frame_system as system;
pub use pallet_kitties::{Error, Event as KittyEvent};
use pallet_randomness_collective_flip;
use sp_core::H256;
use sp_runtime::{
	parameter_types,
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Kitties: pallet_kitties::{Pallet, Call, Storage, Event<T>},
		Randomness: pallet_randomness_collective_flip::{Pallet, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}

impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = u128;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_types! {
	pub const CreatKittyDeposit: u128 = 1_000_000_000_000_000_000;
	pub const KittiesPalletId: PalletId = PalletId(*b"nb/kitty");
}

impl pallet_kitties::Config for Test {
	type Event = Event;
	type Randomness = Randomness;
	type KittyIndex = u32;
	type Currency = Balances;
	type CreatKittyDeposit = CreatKittyDeposit;
	type PalletId = KittiesPalletId;
}

impl pallet_randomness_collective_flip::Config for Test {}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 100_000_000_000_000_000_000), (2, 100_000_000_000_000_000_000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub(crate) fn last_event() -> Event {
	system::Pallet::<Test>::events().pop().expect("Event expected").event
}

pub(crate) fn expect_event<E: Into<Event>>(e: E) {
	assert_eq!(last_event(), e.into());
}
