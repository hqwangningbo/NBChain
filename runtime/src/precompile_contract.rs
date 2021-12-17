#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, ExistenceRequirement};
use core::marker::PhantomData;
use fp_evm::{PrecompileOutput, Context, ExitError, ExitSucceed};
use pallet_evm::{Precompile, AddressMapping};
use sp_core::{H160, U256, hexdisplay::HexDisplay};
use sp_runtime::{traits::UniqueSaturatedInto, AccountId32};
use codec::{Encode, Decode};
use frame_support::log;
// use pallet_coming_id::ComingNFT;
use sp_std::vec;

pub struct PrecompileTest<T: pallet_evm::Config> {
    _marker: PhantomData<T>,
}


impl<T: pallet_evm::Config> PrecompileTest<T>
{
    fn process(
        input: &[u8]
    ) -> Result<bool, ExitError> {
        match input.len() {
            // withdraw balance
            // input = from(evm address, 20 bytes) + to(substrate pubkey, 32 bytes) + value(32 bytes)
            84 => {
                log::debug!(target: "pallet-erc20", "withdraw balance: call");

                Self::process_withdraw_balance(input)
                    .map_err(|err| {
                        log::warn!(target: "pallet-erc20", "withdraw balance: err = {:?}", err);
                        err
                    })?;

                log::debug!(target: "pallet-erc20", "withdraw balance: success");

                Ok(true)
            }
            _ => {
                log::warn!(target: "pallet-erc20", "invalid input: {:?}", input);

                Err(ExitError::Other("invalid input".into()))
            }
        }
    }

    fn account_from_address(
        address: &[u8]
    ) -> Result<T::AccountId, ExitError> {
        frame_support::ensure!(address.len() == 20, ExitError::Other("invalid address".into()));

        let from = H160::from_slice(&address[0..20]);

        Ok(T::AddressMapping::into_account_id(from))
    }

    fn account_from_pubkey(
        pubkey: &[u8]
    ) -> Result<T::AccountId, ExitError> {
        frame_support::ensure!(pubkey.len() == 32, ExitError::Other("invalid pubkey".into()));

        let mut target = [0u8; 32];
        target[0..32].copy_from_slice(&pubkey[0..32]);

        T::AccountId::decode(&mut &AccountId32::new(target).encode()[..])
            .map_err(|_| ExitError::Other("decode AccountId32 failed".into()))
    }

    fn balance(value: &[u8]) -> Result<u128, ExitError> {
        frame_support::ensure!(value.len() == 32, ExitError::Other("invalid balance".into()));

        Ok(U256::from_big_endian(&value[0..32]).low_u128())
    }


    fn process_withdraw_balance(
        input: &[u8]
    ) -> Result<(), ExitError> {
        let from = Self::account_from_address(&input[0..20])?;
        let to = Self::account_from_pubkey(&input[20..52])?;
        let balance = Self::balance(&input[52..84])?;

        log::debug!(target: "pallet-erc20", "from(evm): {:?}", H160::from_slice(&input[0..20]));
        log::debug!(target: "pallet-erc20", "from(sub): {:?}", HexDisplay::from(&from.encode()));
        log::debug!(target: "pallet-erc20", "to(sub): {:?}", HexDisplay::from(&to.encode()));
        log::debug!(target: "pallet-erc20", "value(sub): {:?}", balance);

        T::Currency::transfer(
            &from,
            &to,
            balance.unique_saturated_into(),
            ExistenceRequirement::AllowDeath,
        ).map_err(|err| {
            ExitError::Other(sp_std::borrow::Cow::Borrowed(err.into()))
        })
    }
}

impl<T> Precompile for PrecompileTest<T>
    where
        T: pallet_evm::Config,
        T::AccountId: Decode,
{
    fn execute(
        input: &[u8],
        _target_gas: Option<u64>,
        context: &Context,
    ) -> Result<PrecompileOutput, ExitError> {
        log::debug!(target: "pallet-erc20", "caller: {:?}", context.caller);
        log::debug!(target: "pallet-erc20", "address: {:?}", context.address);

        const BASE_GAS_COST: u64 = 45_000;

        // Refer: https://github.com/rust-ethereum/ethabi/blob/master/ethabi/src/encoder.rs#L144
        let mut out = vec![0u8; 32];

        if Self::process(input)? {
            out[31] = 1u8;
        }

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: BASE_GAS_COST,
            output: out.to_vec(),
            logs: Default::default(),
        })
    }
}
