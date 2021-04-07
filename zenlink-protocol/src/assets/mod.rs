// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use crate::{
    ensure,
    sp_api_hidden_includes_decl_storage::hidden_include::{traits::Get, StorageMap, StorageValue},
    AssetHandler, AssetId, AssetProperty, Assets, AssetsMetadata, Balances, Config, DispatchError,
    DispatchResult, Error, Module, MultiAssetHandler,
    RawEvent::{Burned, Issued, Minted, Transferred},
    TokenBalance, TotalSupply, Vec, INNER_ASSET,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// TODO: refactor storage

// The Zenlink Protocol inner asset which store other chain assets and liquidity tokens
impl<T: Config> Module<T> {
    /// public mutable functions

    /// Implement of the transfer function.
    pub(crate) fn inner_transfer(
        id: AssetId,
        owner: &T::AccountId,
        target: &T::AccountId,
        amount: TokenBalance,
    ) -> DispatchResult {
        let owner_balance = <Balances<T>>::get((&id, owner));
        ensure!(owner_balance >= amount, Error::<T>::InsufficientAssetBalance);

        let new_balance = owner_balance.saturating_sub(amount);

        <Balances<T>>::mutate((id, owner), |balance| *balance = new_balance);
        <Balances<T>>::mutate((id, target), |balance| *balance = balance.saturating_add(amount));

        Self::deposit_event(Transferred(id, owner.clone(), target.clone(), amount));

        Ok(())
    }

    pub(crate) fn inner_issue(id: AssetId, property: AssetProperty) -> DispatchResult {
        ensure!(!<Assets>::get().contains(&id), Error::<T>::AssetAlreadyExist);
        <Assets>::mutate(|list| list.push(id));
        <AssetsMetadata>::insert(id, property);
        Self::deposit_event(Issued(id));
        Ok(())
    }

    /// Increase the total supply of the asset
    pub(crate) fn inner_mint(
        id: AssetId,
        owner: &T::AccountId,
        amount: TokenBalance,
    ) -> DispatchResult {
        ensure!(<Assets>::get().contains(&id), Error::<T>::AssetNotExists);

        let new_balance = <Balances<T>>::get((id, owner)).saturating_add(amount);

        <Balances<T>>::mutate((id, owner), |balance| *balance = new_balance);
        <TotalSupply>::mutate(id, |supply| {
            *supply = supply.saturating_add(amount);
        });

        Self::deposit_event(Minted(id, owner.clone(), amount));

        Ok(())
    }

    /// Decrease the total supply of the asset
    pub(crate) fn inner_burn(
        id: AssetId,
        owner: &T::AccountId,
        amount: TokenBalance,
    ) -> DispatchResult {
        ensure!(<Assets>::get().contains(&id), Error::<T>::AssetNotExists);
        let new_balance = <Balances<T>>::get((id, owner))
            .checked_sub(amount)
            .ok_or(Error::<T>::InsufficientAssetBalance)?;

        <Balances<T>>::mutate((id, owner), |balance| *balance = new_balance);
        <TotalSupply>::mutate(id, |supply| {
            *supply = supply.saturating_sub(amount);
        });

        Self::deposit_event(Burned(id, owner.clone(), amount));

        Ok(())
    }

    // Public immutable functions

    /// Get the asset `id` balance of `owner`.
    pub fn balance_of(id: AssetId, owner: &T::AccountId) -> TokenBalance {
        <Balances<T>>::get((id, owner))
    }

    /// Get the total supply of an asset `id`.
    pub fn total_supply(id: AssetId) -> TokenBalance {
        <TotalSupply>::get(id)
    }

    /// Get the assets list
    pub fn assets_list() -> Vec<AssetId> {
        <Assets>::get()
    }
}

// The AssetHandler implementation for Zenlink Protocol inner asset
impl<T: Config> AssetHandler<T::AccountId> for Module<T> {
    fn a_parachain_id() -> u32 {
        <T as Config>::ParaId::get().into()
    }

    fn a_module_index() -> u8 {
        INNER_ASSET
    }

    fn a_is_manageable(asset_id: AssetId) -> bool {
        Self::assets_list().contains(&asset_id)
    }

    fn a_balance_of(asset_id: AssetId, who: T::AccountId) -> TokenBalance {
        Self::balance_of(asset_id, &who)
    }

    fn a_total_supply(asset_id: AssetId) -> TokenBalance {
        Self::total_supply(asset_id)
    }

    fn a_transfer(
        asset_id: AssetId,
        origin: T::AccountId,
        target: T::AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        Self::inner_transfer(asset_id, &origin, &target, amount)?;

        Ok(amount)
    }

    fn a_deposit(asset_id: AssetId, origin: T::AccountId, amount: TokenBalance) -> DispatchResult {
        Self::inner_mint(asset_id, &origin, amount)
    }

    fn a_withdraw(
        asset_id: AssetId,
        origin: T::AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        Self::inner_burn(asset_id, &origin, amount)?;

        Ok(amount)
    }
}

// Zenlink module control the other asset module. It only has this one entrance.
impl<T: Config> MultiAssetHandler<T::AccountId> for Module<T> {
    fn multi_asset_total_supply(asset_id: &AssetId) -> TokenBalance {
        let aid = *asset_id;

        if Self::a_is_manageable(aid) {
            return Self::a_total_supply(aid);
        }

        if T::NativeCurrency::a_is_manageable(aid) {
            return T::NativeCurrency::a_total_supply(aid);
        }

        if T::OtherAssets::a_is_manageable(aid) {
            return T::OtherAssets::a_total_supply(aid);
        }

        Default::default()
    }

    fn multi_asset_balance_of(asset_id: &AssetId, who: &T::AccountId) -> TokenBalance {
        let aid = *asset_id;

        if Self::a_is_manageable(aid) {
            return Self::a_balance_of(aid, who.clone());
        }

        if T::NativeCurrency::a_is_manageable(aid) {
            return T::NativeCurrency::a_balance_of(*asset_id, who.clone());
        }

        if T::OtherAssets::a_is_manageable(aid) {
            return T::OtherAssets::a_balance_of(*asset_id, who.clone());
        }

        Default::default()
    }

    fn multi_asset_transfer(
        asset_id: &AssetId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: TokenBalance,
    ) -> DispatchResult {
        let aid = *asset_id;

        if Self::a_is_manageable(aid) {
            Self::a_transfer(aid, from.clone(), to.clone(), amount)?;
            return Ok(());
        }

        if T::NativeCurrency::a_is_manageable(aid) {
            T::NativeCurrency::a_transfer(aid, from.clone(), to.clone(), amount)?;
            return Ok(());
        }

        if T::OtherAssets::a_is_manageable(aid) {
            T::OtherAssets::a_transfer(aid, from.clone(), to.clone(), amount)?;
            return Ok(());
        }

        Err(Error::<T>::AssetNotExists.into())
    }

    fn multi_asset_withdraw(
        asset_id: &AssetId,
        who: &T::AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        let aid = *asset_id;

        if Self::a_is_manageable(aid) {
            Self::a_withdraw(aid, who.clone(), amount)?;
            return Ok(amount);
        }

        if T::NativeCurrency::a_is_manageable(*asset_id) {
            T::NativeCurrency::a_withdraw(aid, who.clone(), amount)?;
            return Ok(amount);
        }

        if T::OtherAssets::a_is_manageable(*asset_id) {
            T::OtherAssets::a_withdraw(aid, who.clone(), amount)?;
            return Ok(amount);
        }

        Err(Error::<T>::AssetNotExists.into())
    }

    fn multi_asset_deposit(
        asset_id: &AssetId,
        who: &T::AccountId,
        amount: TokenBalance,
    ) -> Result<TokenBalance, DispatchError> {
        let aid = *asset_id;

        if Self::a_is_manageable(aid) {
            Self::a_deposit(aid, who.clone(), amount)?;
            return Ok(amount);
        }

        if Self::a_parachain_id() != asset_id.chain_id {
            Self::inner_issue(aid, AssetProperty::Foreign)?;
            Self::a_deposit(aid, who.clone(), amount)?;
            return Ok(amount);
        }

        if T::NativeCurrency::a_is_manageable(aid) {
            T::NativeCurrency::a_deposit(aid, who.clone(), amount)?;
            return Ok(amount);
        }

        if T::OtherAssets::a_is_manageable(aid) {
            T::OtherAssets::a_deposit(aid, who.clone(), amount)?;
            return Ok(amount);
        }

        Err(Error::<T>::AssetNotExists.into())
    }
}
