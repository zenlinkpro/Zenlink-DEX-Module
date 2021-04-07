// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use frame_support::traits::{
    Currency, ExistenceRequirement, ExistenceRequirement::KeepAlive, Get, WithdrawReasons,
};
use sp_runtime::{DispatchError, DispatchResult, SaturatedConversion};
use sp_std::marker::PhantomData;

use super::{AssetId, Config, TokenBalance, TryInto, NATIVE_CURRENCY};

pub trait MultiAssetHandler<AccountId> {
    fn multi_asset_total_supply(asset_id: &AssetId) -> TokenBalance;

    fn multi_asset_balance_of(asset_id: &AssetId, who: &AccountId) -> TokenBalance;

    fn multi_asset_transfer(
        asset_id: &AssetId,
        from: &AccountId,
        to: &AccountId,
        amount: TokenBalance,
    ) -> DispatchResult {
        Self::multi_asset_withdraw(asset_id, from, amount)?;
        Self::multi_asset_deposit(asset_id, to, amount)?;

        Ok(())
    }

    fn multi_asset_withdraw(
        asset_id: &AssetId,
        who: &AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError>;

    fn multi_asset_deposit(
        asset_id: &AssetId,
        who: &AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError>;
}

impl<AccountId> MultiAssetHandler<AccountId> for ()
where
    TokenBalance: Copy,
{
    fn multi_asset_total_supply(_asset_id: &AssetId) -> TokenBalance {
        unimplemented!()
    }

    fn multi_asset_balance_of(_asset_id: &AssetId, _who: &AccountId) -> TokenBalance {
        unimplemented!()
    }

    fn multi_asset_withdraw(
        _asset_id: &AssetId,
        _who: &AccountId,
        _amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        unimplemented!()
    }

    fn multi_asset_deposit(
        _asset_id: &AssetId,
        _who: &AccountId,
        _amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        unimplemented!()
    }
}

pub trait AssetHandler<AccountId> {
    fn a_parachain_id() -> u32;

    fn a_module_index() -> u8;

    fn a_is_manageable(asset_id: AssetId) -> bool {
        let chain_id = asset_id.chain_id;
        let module_index = asset_id.module_index;
        let registered_chain_id = Self::a_parachain_id();
        let registered_module_index = Self::a_module_index();

        chain_id == registered_chain_id && module_index == registered_module_index
    }

    fn a_balance_of(asset_id: AssetId, who: AccountId) -> TokenBalance;

    fn a_total_supply(asset_id: AssetId) -> TokenBalance;

    fn a_transfer(
        asset_id: AssetId,
        origin: AccountId,
        target: AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        let withdrawn = Self::a_withdraw(asset_id, origin, amount)?;
        Self::a_deposit(asset_id, target, amount)?;

        Ok(withdrawn)
    }

    fn a_deposit(asset_id: AssetId, origin: AccountId, amount: TokenBalance) -> DispatchResult;

    fn a_withdraw(
        asset_id: AssetId,
        origin: AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError>;
}

impl<AccountId> AssetHandler<AccountId> for () {
    fn a_parachain_id() -> u32 {
        unimplemented!()
    }

    fn a_module_index() -> u8 {
        unimplemented!()
    }

    fn a_is_manageable(_asset_id: AssetId) -> bool {
        false
    }

    fn a_balance_of(_asset_id: AssetId, _who: AccountId) -> TokenBalance {
        unimplemented!()
    }

    fn a_total_supply(_asset_id: AssetId) -> TokenBalance {
        unimplemented!()
    }

    fn a_deposit(_asset_id: AssetId, _origin: AccountId, _amount: TokenBalance) -> DispatchResult {
        unimplemented!()
    }

    fn a_withdraw(
        _asset_id: AssetId,
        _origin: AccountId,
        _amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        unimplemented!()
    }
}

pub struct NativeCurrencyAdaptor<T, NativeCurrency>(PhantomData<(T, NativeCurrency)>);

impl<T, NativeCurrency> AssetHandler<T::AccountId> for NativeCurrencyAdaptor<T, NativeCurrency>
where
    T: Config,
    NativeCurrency: Currency<T::AccountId>,
{
    fn a_parachain_id() -> u32 {
        <T as Config>::ParaId::get().into()
    }

    fn a_module_index() -> u8 {
        NATIVE_CURRENCY
    }

    fn a_balance_of(_asset_id: AssetId, who: T::AccountId) -> TokenBalance {
        NativeCurrency::free_balance(&who).saturated_into::<TokenBalance>()
    }

    fn a_total_supply(_asset_id: AssetId) -> TokenBalance {
        NativeCurrency::total_issuance().saturated_into::<TokenBalance>()
    }

    fn a_transfer(
        _asset_id: AssetId,
        origin: T::AccountId,
        target: T::AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        let balance_amount = amount
            .try_into()
            .map_err(|_| DispatchError::Other("AmountToBalanceConversionFailed"))?;

        NativeCurrency::transfer(&origin, &target, balance_amount, KeepAlive)?;

        Ok(amount)
    }

    fn a_deposit(_asset_id: AssetId, origin: T::AccountId, amount: TokenBalance) -> DispatchResult {
        let balance_amount = amount
            .try_into()
            .map_err(|_| DispatchError::Other("AmountToBalanceConversionFailed"))?;

        NativeCurrency::deposit_creating(&origin, balance_amount);

        Ok(())
    }

    fn a_withdraw(
        _asset_id: AssetId,
        origin: T::AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        let balance_amount = amount
            .try_into()
            .map_err(|_| DispatchError::Other("AmountToBalanceConversionFailed"))?;

        NativeCurrency::withdraw(
            &origin,
            balance_amount,
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::AllowDeath,
        )?;

        Ok(amount)
    }
}
