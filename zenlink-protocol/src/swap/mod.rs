// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use codec::{Decode, Encode};
use sp_core::U256;
use sp_runtime::{
    traits::{AccountIdConversion, IntegerSquareRoot, One, Zero},
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::convert::TryInto;

use crate::{
    ensure,
    primitives::{AssetProperty, LpProperty, MultiAsset},
    sp_api_hidden_includes_decl_storage::hidden_include::{StorageMap, StorageValue},
    vec, AssetId, Assets, AssetsToPair, Config, Error, Get, Module, NextPairId, Pairs,
    TokenBalance, Vec,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
pub struct Pair<AccountId, TokenBalance> {
    pub token_0: AssetId,
    pub token_1: AssetId,

    pub account: AccountId,
    pub total_liquidity: TokenBalance,
    pub lp_asset_id: AssetId,
}

impl<T: Config> Module<T> {
    pub(crate) fn inner_create_pair(token_0: &AssetId, token_1: &AssetId) -> DispatchResult {
        ensure!(token_0 != token_1, Error::<T>::DeniedCreatePair);
        ensure!(
            Self::get_pair_from_asset_id(token_0, token_1).is_none(),
            Error::<T>::PairAlreadyExists
        );
        let pair_id = Self::next_pair_id();
        let next_id = pair_id.checked_add(One::one()).ok_or(Error::<T>::Overflow)?;

        let account: T::AccountId = <T as Config>::ModuleId::get().into_sub_account(pair_id);
        sp_std::if_std! { println!("zenlink::<inner_create_pair> {:#?} pair_id {:#?}",account, pair_id );}
        let (token_0, token_1) = Self::sort_asset_id(*token_0, *token_1);
        let lp_asset_index = <Assets>::get().len() as u32;
        let lp_asset_id = AssetId {
            chain_id: T::ParaId::get().into(),
            module_index: Self::index(),
            asset_index: lp_asset_index,
        };
        let new_pair =
            Pair { token_0, token_1, account, total_liquidity: Zero::zero(), lp_asset_id };
        <AssetsToPair<T>>::insert((token_0, token_1), new_pair);
        <Pairs>::mutate(|list| list.push((token_0, token_1)));
        <NextPairId>::put(next_id);

        let asset_property = AssetProperty::Lp(LpProperty { token_0, token_1 });
        Self::inner_issue(lp_asset_id, asset_property)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn inner_add_liquidity(
        who: &T::AccountId,
        token_0: &AssetId,
        token_1: &AssetId,
        amount_0_desired: TokenBalance,
        amount_1_desired: TokenBalance,
        amount_0_min: TokenBalance,
        amount_1_min: TokenBalance,
    ) -> DispatchResult {
        let mut pair =
            Self::get_pair_from_asset_id(token_0, token_1).ok_or(Error::<T>::PairNotExists)?;
        let reserve_0 = Self::multi_asset_balance_of(&token_0, &pair.account);
        let reserve_1 = Self::multi_asset_balance_of(&token_1, &pair.account);

        let (amount_0, amount_1) = Self::calculate_added_amount(
            amount_0_desired,
            amount_1_desired,
            amount_0_min,
            amount_1_min,
            reserve_0,
            reserve_1,
        )?;

        let balance_token_0 = Self::multi_asset_balance_of(&token_0, &who);
        let balance_token_1 = Self::multi_asset_balance_of(&token_1, &who);
        ensure!(
            balance_token_0 >= amount_0 && balance_token_1 >= amount_1,
            Error::<T>::InsufficientAssetBalance
        );

        let mint_liquidity =
            Self::mint_liquidity(amount_0, amount_1, reserve_0, reserve_1, pair.total_liquidity);
        ensure!(mint_liquidity > Zero::zero(), Error::<T>::Overflow);

        pair.total_liquidity =
            pair.total_liquidity.checked_add(mint_liquidity).ok_or(Error::<T>::Overflow)?;

        Self::multi_asset_deposit(&pair.lp_asset_id, who, mint_liquidity)?;
        Self::multi_asset_transfer(&token_0, &who, &pair.account, amount_0)?;
        Self::multi_asset_transfer(&token_1, &who, &pair.account, amount_1)?;

        <AssetsToPair<T>>::insert((pair.token_0, pair.token_1), pair);

        Ok(())
    }

    pub(crate) fn mint_liquidity(
        amount_0: TokenBalance,
        amount_1: TokenBalance,
        reserve_0: TokenBalance,
        reserve_1: TokenBalance,
        total_liquidity: TokenBalance,
    ) -> TokenBalance {
        if total_liquidity == Zero::zero() {
            amount_0.saturating_mul(amount_1).integer_sqrt()
        } else {
            core::cmp::min(
                Self::calculate_share_amount(amount_0, reserve_0, total_liquidity),
                Self::calculate_share_amount(amount_1, reserve_1, total_liquidity),
            )
        }
    }

    pub(crate) fn calculate_added_amount(
        amount_0_desired: TokenBalance,
        amount_1_desired: TokenBalance,
        amount_0_min: TokenBalance,
        amount_1_min: TokenBalance,
        reserve_0: TokenBalance,
        reserve_1: TokenBalance,
    ) -> Result<(TokenBalance, TokenBalance), DispatchError> {
        if reserve_0 == Zero::zero() || reserve_1 == Zero::zero() {
            return Ok((amount_0_desired, amount_1_desired));
        }
        let amount_1_optimal = Self::calculate_share_amount(amount_0_desired, reserve_0, reserve_1);
        if amount_1_optimal <= amount_1_desired {
            ensure!(amount_1_optimal >= amount_1_min, Error::<T>::IncorrectAssetAmountRange);
            return Ok((amount_0_desired, amount_1_optimal));
        }
        let amount_0_optimal = Self::calculate_share_amount(amount_1_desired, reserve_1, reserve_0);
        ensure!(
            amount_0_optimal >= amount_0_min && amount_0_optimal <= amount_0_desired,
            Error::<T>::IncorrectAssetAmountRange
        );
        Ok((amount_0_optimal, amount_1_desired))
    }

    pub(crate) fn inner_remove_liquidity(
        who: &T::AccountId,
        token_0: &AssetId,
        token_1: &AssetId,
        remove_liquidity: TokenBalance,
        amount_token_0_min: TokenBalance,
        amount_token_1_min: TokenBalance,
        to: &T::AccountId,
    ) -> DispatchResult {
        let pair =
            Self::get_pair_from_asset_id(token_0, token_1).ok_or(Error::<T>::PairNotExists)?;
        let liquidity = Self::multi_asset_balance_of(&pair.lp_asset_id, who);
        ensure!(liquidity >= remove_liquidity, Error::<T>::InsufficientLiquidity);

        let reserve_0 = Self::multi_asset_balance_of(&token_0, &pair.account);
        let reserve_1 = Self::multi_asset_balance_of(&token_1, &pair.account);

        let amount_0 =
            Self::calculate_share_amount(remove_liquidity, pair.total_liquidity, reserve_0);
        let amount_1 =
            Self::calculate_share_amount(remove_liquidity, pair.total_liquidity, reserve_1);

        ensure!(
            amount_0 >= amount_token_0_min && amount_1 >= amount_token_1_min,
            Error::<T>::InsufficientTargetAmount
        );

        Self::multi_asset_transfer(&token_0, &pair.account, &to, amount_0)?;
        Self::multi_asset_transfer(&token_1, &pair.account, &to, amount_1)?;

        Self::multi_asset_withdraw(&pair.lp_asset_id, who, remove_liquidity)?;

        <AssetsToPair<T>>::mutate((pair.token_0, pair.token_1), |option_pair| {
            if let Some(pair) = option_pair {
                pair.total_liquidity = pair.total_liquidity.saturating_sub(remove_liquidity);
            }
        });

        Ok(())
    }

    pub(crate) fn inner_swap_exact_tokens_for_tokens(
        who: &T::AccountId,
        amount_in: TokenBalance,
        amount_out_min: TokenBalance,
        path: &[AssetId],
        to: &T::AccountId,
    ) -> DispatchResult {
        let amounts = Self::get_amount_out_by_path(amount_in, &path)?;
        ensure!(amounts[amounts.len() - 1] >= amount_out_min, Error::<T>::InsufficientTargetAmount);

        let pair =
            Self::get_pair_from_asset_id(&path[0], &path[1]).ok_or(Error::<T>::PairNotExists)?;

        Self::multi_asset_transfer(&path[0], &who, &pair.account, amount_in)?;
        Self::swap(&amounts, &path, &to)?;

        Ok(())
    }

    pub fn inner_swap_tokens_for_exact_tokens(
        who: &T::AccountId,
        amount_out: TokenBalance,
        amount_in_max: TokenBalance,
        path: &[AssetId],
        to: &T::AccountId,
    ) -> DispatchResult {
        let amounts = Self::get_amount_in_by_path(amount_out, &path)?;

        ensure!(amounts[0] <= amount_in_max, Error::<T>::ExcessiveSoldAmount);
        let pair =
            Self::get_pair_from_asset_id(&path[0], &path[1]).ok_or(Error::<T>::PairNotExists)?;

        Self::multi_asset_transfer(&path[0], &who, &pair.account, amounts[0])?;
        Self::swap(&amounts, &path, &to)?;

        Ok(())
    }

    pub(crate) fn get_amount_out_by_path(
        amount_in: TokenBalance,
        path: &[AssetId],
    ) -> Result<Vec<TokenBalance>, DispatchError> {
        ensure!(path.len() > 1, Error::<T>::InvalidPath);

        let len = path.len() - 1;
        let mut out_vec = vec![amount_in];

        for i in 0..len {
            if let Some(pair) = Self::get_pair_from_asset_id(&path[i], &path[i + 1]) {
                let reserve_0 = Self::multi_asset_balance_of(&path[i], &pair.account);
                let reserve_1 = Self::multi_asset_balance_of(&path[i + 1], &pair.account);
                ensure!(
                    reserve_1 > Zero::zero() && reserve_0 > Zero::zero(),
                    Error::<T>::InvalidPath
                );

                let amount = Self::get_amount_out(out_vec[i], reserve_0, reserve_1);
                ensure!(amount > Zero::zero(), Error::<T>::InvalidPath);
                out_vec.push(amount);
            } else {
                return Err(Error::<T>::PairNotExists.into());
            }
        }
        Ok(out_vec)
    }

    pub(crate) fn get_amount_in_by_path(
        amount_out: TokenBalance,
        path: &[AssetId],
    ) -> Result<Vec<TokenBalance>, DispatchError> {
        let len = path.len();
        ensure!(len > 1, Error::<T>::InvalidPath);

        let mut tvec = vec![amount_out];
        let mut i = len - 1;

        while i > 0 {
            if let Some(pair) = Self::get_pair_from_asset_id(&path[i], &path[i - 1]) {
                let reserve_0 = Self::multi_asset_balance_of(&path[i], &pair.account);
                let reserve_1 = Self::multi_asset_balance_of(&path[i - 1], &pair.account);
                ensure!(
                    reserve_1 > Zero::zero() && reserve_0 > Zero::zero(),
                    Error::<T>::InvalidPath
                );

                let amount = Self::get_amount_in(tvec[len - 1 - i], reserve_1, reserve_0);
                ensure!(amount > One::one(), Error::<T>::InvalidPath);

                tvec.push(amount);
                i -= 1;
            } else {
                return Err(Error::<T>::PairNotExists.into());
            }
        }
        tvec.reverse();
        Ok(tvec)
    }

    fn calculate_share_amount(
        amount_0: TokenBalance,
        reserve_0: TokenBalance,
        reserve_1: TokenBalance,
    ) -> TokenBalance {
        U256::from(amount_0)
            .saturating_mul(U256::from(reserve_1))
            .checked_div(U256::from(reserve_0))
            .and_then(|n| TryInto::<TokenBalance>::try_into(n).ok())
            .unwrap_or_else(Zero::zero)
    }

    fn get_amount_in(
        output_amount: TokenBalance,
        input_reserve: TokenBalance,
        output_reserve: TokenBalance,
    ) -> TokenBalance {
        let numerator = U256::from(input_reserve)
            .saturating_mul(U256::from(output_amount))
            .saturating_mul(U256::from(1000));

        let denominator = (U256::from(output_reserve).saturating_sub(U256::from(output_amount)))
            .saturating_mul(U256::from(997));

        numerator
            .checked_div(denominator)
            .and_then(|r| r.checked_add(U256::one()))
            .and_then(|n| TryInto::<TokenBalance>::try_into(n).ok())
            .unwrap_or_else(Zero::zero)
    }

    fn get_amount_out(
        input_amount: TokenBalance,
        input_reserve: TokenBalance,
        output_reserve: TokenBalance,
    ) -> TokenBalance {
        let input_amount_with_fee = U256::from(input_amount).saturating_mul(U256::from(997));

        let numerator = input_amount_with_fee.saturating_mul(U256::from(output_reserve));

        let denominator = U256::from(input_reserve)
            .saturating_mul(U256::from(1000))
            .saturating_add(input_amount_with_fee);

        numerator
            .checked_div(denominator)
            .and_then(|n| TryInto::<TokenBalance>::try_into(n).ok())
            .unwrap_or_else(Zero::zero)
    }

    fn sort_asset_id(token_0: AssetId, token_1: AssetId) -> (AssetId, AssetId) {
        if token_0 < token_1 {
            (token_0, token_1)
        } else {
            (token_1, token_0)
        }
    }

    fn swap(amounts: &[TokenBalance], path: &[AssetId], to: &T::AccountId) -> DispatchResult {
        for i in 0..(amounts.len() - 1) {
            let input = path[i];
            let output = path[i + 1];
            let (token0, _) = Self::sort_asset_id(input, output);
            let mut amount0_out: TokenBalance = TokenBalance::default();
            let mut amount1_out = amounts[i + 1];
            if input != token0 {
                amount0_out = amounts[i + 1];
                amount1_out = TokenBalance::default();
            }
            let pair =
                Self::get_pair_from_asset_id(&input, &output).ok_or(Error::<T>::PairNotExists)?;

            if i < (amounts.len() - 2) {
                let mid_account = Self::get_pair_from_asset_id(&output, &path[i + 2])
                    .ok_or(Error::<T>::PairNotExists)?;
                Self::pair_swap(&pair, amount0_out, amount1_out, &mid_account.account)?;
            } else {
                Self::pair_swap(&pair, amount0_out, amount1_out, &to)?;
            };
        }
        Ok(())
    }

    fn pair_swap(
        pair: &Pair<T::AccountId, TokenBalance>,
        amount_0: TokenBalance,
        amount_1: TokenBalance,
        to: &T::AccountId,
    ) -> DispatchResult {
        let reserve_0 = Self::multi_asset_balance_of(&pair.token_0, &pair.account);
        let reserve_1 = Self::multi_asset_balance_of(&pair.token_1, &pair.account);
        ensure!(
            amount_0 <= reserve_0 && amount_1 <= reserve_1,
            Error::<T>::InsufficientPairReserve
        );

        if amount_0 > Zero::zero() {
            Self::multi_asset_transfer(&pair.token_0, &pair.account, to, amount_0)?;
        }
        if amount_1 > Zero::zero() {
            Self::multi_asset_transfer(&pair.token_1, &pair.account, to, amount_1)?;
        }

        Ok(())
    }

    pub(crate) fn get_pair_from_asset_id(
        token_0: &AssetId,
        token_1: &AssetId,
    ) -> Option<Pair<T::AccountId, TokenBalance>> {
        Self::tokens_to_pair((token_0, token_1))
            .or_else(|| Self::tokens_to_pair((token_1, token_0)))
    }
}
