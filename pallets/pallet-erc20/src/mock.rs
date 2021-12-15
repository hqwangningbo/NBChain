use crate as pallet_erc20;
use frame_support::{
    parameter_types,
    traits::{GenesisBuild, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

pub use pallet_erc20::{Config, Error, Event as ERC20Event};

pub use pallet_balances::Error as BalancesError;
use sp_core::crypto::AccountId32;
use sp_core::sp_std::str::FromStr;

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
		Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
		ERC20: pallet_erc20::{Pallet, Config<T>, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;

}

impl system::Config for Test {
    type AccountData = pallet_balances::AccountData<u128>;
    type AccountId = u128;
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
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}

parameter_types! {
	pub const MaxLocks: u32 = 50;
	pub static ExistentialDeposit: u64 = 0;
}

impl pallet_balances::Config for Test {
    type AccountStore = System;
    type Balance = u128;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for Test {}

impl pallet_erc20::Config for Test {
    type Event = Event;
}

// Build test environment by setting the admin `key` for the Genesis.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t =
        frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 100_000_000_000_000_000_000),
            (2, 100_000_000_000_000_000_000),
            (3, 100_000_000_000_000_000_000),
        ],
    }.assimilate_storage(&mut t)
        .unwrap();

    pallet_erc20::GenesisConfig::<Test> {
        name: String::from("NB Token").into_bytes(),
        symbol: String::from("NB").into_bytes(),
        decimal: 18,
        owner: 1,
    }.assimilate_storage(&mut t).unwrap();

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

/// Run until a particular block.
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
    }
}
