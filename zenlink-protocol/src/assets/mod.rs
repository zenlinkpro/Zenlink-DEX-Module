// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use cumulus_primitives_core::ParaId;
use frame_support::sp_runtime::traits::Zero;

use crate::{
    ensure,
    primitives::{AssetId, MultiAsset},
    sp_api_hidden_includes_decl_storage::hidden_include::{traits::Get, StorageMap, StorageValue},
    AssetProperty, Assets, AssetsMetadata, Balances, Config, DispatchResult, Error, Module,
    RawEvent::{Burned, Issued, Minted, Transferred},
    TokenBalance, TotalSupply, Vec,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// The main implementation block for the module.
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

// Zenlink module control the other asset module. It only has this one entrance.
impl<T: Config> MultiAsset<T::AccountId, TokenBalance> for Module<T> {
    fn multi_asset_total_supply(asset_id: &AssetId) -> TokenBalance {
        if Self::assets_list().contains(asset_id) {
            Self::total_supply(*asset_id)
        } else {
            T::AssetModuleRegistry::get()
                .iter()
                .find(|(index, _)| {
                    *index == asset_id.module_index
                        && <T as Config>::ParaId::get() == ParaId::from(asset_id.chain_id)
                })
                .map_or(Zero::zero(), |(_, t)| t.total_supply(asset_id.asset_index))
        }
    }

    fn multi_asset_balance_of(asset_id: &AssetId, who: &T::AccountId) -> TokenBalance {
        if Self::assets_list().contains(asset_id) {
            Self::balance_of(*asset_id, who)
        } else {
            T::AssetModuleRegistry::get()
                .iter()
                .find(|(index, _)| {
                    *index == asset_id.module_index
                        && <T as Config>::ParaId::get() == ParaId::from(asset_id.chain_id)
                })
                .map_or(Zero::zero(), |(_, t)| t.balance(asset_id.asset_index, who.clone()))
        }
    }

    fn multi_asset_transfer(
        asset_id: &AssetId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: TokenBalance,
    ) -> DispatchResult {
        if Self::assets_list().contains(asset_id) {
            Self::inner_transfer(*asset_id, &from, &to, amount)
        } else {
            T::AssetModuleRegistry::get()
                .iter()
                .find(|(index, _)| {
                    *index == asset_id.module_index
                        && <T as Config>::ParaId::get() == ParaId::from(asset_id.chain_id)
                })
                .map_or(Err((Error::<T>::AssetNotExists).into()), |(_, t)| {
                    t.inner_transfer(asset_id.asset_index, from.clone(), to.clone(), amount)
                })
        }
    }

    fn multi_asset_withdraw(
        asset_id: &AssetId,
        who: &T::AccountId,
        amount: TokenBalance,
    ) -> DispatchResult {
        if Self::assets_list().contains(asset_id) {
            Self::inner_burn(*asset_id, &who, amount)
        } else {
            T::AssetModuleRegistry::get()
                .iter()
                .find(|(index, _)| {
                    *index == asset_id.module_index
                        && <T as Config>::ParaId::get() == ParaId::from(asset_id.chain_id)
                })
                .map_or(Err((Error::<T>::AssetNotExists).into()), |(_, t)| {
                    t.inner_withdraw(asset_id.asset_index, who.clone(), amount)
                })
        }
    }

    fn multi_asset_deposit(
        asset_id: &AssetId,
        who: &T::AccountId,
        amount: TokenBalance,
    ) -> DispatchResult {
        if Self::assets_list().contains(asset_id) {
            Self::inner_mint(*asset_id, &who, amount)
        } else if <T as Config>::ParaId::get() != ParaId::from(asset_id.chain_id) {
            Self::inner_issue(*asset_id, AssetProperty::Foreign)?;
            Self::inner_mint(*asset_id, &who, amount)
        } else {
            T::AssetModuleRegistry::get()
                .iter()
                .find(|(index, _)| *index == asset_id.module_index)
                .map_or(Err((Error::<T>::AssetNotExists).into()), |(_, t)| {
                    t.inner_deposit(asset_id.asset_index, who.clone(), amount)
                })
        }
    }
}
