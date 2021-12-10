#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn chain_id)]
    pub type ChainId<T> = StorageValue<_, u64, ValueQuery>;

    //小心这个没有范形
    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub chain_id: u64,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self { chain_id: 1500u64 }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            ChainId::<T>::put(self.chain_id);
        }
    }

    impl<T: Config> Get<u64> for Pallet<T> {
        fn get() -> u64 {
            Self::chain_id()
        }
    }
}
