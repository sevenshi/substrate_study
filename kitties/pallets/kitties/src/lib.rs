#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Randomness, Currency, ReservableCurrency}
    };
	use frame_system::pallet_prelude::*;
    use codec::{Encode, Decode};
    use sp_io::hashing::blake2_128;
    use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded};


	#[derive(Encode, Decode,TypeInfo,MaxEncodedLen)]
    pub struct Kitty(pub [u8;16]);
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        // Define KittyIndex in Runtime.
        type KittyIndex: Parameter + MaxEncodedLen + AtLeast32BitUnsigned + Default + Copy + Bounded;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        // Configurable constant for the amount of staking when create a kitty,
        // to avoid the user create a big number of kitties to attract the chain.
        #[pallet::constant]
        type StakeForEachKitty: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

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
	#[pallet::getter(fn kitties_owner)]
	pub type KittiesOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::KittyIndex,T::MaxKittyOwned>, ValueQuery>;

	

    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        KittyCreated(T::AccountId, T::KittyIndex),
        KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
        KittyListed(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
	}

	#[pallet::error]
	pub enum Error<T> {
        KittiesCountOverflow,
        NotOwner,
        SameParentIndex,
        InvalidKittyIndex,
        BuyerIsOwner,
        NotForSale,
        NotEnoughBalanceForStaking,
        NotEnoughBalanceForBuying,
		ExceedMaxKittyOwned,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			//get kitty dna
            let dna = Self::random_value(&who);
			//new kitty with stake
			Self::new_kitty_with_stake(&who,dna)?;

			Ok(())
		}
	}

	    // Helper functions.
	impl<T: Config> Pallet<T> {
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
                },
                None => 0u32.into()
            };

            let stake = T::StakeForEachKitty::get();

            T::Currency::reserve(&owner, stake)
                .map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;

            Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
            KittiesOwner::<T>::try_mutate(&owner, |vec| vec.try_push(kitty_id))
			.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;
            KittiesCount::<T>::put(kitty_id + 1u32.into());

            Self::deposit_event(Event::KittyCreated(owner.clone(), kitty_id));

            Ok(())
        }

	}

}
