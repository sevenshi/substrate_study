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
	use codec::{Decode, Encode};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, Randomness, ReservableCurrency},
		transactional, ensure,
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, MaxEncodedLen)]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub owner: AccountOf<T>,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + TypeInfo {
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
	pub type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty<T>>, ValueQuery>;

	/// Storage for every kitty.
	#[pallet::storage]
	#[pallet::getter(fn kitties_owner)]
	pub type KittiesOwner<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::KittyIndex, T::MaxKittyOwned>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
		KittyPriceSet(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ExceedKittyOwned,
		KittiesCountOverflow,
		NotOwner,
		SameParentIndex,
		InvalidKittyIndex,
		BuyerIsOwner,
		KittyNotForSale,
		NotEnoughBalanceForStaking,
		NotEnoughBalanceForBuying,
		TransferToSelf,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(1_000)]
		#[transactional]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			//get kitty dna
			let dna = Self::random_value(&who);
			//new kitty with stake
			Self::new_kitty_with_stake(who, dna)?;
			Ok(())
		}

		#[pallet::weight(1_000)]
		#[transactional]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);
			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			let dna_1 = kitty1.dna;
			let dna_2 = kitty2.dna;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
			}
			//new kitty with stake
			Self::new_kitty_with_stake(who, new_dna)?;

			Ok(())
		}

		#[pallet::weight(1_000)]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			to: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who != to, <Error<T>>::TransferToSelf);
			let kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
			ensure!(Self::is_kitty_owner(kitty_id, &who)?, Error::<T>::NotOwner);
			let stake_amount = T::StakeForEachKitty::get();
			T::Currency::reserve(&to, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;
			T::Currency::unreserve(&kitty.owner, stake_amount);
			Self::transfer_kitty_to(kitty_id, to)?;
			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
			ensure!(Self::is_kitty_owner(kitty_id, &who)?, Error::<T>::NotOwner);
			kitty.price = new_price.clone();
			Kitties::<T>::insert(kitty_id, Some(kitty));
			Self::deposit_event(Event::KittyPriceSet(who, kitty_id, new_price));
			Ok(())
		}

		#[pallet::weight(1_000)]
		#[transactional]
		pub fn buy_kitty(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			let kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
			ensure!(kitty.owner != buyer, <Error<T>>::TransferToSelf);

			if let Some(ask_price) = kitty.price {
				let amount = ask_price;
				let stake_amount = T::StakeForEachKitty::get();
				let buyer_balance = T::Currency::free_balance(&buyer);
				ensure!(
					buyer_balance > (amount + stake_amount),
					Error::<T>::NotEnoughBalanceForBuying
				);
				T::Currency::reserve(&buyer, stake_amount)
					.map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;
				T::Currency::unreserve(&kitty.owner, stake_amount);
				T::Currency::transfer(
					&buyer,
					&kitty.owner,
					amount,
					frame_support::traits::ExistenceRequirement::KeepAlive,
				)?;
				Self::transfer_kitty_to(kitty_id, buyer)?;
			} else {
				Err(<Error<T>>::KittyNotForSale)?;
			}
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
		fn new_kitty_with_stake(owner: T::AccountId, dna: [u8; 16]) -> Result<(), Error<T>> {
			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				},
				None => 0u32.into(),
			};
			KittiesOwner::<T>::try_mutate(&owner, |vec| vec.try_push(kitty_id))
				.map_err(|_| <Error<T>>::ExceedKittyOwned)?;

			let stake = T::StakeForEachKitty::get();

			T::Currency::reserve(&owner, stake)
				.map_err(|_| Error::<T>::NotEnoughBalanceForStaking)?;

			Kitties::<T>::insert(kitty_id, Some(Kitty { dna, price: None, owner: owner.clone() }));



			KittiesCount::<T>::put(kitty_id + 1u32.into());
			Self::deposit_event(Event::KittyCreated(owner.clone(), kitty_id));
			Ok(())
		}

		pub fn is_kitty_owner(
			kitty_id: T::KittyIndex,
			acct: &T::AccountId,
		) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(<Error<T>>::NotOwner),
			}
		}

		pub fn transfer_kitty_to(
			kitty_id: T::KittyIndex,
			to: T::AccountId,
		) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::InvalidKittyIndex)?;
			let prev_owner = kitty.owner.clone();

			KittiesOwner::<T>::try_mutate(&prev_owner, |owned| {
				if let Some(index) = owned.iter().position(|&id| id == kitty_id) {
					owned.swap_remove(index);
					return Ok(())
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::InvalidKittyIndex)?;

			kitty.owner = to.clone();
			kitty.price = None;

			<Kitties<T>>::insert(kitty_id, Some(kitty));

			KittiesOwner::<T>::try_mutate(&to, |vec| vec.try_push(kitty_id))
				.map_err(|_| <Error<T>>::ExceedKittyOwned)?;

			Self::deposit_event(Event::KittyTransferred(prev_owner, to, kitty_id));

			Ok(())
		}
	}
}
