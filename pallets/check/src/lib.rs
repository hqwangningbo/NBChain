#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_application_crypto::sr25519::Signature;
    use frame_support::sp_runtime::MultiSignature;
    use frame_support::inherent::Vec;
    use frame_support::sp_runtime::traits::Verify;
    use sp_core::crypto::AccountId32;
    use frame_support::sp_runtime::app_crypto::TryFrom;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn chain_id)]
    pub type ChainId<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        SignatureVerify(bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        StorageOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn check(
            origin: OriginFor<T>,
            address: AccountId32,
            message: Vec<u8>,
            signature: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let u: &[u8; 64] = <&[u8; 64]>::try_from(signature.as_slice()).unwrap();
            let sign = Signature::from_raw(*u);
            let multi_sig = MultiSignature::from(sign);
            let result = multi_sig.verify(message.as_slice(), &address);
            Self::deposit_event(Event::SignatureVerify(result));
            Ok(().into())
        }
    }
}
