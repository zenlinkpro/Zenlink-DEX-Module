// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use cumulus_primitives_core::ParaId;
use frame_support::{
    sp_runtime::traits::Zero,
    traits::{Currency, ExistenceRequirement, ExistenceRequirement::KeepAlive, WithdrawReasons},
};
use sp_runtime::{sp_std::convert::TryFrom, SaturatedConversion};

use crate::{
    ensure,
    primitives::{AssetId, MultiAsset, NATIVE_CURRENCY_MODULE_INDEX},
    sp_api_hidden_includes_decl_storage::hidden_include::{traits::Get, StorageMap, StorageValue},
    AssetProperty, Assets, AssetsProperty, Balances, Config, DispatchResult, Error, Module,
    OperationalAsset,
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
        <AssetsProperty>::insert(id, property);
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
        } else if <T as Config>::ParaId::get() != ParaId::from(asset_id.chain_id) {
            Zero::zero()
        } else if asset_id.module_index == NATIVE_CURRENCY_MODULE_INDEX {
            <T as Config>::NativeCurrency::total_issuance().saturated_into::<TokenBalance>()
        } else if asset_id.module_index == <T as Config>::OperationalAsset::module_index() {
            <T as Config>::OperationalAsset::total_supply(asset_id.asset_index)
        } else {
            Zero::zero()
        }
    }

    fn multi_asset_balance_of(asset_id: &AssetId, who: &T::AccountId) -> TokenBalance {
        if Self::assets_list().contains(asset_id) {
            Self::balance_of(*asset_id, who)
        } else if <T as Config>::ParaId::get() != ParaId::from(asset_id.chain_id) {
            Zero::zero()
        } else if NATIVE_CURRENCY_MODULE_INDEX == asset_id.module_index {
            <T as Config>::NativeCurrency::free_balance(&who).saturated_into::<TokenBalance>()
        } else if <T as Config>::OperationalAsset::module_index() == asset_id.module_index {
            <T as Config>::OperationalAsset::balance(asset_id.asset_index, (*who).clone())
        } else {
            Zero::zero()
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
            ensure!(
                <T as Config>::ParaId::get() == ParaId::from(asset_id.chain_id),
                Error::<T>::AssetNotExists
            );
            if NATIVE_CURRENCY_MODULE_INDEX == asset_id.module_index {
                let amount = <<<T as Config>::NativeCurrency as Currency<
                    <T as frame_system::Config>::AccountId,
                >>::Balance as TryFrom<u128>>::try_from(amount)
                .map_err(|_| Error::<T>::Overflow)?;
                <T as Config>::NativeCurrency::transfer(&from, &to, amount, KeepAlive)
            } else if <T as Config>::OperationalAsset::module_index() == asset_id.module_index {
                <T as Config>::OperationalAsset::inner_transfer(
                    asset_id.asset_index,
                    (*from).clone(),
                    (*to).clone(),
                    amount,
                )
            } else {
                Err((Error::<T>::AssetNotExists).into())
            }
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
            ensure!(
                <T as Config>::ParaId::get() == ParaId::from(asset_id.chain_id),
                Error::<T>::AssetNotExists
            );
            if NATIVE_CURRENCY_MODULE_INDEX == asset_id.module_index {
                sp_std::if_std! { println!("zenlink::<multi_asset_withdraw>"); }
                let amount = <<<T as Config>::NativeCurrency as Currency<
                    <T as frame_system::Config>::AccountId,
                >>::Balance as TryFrom<u128>>::try_from(amount)
                .map_err(|_| Error::<T>::Overflow)?;
                <T as Config>::NativeCurrency::withdraw(
                    &who,
                    amount,
                    WithdrawReasons::TRANSFER,
                    ExistenceRequirement::AllowDeath,
                )
                .map_or_else(Err, |_| Ok(()))
            } else if <T as Config>::OperationalAsset::module_index() == asset_id.module_index {
                sp_std::if_std! { println!("zenlink::<multi_asset_withdraw> withdraw op_asset model _id "); }
                <T as Config>::OperationalAsset::inner_withdraw(
                    asset_id.asset_index,
                    (*who).clone(),
                    amount,
                )
            } else {
                Err((Error::<T>::AssetNotExists).into())
            }
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
            sp_std::if_std! { println!("zenlink::<multi_asset_deposit> deposit in zenlink module {:#?}", asset_id); }
            Self::inner_issue(*asset_id, AssetProperty::Foreign)?;
            Self::inner_mint(*asset_id, &who, amount)
        } else if asset_id.module_index == NATIVE_CURRENCY_MODULE_INDEX {
            sp_std::if_std! { println!("zenlink::<multi_asset_deposit> deposit native model_id {:#?}", NATIVE_CURRENCY_MODULE_INDEX); }
            let amount = <<<T as Config>::NativeCurrency as Currency<
                <T as frame_system::Config>::AccountId,
            >>::Balance as TryFrom<u128>>::try_from(amount)
            .map_err(|_| Error::<T>::Overflow)?;
            <T as Config>::NativeCurrency::deposit_creating(&who, amount);
            Ok(())
        } else if asset_id.module_index == <T as Config>::OperationalAsset::module_index() {
            sp_std::if_std! { println!("zenlink::<multi_asset_deposit> deposit op_asset{:#?}", asset_id); }
            <T as Config>::OperationalAsset::inner_deposit(
                asset_id.asset_index,
                (*who).clone(),
                amount,
            )
        } else {
            Err((Error::<T>::AssetNotExists).into())
        }
    }
}
