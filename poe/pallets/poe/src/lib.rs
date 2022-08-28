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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		#[pallet::constant]
		type MaxClaimLength :Get<u32> ;
		  
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>; 
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	pub type Proofs<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8,T::MaxClaimLength>,
		(T::AccountId,T::BlockNumber),
	>;



	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		ClaimCreated( T::AccountId,Vec<u8>),
		ClaimRevoked( T::AccountId,Vec<u8>),
		TransferClaim(T::AccountId,Vec<u8>,T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	}

	#[pallet::hooks]
	impl<T:Config>  Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8,T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;
			ensure!(!Proofs::<T>::contains_key(&bounded_claim),Error::<T>::ProofAlreadyExist);

			Proofs::<T>::insert(
				&bounded_claim, 
				(&sender,frame_system::Pallet::<T>::block_number())
			);
			Self::deposit_event(Event::ClaimCreated(sender,claim));

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8,T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;
			let (owner,_) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(owner==sender,Error::<T>::NotClaimOwner);
			
			Proofs::<T>::remove(
				&bounded_claim, 
			);
			Self::deposit_event(Event::ClaimRevoked(sender,claim));

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_claim(origin: OriginFor<T>, claim: Vec<u8>, dest: T::AccountId) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8,T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;
			let (owner,_) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(owner==sender,Error::<T>::NotClaimOwner);
			
			Proofs::<T>::insert(
				&bounded_claim, 
				(&dest,frame_system::Pallet::<T>::block_number())
			);
			Self::deposit_event(Event::TransferClaim(sender,claim,dest));
			Ok(().into())
		}
	}
}
