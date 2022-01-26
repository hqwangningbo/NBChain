#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Encode, Decode, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Person {
    pub name: Vec<u8>,
    pub age: u8,
    pub address: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::dispatch::DispatchResult;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn person_id)]
    pub type PersonId<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_person)]
    pub type Persons<T: Config> = StorageMap<_, Blake2_128Concat, u32, Person>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        AddPerson(T::AccountId, u32, Person),
        RemovePerson(T::AccountId, u32, Person),
        DeletePerson(T::AccountId, u32, Person),
        UpdatePerson(T::AccountId, u32, Person),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        PersonIdOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn add_person(origin: OriginFor<T>, name: Vec<u8>, age: u8, address: Vec<u8>) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            let person_id = PersonId::<T>::get();
            let person = Person {
                name,
                age,
                address,
            };
            Persons::<T>::insert(person_id, person.clone());
            let next_person_id = person_id.checked_add(1).ok_or(Error::<T>::PersonIdOverflow)?;
            PersonId::<T>::put(next_person_id);
            Self::deposit_event(Event::AddPerson(caller, person_id, person));
            Ok(())
        }
        #[pallet::weight(10_000)]
        pub fn update_person(origin: OriginFor<T>, person_id: u32, name: Vec<u8>, age: u8, address: Vec<u8>) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            let new_person = Person {
                name,
                age,
                address,
            };
            Persons::<T>::try_mutate_exists(person_id, |person| {
                *person = Some(new_person.clone());
                Self::deposit_event(Event::UpdatePerson(caller, person_id, new_person));
                Ok(())
            })
        }
    }
}
