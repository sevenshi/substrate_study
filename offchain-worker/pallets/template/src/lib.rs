#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use sp_runtime::{
    offchain::{
        storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
        http, Duration,
    },
};

use frame_system::{
    offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction,
        Signer,
    },
};
use sp_core::crypto::KeyTypeId;
use serde::{Deserialize, Deserializer};


pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");
pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct OcwAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for OcwAuthId
        {
            type RuntimeAppPublic = Public;
            type GenericSignature = sp_core::sr25519::Signature;
            type GenericPublic = sp_core::sr25519::Public;
        }
} 



#[frame_support::pallet]
pub mod pallet {
    use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_std::collections::vec_deque::VecDeque;

    // {"data":{"id":"polkadot","rank":"12","symbol":"DOT","name":"Polkadot","supply":"1153614196.4216700000000000",
    // "maxSupply":null,"marketCapUsd":"7215090302.1136182467874062","volumeUsd24Hr":"188404673.7548428400951199",
    // "priceUsd":"6.2543355694596121","changePercent24Hr":"-1.2555371265218473","vwap24Hr":"6.2500029323747878",
    // "explorer":"https://polkascan.io/polkadot"},"timestamp":1663823391031}

    #[derive(Deserialize, Encode, Decode)]
    struct PolkadotVo {
        data: Polkadot,
    }

    #[derive(Deserialize, Encode, Decode)]
    struct Polkadot {
        #[serde(deserialize_with = "de_string_to_bytes")]
        id: Vec<u8>,
        #[serde(deserialize_with = "de_string_to_bytes")]
        symbol: Vec<u8>,
        #[serde(deserialize_with = "de_string_to_bytes")]
        name: Vec<u8>,
        // supply:f32,
        // marketCapUsd:f32,
        // volumeUsd24Hr:f32,
        #[serde(deserialize_with = "de_string_to_bytes")]
        priceUsd:Vec<u8>,
        // changePercent24Hr:f32,
        // vwap24Hr:f32,
        #[serde(deserialize_with = "de_string_to_bytes")]
        explorer: Vec<u8>,
    }


    pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
        where
        D: Deserializer<'de>,
        {
            let s: &str = Deserialize::deserialize(de)?;
            Ok(s.as_bytes().to_vec())
        }

    use core::{convert::TryInto, fmt};
    impl fmt::Debug for PolkadotVo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{{ Polkadot price: {},}}",
                // &self.timestamp,
                sp_std::str::from_utf8( &self.data.priceUsd).map_err(|_| fmt::Error)?,
                )
        }
    }


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

        #[pallet::weight(0)]
        pub fn submit_data(origin: OriginFor<T>, payload: Vec<u8>) -> DispatchResultWithPostInfo {

            let _who = ensure_signed(origin)?;

            log::info!("in submit_data call: {:?}", payload);

            Ok(().into())
        }
	}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

        fn offchain_worker(_: T::BlockNumber) {
            let key = b"node-template::storage::polkadot::price".to_vec();
            let val_ref = StorageValueRef::persistent(&key);
            let mut vec = VecDeque::with_capacity(10);

            if let Ok(Some(value)) = val_ref.get::<VecDeque<PolkadotVo>>() {
                // print values
                vec = value;
            }


            if let Ok(info) = Self::fetch_polkadot_price() {

                log::info!("fetch Polkdot price: {:?}", info);
                if  let  Some(_) =   vec.get(9) {
                    vec.pop_front();
                }
                vec.push_back(info);
            } else {
                log::info!("Error while fetch Polkdot price!");

            }

            #[derive(Debug)]
            struct StateError;

            //  write or mutate tuple content to key
            let res = val_ref.mutate(|val: Result<Option<VecDeque<PolkadotVo>>, StorageRetrievalError>| -> Result<_, StateError> {
                match val {
                    Ok(Some(_)) => Ok(vec),
                    _ => Ok(vec),
                }
            });

            log::info!("Price VecDeque: {:?}", res.as_ref().unwrap());


        }

        fn on_initialize(_n: T::BlockNumber) -> Weight {
            log::info!("in on_initialize!");
            0
        }

        fn on_finalize(_n: T::BlockNumber) {
            log::info!("in on_finalize!");
        }

        fn on_idle(_n: T::BlockNumber, _remaining_weight: Weight) -> Weight {
            log::info!("in on_idle!");
            0
        }

    }

    impl<T: Config> Pallet<T> {

        fn send_signed_tx(payload: Vec<u8>) -> Result<(), &'static str> {
            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            if !signer.can_sign() {
                return Err(
                    "No local accounts available. Consider adding one via `author_insertKey` RPC.",
                    )
            }

            let results = signer.send_signed_transaction(|_account| {

                Call::submit_data { payload: payload.clone() }
            });

            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Submitted data:{:?}", acc.id, payload),
                    Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                }
            }

            Ok(())
        }

        fn fetch_polkadot_price() -> Result<PolkadotVo, http::Error> {
            // prepare for send request
            let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
            let request =
                http::Request::get("https://api.coincap.io/v2/assets/polkadot");
            let pending = request
                .add_header("User-Agent", "Substrate-Offchain-Worker")
                .deadline(deadline).send().map_err(|_| http::Error::IoError)?;
            let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
            if response.code != 200 {
                log::warn!("Unexpected status code: {}", response.code);
                return Err(http::Error::Unknown)
            }
            let body = response.body().collect::<Vec<u8>>();
            let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
                log::warn!("No UTF8 body");
                http::Error::Unknown
            })?;
            // parse the response str
            let gh_info: PolkadotVo =
                serde_json::from_str(body_str).map_err(|_| {
                    log::warn!("error parse PolkadotVo");
                    http::Error::Unknown
                })?;

            Ok(gh_info)
        }

    }

}
