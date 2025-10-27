#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use codec::alloc::string::String;
use scale_info::prelude::format;
use scale_info::prelude::vec::Vec;
use sp_core::H160;
use sp_io::hashing::keccak_256;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
    // Import various useful types required by all FRAME pallets.
    use super::*;
    use frame_support::{
        pallet_prelude::{OptionQuery, *},
        Blake2_128Concat, BoundedVec,
    };
    use frame_system::pallet_prelude::*;

    // The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
    // (`Call`s) in this pallet.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The pallet's configuration trait.
    ///
    /// All our types and constants a pallet depends on must be declared here.
    /// These types are defined generically and made concrete when the pallet is declared in the
    /// `runtime/src/lib.rs` file of your chain.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxUsernameLength: Get<u32>;
    }

    /// A storage item for this pallet.
    ///
    /// In this template, we are declaring a storage item called `Something` that stores a single
    /// `u32` value. Learn more about runtime storage here: <https://docs.substrate.io/build/runtime-storage/>
    #[pallet::storage]
    pub type Something<T> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn nonces)]
    pub type Nonces<T: Config> = StorageMap<_, Blake2_128Concat, H160, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn usernames)]
    pub type Usernames<T: Config> =
        StorageMap<_, Blake2_128Concat, H160, BoundedVec<u8, T::MaxUsernameLength>, OptionQuery>;
    //
    /// Events that functions in this pallet can emit.
    ///
    /// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
    /// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
    /// documentation for each event field and its parameters is added to a node's metadata so it
    /// can be used by external interfaces or tools.
    ///
    ///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
    /// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
    /// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        UsernameSet {
            eth_address: H160,
            username: BoundedVec<u8, T::MaxUsernameLength>,
        },
    }

    /// Errors that can be returned by this pallet.
    ///
    /// Errors tell users that something went wrong so it's important that their naming is
    /// informative. Similar to events, error documentation is added to a node's metadata so it's
    /// equally important that they have helpful documentation associated with them.
    ///
    /// This type of runtime error can be up to 4 bytes in size should you want to return additional
    /// information.
    #[pallet::error]
    pub enum Error<T> {
        UsernameTooLong,
        InvalidUsername,
        InvalidNonce,
        InvalidEthereumSignature,
    }

    /// The pallet's dispatchable functions ([`Call`]s).
    ///
    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// They must always return a `DispatchResult` and be annotated with a weight and call index.
    ///
    /// The [`call_index`] macro is used to explicitly
    /// define an index for calls in the [`Call`] enum. This is useful for pallets that may
    /// introduce new dispatchables over time. If the order of a dispatchable changes, its index
    /// will also change which will break backwards compatibility.
    ///
    /// The [`weight`] macro is used to assign a weight to each call.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a single u32 value as a parameter, writes the value
        /// to storage and emits an event.
        ///
        /// It checks that the _origin_ for this call is _Signed_ and returns a dispatch
        /// error if it isn't. Learn more about origins here: <https://docs.substrate.io/build/origins/>
        #[pallet::call_index(0)]
        #[pallet::weight(1000)]
        pub fn set_username(
            origin: OriginFor<T>,
            eth_address: H160,
            username: Vec<u8>,
            nonce: u64,
            eth_signature: Vec<u8>,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let _ = ensure_signed(origin)?;

            let expected_nonce = Nonces::<T>::get(&eth_address);
            ensure!(nonce == expected_nonce, Error::<T>::InvalidNonce);

            let bounded_username: BoundedVec<u8, T::MaxUsernameLength> = username
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::UsernameTooLong)?;

            ensure!(
                bounded_username
                    .iter()
                    .all(|&c| c.is_ascii_alphanumeric() || c == b'_'),
                Error::<T>::InvalidUsername
            );

            let message = format!(
                "set_username:{}:{}",
                String::from_utf8_lossy(&username.clone()),
                &nonce
            );

            ensure!(
                Self::verify_ethereum_signature(&eth_address, message.as_bytes(), &eth_signature),
                Error::<T>::InvalidEthereumSignature
            );

            // Store
            Nonces::<T>::insert(&eth_address, nonce + 1);
            Usernames::<T>::insert(&eth_address, bounded_username.clone());

            Self::deposit_event(Event::UsernameSet {
                eth_address,
                username: bounded_username,
            });

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn get_nonce(eth_address: H160) -> u64 {
        Nonces::<T>::get(eth_address)
    }

    pub fn get_username(eth_address: H160) -> Option<Vec<u8>> {
        Usernames::<T>::get(eth_address).map(|b| b.into_inner())
    }

    pub fn verify_ethereum_signature(eth_address: &H160, message: &[u8], signature: &[u8]) -> bool {
        // TODO, make 65, 30, 27 as constants
        if signature.len() != 65 {
            return false;
        }

        let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
        let mut eth_message = prefix.as_bytes().to_vec();
        eth_message.extend_from_slice(message);

        let hash = keccak_256(&eth_message);

        let mut sig_array = [0u8; 65];
        sig_array.copy_from_slice(&signature[..65]);

        if sig_array[64] < 27 || sig_array[64] > 30 {
            return false;
        }
        sig_array[64] -= 27;

        match sp_io::crypto::secp256k1_ecdsa_recover(&sig_array, &hash) {
            Ok(pubkey) => {
                let recovered_addr_hash = keccak_256(&pubkey[1..]);
                let recovered_eth_addr = H160::from_slice(&recovered_addr_hash[12..32]);
                &recovered_eth_addr == eth_address
            }
            Err(_) => false,
        }
    }
}
