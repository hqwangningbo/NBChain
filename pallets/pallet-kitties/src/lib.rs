#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{ExistenceRequirement, Randomness, Currency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use codec::{Encode, Decode};
    use sp_io::hashing::blake2_128;
    use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, StaticLookup};

    #[derive(Clone, Eq, PartialEq, Encode, Decode, scale_info::TypeInfo)]
    pub struct Kitty(pub [u8; 16]);

    #[derive(Clone, Eq, PartialEq, Encode, Decode, scale_info::TypeInfo)]
    pub enum Gender {
        Male,
        Female,
    }

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        // Define KittyIndex in Runtime.
        type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        // Configurable constant for the amount of staking when create a kitty,
        // to avoid the user create a big number of kitties to attract the chain.
        #[pallet::constant]
        type StakeForEachKitty: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxCreated: Get<u8>;

        #[pallet::constant]
        type BreedFee: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        KittyCreated(T::AccountId, T::KittyIndex),
        KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
        KittyListed(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
        // pause_at
        Paused(T::BlockNumber),
        // unpause_at
        UnPaused(T::BlockNumber),
        SetAdmin(T::AccountId),
    }

    /// The pallet admin key.
    #[pallet::storage]
    #[pallet::getter(fn admin_key)]
    pub(super) type Admin<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

    /// Storage for tracking all the kitties
    #[pallet::storage]
    #[pallet::getter(fn kitties_count)]
    pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

    /// Storage for every kitty.
    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

    /// Storage for every kitty.
    #[pallet::storage]
    #[pallet::getter(fn created)]
    pub type Created<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u8, ValueQuery>;

    /// Storage for kitties which are listed for sale.
    /// If the list price (Option<BalanceOf<T>>) is None, means the specific kitty is not for sale.
    #[pallet::storage]
    #[pallet::getter(fn kitties_list_for_sales)]
    pub type ListForSale<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

    /// Storage for tracking the ownership of kitties.
    #[pallet::storage]
    #[pallet::getter(fn owner)]
    pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

    /// Storage for tracking the gender of kitties.
    #[pallet::storage]
    #[pallet::getter(fn kitty_gender)]
    pub type KittyGender<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Gender>, ValueQuery>;

    /// The emergency stop.
    #[pallet::storage]
    #[pallet::getter(fn in_emergency)]
    pub(super) type InEmergency<T: Config> = StorageValue<_, bool, ValueQuery>;


    #[pallet::error]
    pub enum Error<T> {
        KittiesCountOverflow,
        NotOwner,
        SameParentIndex,
        SameGender,
        InvalidKittyIndex,
        BuyerIsOwner,
        NotForSale,
        NotEnoughBalanceForStaking,
        NotEnoughBalanceForBuying,
        NoCreationCount,
        InEmergency,
        OnlyInEmergency,
        RequireAdmin,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn create(origin: OriginFor<T>) -> DispatchResult {
            ensure!(!Self::is_in_emergency(), Error::<T>::InEmergency);
            let who = ensure_signed(origin)?;
            let dna = Self::random_value(&who);
            if who == Self::admin_key() {
                Self::new_kitty_with_stake(&who, dna)?;
                Created::<T>::mutate(&who, |count| {
                    *count = count.saturating_add(1);
                });
            } else {
                ensure!(Self::created(&who) < T::MaxCreated::get(),Error::<T>::NoCreationCount);
                Self::new_kitty_with_stake(&who, dna)?;
                Created::<T>::mutate(&who, |count| {
                    *count = count.saturating_add(1);
                });
            }
            Ok(())
        }

        /// Transfer a kitty from owner to another.
        #[pallet::weight(1_000)]
        pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
            ensure!(!Self::is_in_emergency(), Error::<T>::InEmergency);
            let who = ensure_signed(origin)?;
            // Ensure transfer only from the OWNER of kitties.
            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
            // Update storage.
            Owner::<T>::insert(kitty_id, Some(new_owner.clone()));
            // Emit the event.
            Self::deposit_event(Event::KittyTransferred(who, new_owner, kitty_id));
            Ok(())
        }

        /// Breed a kitty from other 2 kitties (Allow the kitty parents belong to other owners).
        #[pallet::weight(1_000)]
        pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult {
            ensure!(!Self::is_in_emergency(), Error::<T>::InEmergency);
            let who = ensure_signed(origin)?;
            // Ensure the parents are not same.
            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);
            // Ensure there're the parents in the Storage.
            let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
            let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

            let kitty1_gender = KittyGender::<T>::get(kitty_id_1);
            let kitty2_gender = KittyGender::<T>::get(kitty_id_2);

            ensure!(!kitty1_gender.eq(&kitty2_gender), Error::<T>::SameGender);

            // Breed new kitty from the parents.
            let dna_1 = kitty1.0;
            let dna_2 = kitty2.0;
            let selector = Self::random_value(&who);
            let mut new_dna = [0u8; 16];
            for i in 0..dna_1.len() {
                new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
            }
            Self::new_kitty_with_stake(&who, new_dna)?;
            T::Currency::transfer(&who, &Self::admin_key(), T::BreedFee::get(), ExistenceRequirement::KeepAlive)?;
            Ok(())
        }

        /// Set a price and list a kitty for sale. (Allow set None which means NOT_FOR_SALE.)
        #[pallet::weight(1_000)]
        pub fn sell(origin: OriginFor<T>, kitty_id: T::KittyIndex, price: Option<BalanceOf<T>>) -> DispatchResult {
            ensure!(!Self::is_in_emergency(), Error::<T>::InEmergency);
            let who = ensure_signed(origin)?;
            // Ensure only the kitty owner can sell it.
            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);
            // Set a price. If the price is None, it means the kitty is not for sale.
            ListForSale::<T>::mutate_exists(kitty_id, |p| *p = Some(price));
            // Emit event.
            Self::deposit_event(Event::KittyListed(who, kitty_id, price));

            Ok(())
        }

        /// Buy a kitty from its owner.
        #[pallet::weight(1_000)]
        pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
            ensure!(!Self::is_in_emergency(), Error::<T>::InEmergency);
            let buyer = ensure_signed(origin)?;
            let owner = Owner::<T>::get(kitty_id).unwrap();
            // Ensure the buyer is not the owner.
            ensure!(Some(buyer.clone()) != Some(owner.clone()), Error::<T>::BuyerIsOwner);
            // If the price in the ListForSale is None, the kitty is not for sale.
            let amount = ListForSale::<T>::get(kitty_id).ok_or(Error::<T>::NotForSale)?;
            // Check the buyer with enough balance to buy. Ensure the free balance can pay and stake also.
            let buyer_balance = T::Currency::free_balance(&buyer);
            ensure!(buyer_balance >= amount , Error::<T>::NotEnoughBalanceForBuying);
            // Transfer the price from buyer to the seller.
            T::Currency::transfer(&buyer, &owner, amount, ExistenceRequirement::KeepAlive)?;
            // Remove from the List.
            ListForSale::<T>::remove(kitty_id);
            // Update the storage with the new owner.
            Owner::<T>::insert(kitty_id, Some(buyer.clone()));
            // Emit the event.
            Self::deposit_event(Event::KittyTransferred(owner, buyer, kitty_id));

            Ok(())
        }
        /// Set this pallet admin key
        /// Note: for super admin
        #[pallet::weight(100_000_000u64)]
        pub fn set_admin(
            origin: OriginFor<T>,
            new_admin: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let new_admin = T::Lookup::lookup(new_admin)?;

            Admin::<T>::mutate(|admin| *admin = new_admin.clone());

            Self::deposit_event(Event::SetAdmin(new_admin));

            Ok(())
        }

        #[pallet::weight(100_000_000u64)]
        pub fn pause(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who==Self::admin_key(), Error::<T>::RequireAdmin);

            InEmergency::<T>::try_mutate(|in_emergency| {
                if !*in_emergency {
                    *in_emergency = true;

                    Self::deposit_event(Event::Paused(Self::now()));
                }

                Ok(())
            })
        }
        #[pallet::weight(100_000_000u64)]
        pub fn unpause(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who==Self::admin_key(), Error::<T>::RequireAdmin);

            InEmergency::<T>::try_mutate(|in_emergency| {
                if *in_emergency {
                    *in_emergency = false;

                    Self::deposit_event(Event::UnPaused(Self::now()));
                }

                Ok(())
            })
        }
    }

    // Helper functions.
    impl<T: Config> Pallet<T> {
        fn is_in_emergency() -> bool {
            InEmergency::<T>::get()
        }
        fn now() -> T::BlockNumber {
            frame_system::Pallet::<T>::block_number()
        }
        fn gen_gender() -> Gender {
            let random = T::Randomness::random(&b"nbgender"[..]).0;
            match random.as_ref()[0] % 2 {
                0 => Gender::Male,
                _ => Gender::Female,
            }
        }
        fn random_value(sender: &T::AccountId) -> [u8; 16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );
            payload.using_encoded(blake2_128)
        }

        // Helper function for optimizing the codes from create() and transfer().
        fn new_kitty_with_stake(owner: &T::AccountId, dna: [u8; 16]) -> DispatchResult {
            let kitty_id = match Self::kitties_count() {
                Some(id) => {
                    ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
                    id
                }
                None => 0u32.into()
            };

            Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
            Owner::<T>::insert(kitty_id, Some(owner.clone()));
            KittyGender::<T>::insert(kitty_id, Some(Self::gen_gender()));
            KittiesCount::<T>::put(kitty_id + 1u32.into());

            Self::deposit_event(Event::KittyCreated(owner.clone(), kitty_id));

            Ok(())
        }
    }
}
