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

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

  use frame_support::dispatch::{DispatchError};
  use frame_support::sp_runtime::print;
  use frame_support::sp_std::vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]

		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			//     let wat = r#"
			//     (module
			//   (import "" "" (func $host_hello (param i32)))

			//   (func (export "hello")
			// i32.const 3
			// call $host_hello)
			//     )
			// "#;
			//     let wasm = wat::parse_str(&wat)
			//   .map_err(|_| { DispatchError::Other("Contract parse failed") })?;
			let wasm = vec![0u8, 0u8, 0u8, 0u8];

			let engine = wasmi::Engine::default();
			let module = wasmi::Module::new(&engine, &wasm[..])
				.map_err(|_| { DispatchError::Other("Contract engine setup failed") })?;

			// All wasm objects operate within the context of a "store". Each
			// `Store` has a type parameter to store host-specific data, which in
			// this case we're using `42` for.
			type HostState = u32;
			let mut store = wasmi::Store::new(&engine, 42);

			let host_hello = wasmi::Func::wrap(&mut store, |caller: wasmi::Caller<'_, HostState>, param: i32| {
				print("Got returned value from WebAssembly");
				print(param as u32);
				print("My host state is:");
				print(caller.host_data());
			});

			// In order to create Wasm module instances and link their imports
			// and exports we require a `Linker`.
			let mut linker = <wasmi::Linker<HostState>>::new();
			linker.define("host", "hello", host_hello)
				.map_err(|_| { DispatchError::Other("Linker setup failed") })?;
			let instance = linker
				.instantiate(&mut store, &module)
				.map_err(|_| { DispatchError::Other("Linker instantiate failed") })?
				.start(&mut store)
				.map_err(|_| { DispatchError::Other("Linker start failed") })?;
			let hello = instance
				.get_export(&store, "hello")
				.and_then(wasmi::Extern::into_func)
				.ok_or_else(|| DispatchError::Other("could not find function \"hello\""))?
				.typed::<(), (), _>(&mut store)
				.map_err(|_| { DispatchError::Other("Instance function setup failed") })?;

			// And finally we can call the wasm as if it were a Rust function!
			hello.call(&mut store, ())
  .map_err(|_trap| DispatchError::Other("Trapped"))?;

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
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
	}
}
