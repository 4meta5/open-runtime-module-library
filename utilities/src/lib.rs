#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::storage::{with_transaction, TransactionOutcome};
use sp_runtime::DispatchError;
use sp_std::result::Result;

pub mod ordered_set;

pub use ordered_set::OrderedSet;

/// Execute the supplied function in a new storage transaction.
///
/// All changes to storage performed by the supplied function are discarded if
/// the returned outcome is `Result::Err`.
///
/// Transactions can be nested to any depth. Commits happen to the parent
/// transaction.
pub fn with_transaction_result<R>(f: impl FnOnce() -> Result<R, DispatchError>) -> Result<R, DispatchError> {
	with_transaction(|| {
		let res = f();
		if res.is_ok() {
			TransactionOutcome::Commit(res)
		} else {
			TransactionOutcome::Rollback(res)
		}
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{assert_err, assert_ok, decl_module, decl_storage};
	use sp_io::TestExternalities;
	use sp_runtime::{DispatchError, DispatchResult};

	pub trait Trait: frame_system::Trait {}

	decl_module! {
		pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
	}

	decl_storage! {
		trait Store for Module<T: Trait> as StorageTransactions {
			pub Value: u32;
			pub Map: map hasher(twox_64_concat) String => u32;
		}
	}

	#[test]
	fn storage_transaction_basic_commit() {
		TestExternalities::default().execute_with(|| {
			assert_eq!(Value::get(), 0);
			assert!(!Map::contains_key("val0"));

			assert_ok!(with_transaction_result(|| -> DispatchResult {
				Value::set(99);
				Map::insert("val0", 99);
				assert_eq!(Value::get(), 99);
				assert_eq!(Map::get("val0"), 99);
				Ok(())
			}));

			assert_eq!(Value::get(), 99);
			assert_eq!(Map::get("val0"), 99);
		});
	}

	#[test]
	fn storage_transaction_basic_rollback() {
		TestExternalities::default().execute_with(|| {
			assert_eq!(Value::get(), 0);
			assert_eq!(Map::get("val0"), 0);

			assert_err!(
				with_transaction_result(|| -> DispatchResult {
					Value::set(99);
					Map::insert("val0", 99);
					assert_eq!(Value::get(), 99);
					assert_eq!(Map::get("val0"), 99);
					Err("test".into())
				}),
				DispatchError::Other("test")
			);

			assert_eq!(Value::get(), 0);
			assert_eq!(Map::get("val0"), 0);
		});
	}
}
