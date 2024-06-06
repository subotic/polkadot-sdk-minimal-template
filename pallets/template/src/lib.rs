//! A shell pallet built with [`frame`].

#![cfg_attr(not(feature = "std"), no_std)]

// Re-export all pallet parts, this is needed to properly import the pallet into the runtime.
pub use pallet::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
    use frame::prelude::*;

    pub type Balance = u128;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        fn ed() -> Balance;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type TotalIssuance<T: Config> = StorageValue<Value = Balance>;

    #[pallet::storage]
    pub type Balances<T: Config> =
        StorageMap<Key = <T as frame_system::Config>::AccountId, Value = Balance>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An unsafe mint that can be called by anyone. Not a great idea.
        pub fn mint_unsafe(
            origin: T::RuntimeOrigin,
            dest: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            // ensure that this is a signed account, but we don't really check `_anyone`.
            let who = ensure_signed(origin)?;

            if amount < T::ed() {
                return Err("???".into());
            }

            // if who already has balance, we cannot mint again into it.
            if Balances::<T>::contains_key(who) {
                return Err("???".into());
            }

            // update the balances map. Notice how all `<T: Config>` remains as `<T>`.
            Balances::<T>::mutate(dest, |b| *b = Some(b.unwrap_or(0) + amount));
            // update total issuance.
            TotalIssuance::<T>::mutate(|t| *t = Some(t.unwrap_or(0) + amount));

            Ok(())
        }

        /// Transfer `amount` from `origin` to `dest`.
        pub fn transfer(
            origin: T::RuntimeOrigin,
            dest: T::AccountId,
            amount: Balance,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // ensure sender has enough balance, and if so, calculate what is left after `amount`.
            let sender_balance = Balances::<T>::get(&sender).ok_or("NonExistentAccount")?;
            let reminder = sender_balance.checked_sub(amount).ok_or("InsufficientBalance")?;

            // update sender and dest balances.
            Balances::<T>::mutate(dest, |b| *b = Some(b.unwrap_or(0) + amount));
            Balances::<T>::insert(&sender, reminder);

            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::pallet as currency_pallet;
    use super::*;
    use frame::testing_prelude::*;

    const ALICE: u64 = 1;
    const BOB: u64 = 2;
    const CHARLIE: u64 = 3;

    construct_runtime!(
        pub struct Runtime {
            System: frame_system,
            Currency: currency_pallet,
        }
    );

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl frame_system::Config for Runtime {
        type Block = MockBlock<Runtime>;
        // within pallet we just said `<T as frame_system::Config>::AccountId`, now we
        // finally specified it.
        type AccountId = u64;
    }

    // our simple pallet has nothing to be configured.
    impl currency_pallet::Config for Runtime {
        fn ed() -> crate::Balance {
            3
        }
    }

    #[test]
    fn first_test() {
        TestState::new_empty().execute_with(|| {
            // We expect Alice's account to have no funds.
            assert_eq!(Balances::<Runtime>::get(&ALICE), None);
            assert_eq!(TotalIssuance::<Runtime>::get(), None);

            // mint some funds into Alice's account.
            assert_ok!(Pallet::<Runtime>::mint_unsafe(RuntimeOrigin::signed(ALICE), ALICE, 100));

            // re-check the above
            assert_eq!(Balances::<Runtime>::get(&ALICE), Some(100));
            assert_eq!(TotalIssuance::<Runtime>::get(), Some(100));
        })
    }
}
