use codec::{Decode, Encode};
pub use cumulus_primitives::ParaId;
use frame_support::{
    ensure,
    traits::{ExistenceRequirement::KeepAlive, Get, Vec},
};
use sp_core::U256;

use sp_runtime::{
    DispatchResult,
    RuntimeDebug, SaturatedConversion, traits::{AccountIdConversion, IntegerSquareRoot, One, Zero},
};
use sp_std::{convert::{TryFrom, TryInto}};
pub use xcm_executor::{
    Config as XcmCfg, traits::{FilterAssetLocation, LocationConversion, TransactAsset},
    XcmExecutor,
};

use crate::{
    AssetId, AssetsToPair, Config, Error, ExecuteXcm,
    LiquidityPool, Module, NextPairId, Pairs,
    primitives::{CrossChainOperation, OperateAsset},
    sp_api_hidden_includes_decl_storage::hidden_include::{
    StorageMap, StorageValue, traits::Currency},
    TokenBalance, Xcm
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
}

impl<T: Config> Module<T> {
    pub(crate) fn inner_create_pair(token_0: &AssetId, token_1: &AssetId) -> DispatchResult {
        ensure!( token_0 != token_1, Error::<T>::DeniedCreatePair );
        ensure!( Self::get_pair_from_asset_id(token_0, token_1).is_none(), Error::<T>::PairAlreadyExists );
        let pair_id = Self::next_pair_id();
        let next_id = pair_id.checked_add(One::one()).ok_or(Error::<T>::Overflow)?;

        let account: T::AccountId = <T as Config>::ModuleId::get().into_sub_account(pair_id);
        let (token_0, token_1) = Self::sort_asset_id(*token_0, *token_1);
        let new_pair = Pair {
            token_0,
            token_1,
            account,
            total_liquidity: Zero::zero(),
        };
        <AssetsToPair<T>>::insert((token_0, token_1), new_pair);
        <Pairs>::mutate(|list| list.push((token_0, token_1)));
        <NextPairId>::put(next_id);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn inner_add_liquidity_local(
        who: &T::AccountId,
        token_0: &AssetId,
        token_1: &AssetId,
        amount_0_desired: TokenBalance,
        amount_1_desired: TokenBalance,
        amount_0_min: TokenBalance,
        amount_1_min: TokenBalance,
    ) -> DispatchResult {
        let mut pair = Self::get_pair_from_asset_id(token_0, token_1).ok_or(Error::<T>::PairNotExists)?;
        let reserve_0 = Self::asset_balance_of(&token_0, &pair.account);
        let reserve_1 = Self::asset_balance_of(&token_1, &pair.account);

        let (amount_0, amount_1) = Self::calculate_added_amount(
            amount_0_desired,
            amount_1_desired,
            amount_0_min,
            amount_1_min,
            reserve_0,
            reserve_1,
        )?;

        let balance_token_0 = Self::asset_balance_of(&token_0, &who);
        let balance_token_1 = Self::asset_balance_of(&token_1, &who);
        ensure!( balance_token_0 >= amount_0 && balance_token_1 >= amount_1, Error::<T>::InsufficientAssetBalance );

        let mint_liquidity = Self::mint_liquidity(amount_0, amount_1, reserve_0, reserve_1, pair.total_liquidity);
        ensure!( mint_liquidity > Zero::zero(), Error::<T>::Overflow );

        pair.total_liquidity = pair.total_liquidity
            .checked_add(mint_liquidity)
            .ok_or(Error::<T>::Overflow)?;

        Self::asset_transfer(&token_0, &who, &pair.account, amount_0)?;
        Self::asset_transfer(&token_1, &who, &pair.account, amount_1)?;

        <LiquidityPool<T>>::mutate((pair.clone().account, who), |balance| { *balance = balance.saturating_add(mint_liquidity); });
        <AssetsToPair<T>>::insert((pair.token_0, pair.token_1), pair);

        Ok(())
    }

    fn mint_liquidity(
        amount_0: TokenBalance,
        amount_1: TokenBalance,
        reserve_0: TokenBalance,
        reserve_1: TokenBalance,
        total_liquidity: TokenBalance,
    ) -> TokenBalance {
        if total_liquidity == Zero::zero() {
            amount_0.saturating_mul(amount_1).integer_sqrt()
        } else {
            core::cmp::min(Self::calculate_share_amount(amount_0, reserve_0, total_liquidity),
                           Self::calculate_share_amount(amount_1, reserve_1, total_liquidity))
        }
    }

    pub(crate) fn inner_add_liquidity_foreign(
        who: &T::AccountId,
        token_0: &AssetId,
        token_1: &AssetId,
        amount_0_desired: TokenBalance,
        amount_1_desired: TokenBalance,
        target_parachain: ParaId,
    ) -> DispatchResult {
        let local_para_id = T::ParaId::get();
        ensure!( target_parachain != local_para_id, Error::<T>::DeniedSwapInLocal );
        ensure!( *token_0 == AssetId::NativeCurrency, Error::<T>::DeniedAddLiquidityToParachain );

        let parachain_asset = match token_1 {
            AssetId::NativeCurrency => local_para_id.into(),
            AssetId::ParaCurrency(parachain_asset) => *parachain_asset,
        };

        let add_liquidity_operate_encode = CrossChainOperation::AddLiquidity {
            origin_chain: local_para_id.into(),
            target_chain: target_parachain.into(),
            token_0: local_para_id.into(),
            token_1: parachain_asset,
            amount_1: amount_1_desired.saturated_into::<TokenBalance>(),
        }.encode();

        let xcm = Self::make_xcm_by_cross_chain_operate(
            target_parachain.into(),
            &who,
            amount_0_desired,
            &add_liquidity_operate_encode);

        Self::inner_execute_xcm(who, xcm)?;
        Ok(())
    }

    fn calculate_added_amount(
        amount_0_desired: TokenBalance,
        amount_1_desired: TokenBalance,
        amount_0_min: TokenBalance,
        amount_1_min: TokenBalance,
        reserve_0: TokenBalance,
        reserve_1: TokenBalance,
    ) -> Result<(TokenBalance, TokenBalance), sp_runtime::DispatchError> {
        if reserve_0 == Zero::zero() || reserve_1 == Zero::zero() {
            Ok((amount_0_desired, amount_1_desired))
        } else {
            let amount_1_optimal = Self::calculate_share_amount(amount_0_desired, reserve_0, reserve_1);
            if amount_1_optimal <= amount_1_desired {
                ensure!(amount_1_optimal >= amount_1_min, Error::<T>::IncorrectAssetAmountRange);
                Ok((amount_0_desired, amount_1_optimal))
            } else {
                let amount_0_optimal = Self::calculate_share_amount(amount_1_desired, reserve_1, reserve_0);
                ensure!( amount_0_optimal >= amount_0_min && amount_0_optimal <= amount_0_desired, Error::<T>::IncorrectAssetAmountRange );
                Ok((amount_0_optimal, amount_1_desired))
            }
        }
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
        let pair = Self::get_pair_from_asset_id(token_0, token_1).ok_or(Error::<T>::PairNotExists)?;
        let liquidity = <LiquidityPool<T>>::get((pair.account.clone(), who));
        ensure!( liquidity >= remove_liquidity, Error::<T>::InsufficientLiquidity );

        let reserve_0 = Self::asset_balance_of(&token_0, &pair.account);
        let reserve_1 = Self::asset_balance_of(&token_1, &pair.account);

        let amount_0 = Self::calculate_share_amount(reserve_0, remove_liquidity, pair.total_liquidity);
        let amount_1 = Self::calculate_share_amount(reserve_1, remove_liquidity, pair.total_liquidity);

        ensure!( amount_0 >= amount_token_0_min && amount_1 >= amount_token_1_min, Error::<T>::InsufficientTargetAmount);

        Self::asset_transfer(&token_0, &pair.account, &to, amount_0)?;
        Self::asset_transfer(&token_1, &pair.account, &to, amount_1)?;

        <LiquidityPool<T>>::mutate((pair.account, who), |balance| {
            *balance = (*balance).saturating_sub(remove_liquidity);
        });

        <AssetsToPair<T>>::mutate((pair.token_0, pair.token_1), |option_pair| {
            if let Some(pair) = option_pair {
                pair.total_liquidity = pair.total_liquidity.saturating_sub(remove_liquidity);
            }
        });

        Self::restore_parachain_asset(&to, amount_0, token_0)?;
        Self::restore_parachain_asset(&to, amount_1, token_1)?;

        Ok(())
    }

    pub(crate) fn inner_swap_exact_tokens_for_tokens_foreign(
        who: &T::AccountId,
        amount_in: TokenBalance,
        amount_out_min: TokenBalance,
        path: &[AssetId],
        target_parachain: ParaId,
    ) -> DispatchResult {
        let local_para_id = T::ParaId::get();
        ensure!( target_parachain != local_para_id, Error::<T>::DeniedSwapInLocal );
        let asset_id_path = path
            .iter()
            .map(|id| {
                match *id {
                    AssetId::NativeCurrency => local_para_id.into(),
                    AssetId::ParaCurrency(currency_id) => currency_id,
                }
            })
            .collect();

        let swap_exact_tokens_for_tokens_encode = CrossChainOperation::SwapExactTokensForTokens {
            origin_chain: local_para_id.into(),
            target_chain: target_parachain.into(),
            amount_out_min: amount_out_min.saturated_into::<TokenBalance>(),
            path: asset_id_path,
        }.encode();

        let xcm = Self::make_xcm_by_cross_chain_operate(
            target_parachain.into(),
            &who,
            amount_in,
            &swap_exact_tokens_for_tokens_encode);
        Self::inner_execute_xcm(who, xcm)?;

        Ok(())
    }

    pub(crate) fn inner_swap_exact_tokens_for_tokens_local(
        who: &T::AccountId,
        amount_in: TokenBalance,
        amount_out_min: TokenBalance,
        path: &[AssetId],
        to: &T::AccountId,
    ) -> DispatchResult {
        let amounts = Self::get_amount_out_by_path(amount_in, &path)?;
        ensure!( amounts[amounts.len() - 1] >= amount_out_min, Error::<T>::InsufficientTargetAmount );

        let pair = Self::get_pair_from_asset_id(&path[0], &path[1])
            .ok_or(Error::<T>::PairNotExists)?;
        let path_last = path.last().ok_or(Error::<T>::InvalidPath)?;

        let target_asset_balance_reserve = Self::asset_balance_of(path_last, who);

        Self::asset_transfer(&path[0], &who, &pair.account, amount_in)?;
        Self::swap(&amounts, &path, &to)?;

        let target_asset_reserve_balance = Self::asset_balance_of(path_last, who).saturating_sub(target_asset_balance_reserve);
        Self::restore_parachain_asset(&who, target_asset_reserve_balance, path_last)?;

        Ok(())
    }

    pub fn inner_swap_tokens_for_exact_tokens_foreign(
        who: &T::AccountId,
        amount_out: TokenBalance,
        amount_in_max: TokenBalance,
        path: &[AssetId],
        target_parachain: ParaId,
    ) -> DispatchResult {
        let local_para_id = T::ParaId::get();
        ensure!( target_parachain != local_para_id, Error::<T>::DeniedSwapInLocal );
        let asset_id_path = path
            .iter()
            .map(|id| {
                match *id {
                    AssetId::NativeCurrency => local_para_id.into(),
                    AssetId::ParaCurrency(currency_id) => currency_id,
                }
            })
            .collect();

        let swap_tokens_for_exact_tokens_encode = CrossChainOperation::SwapTokensForExactTokens {
            origin_chain: local_para_id.into(),
            target_chain: target_parachain.into(),
            path: asset_id_path,
            amount_out: amount_out.saturated_into::<TokenBalance>(),
        }.encode();

        let xcm = Self::make_xcm_by_cross_chain_operate(
            target_parachain.into(),
            &who,
            amount_in_max,
            &swap_tokens_for_exact_tokens_encode);

        Self::inner_execute_xcm(who, xcm)?;

        Ok(())
    }

    pub fn inner_swap_tokens_for_exact_tokens_local(
        who: &T::AccountId,
        amount_out: TokenBalance,
        amount_in_max: TokenBalance,
        path: &[AssetId],
        to: &T::AccountId,
    ) -> DispatchResult {
        let amounts = Self::get_amount_in_by_path(amount_out, &path)?;

        ensure!( amounts[0] <= amount_in_max, Error::<T>::ExcessiveSoldAmount );
        let pair = Self::get_pair_from_asset_id(&path[0], &path[1]).ok_or(Error::<T>::PairNotExists)?;

        let path_target = path.last().ok_or(Error::<T>::InvalidPath)?;
        let path_first = path.first().ok_or(Error::<T>::InvalidPath)?;
        let target_asset_balance_reserve = Self::asset_balance_of(path_target, who);

        Self::asset_transfer(&path[0], &who, &pair.account, amounts[0])?;
        Self::swap(&amounts, &path, &to)?;

        let target_asset_reserve_balance = Self::asset_balance_of(path_target, who).saturating_sub(target_asset_balance_reserve);
        let supply_asset_reserve_balance = amount_in_max.saturating_sub(amounts[0]);

        Self::restore_parachain_asset(&who, target_asset_reserve_balance, path_target)?;
        Self::restore_parachain_asset(&who, supply_asset_reserve_balance, path_first)?;

        Ok(())
    }

    pub(crate) fn get_amount_out_by_path(
        amount_in: TokenBalance,
        path: &[AssetId],
    ) -> Result<Vec<TokenBalance>, sp_runtime::DispatchError> {
        ensure!( path.len() > 1, Error::<T>::InvalidPath );

        let len = path.len() - 1;
        let mut out_vec = Vec::new();
        out_vec.push(amount_in);

        for i in 0..len {
            if let Some(pair) = Self::get_pair_from_asset_id(&path[i], &path[i + 1]) {
                let reserve_0 = Self::asset_balance_of(&path[i], &pair.account);
                let reserve_1 = Self::asset_balance_of(&path[i + 1], &pair.account);
                ensure!( reserve_1 > Zero::zero() && reserve_0 > Zero::zero(), Error::<T>::InvalidPath );

                let amount = Self::get_amount_out(out_vec[i], reserve_0, reserve_1);
                ensure!( amount > Zero::zero(), Error::<T>::InvalidPath );
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
    ) -> Result<Vec<TokenBalance>, sp_runtime::DispatchError> {
        let len = path.len();
        ensure!( len > 1, Error::<T>::InvalidPath );

        let mut tvec = Vec::new();
        tvec.push(amount_out);
        let mut i = len - 1;
        while i > 0 {
            if let Some(pair) = Self::get_pair_from_asset_id(&path[i], &path[i - 1]) {
                let reserve_0 = Self::asset_balance_of(&path[i], &pair.account);
                let reserve_1 = Self::asset_balance_of(&path[i - 1], &pair.account);
                ensure!( reserve_1 > Zero::zero() && reserve_0 > Zero::zero(), Error::<T>::InvalidPath );

                let amount = Self::get_amount_in(tvec[len - 1 - i], reserve_0, reserve_1);
                ensure!( amount > One::one(), Error::<T>::InvalidPath );

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
        amount: TokenBalance,
        reserve_0: TokenBalance,
        reserve_1: TokenBalance,
    ) -> TokenBalance {
        U256::from(amount)
            .saturating_mul(U256::from(reserve_0))
            .checked_div(U256::from(reserve_1))
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

        let denominator = (U256::from(output_reserve)
            .saturating_sub(U256::from(output_amount)))
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
        let input_amount_with_fee = U256::from(input_amount)
            .saturating_mul(U256::from(997));

        let numerator = input_amount_with_fee
            .saturating_mul(U256::from(output_reserve));

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

    fn swap(
        amounts: &[TokenBalance],
        path: &[AssetId],
        to: &T::AccountId,
    ) -> DispatchResult {
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
            let pair = Self::get_pair_from_asset_id(&input, &output)
                .ok_or(Error::<T>::PairNotExists)?;
            if i < (amounts.len() - 2) {
                let mid_account = Self::get_pair_from_asset_id(&output, &path[i + 2])
                    .ok_or(Error::<T>::PairNotExists)?;
                Self::pair_swap(
                    &pair,
                    amount0_out,
                    amount1_out,
                    &mid_account.account,
                )?;
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
        let reserve_0 = Self::asset_balance_of(&pair.token_0, &pair.account);
        let reserve_1 = Self::asset_balance_of(&pair.token_1, &pair.account);
        ensure!( amount_0 <= reserve_0 && amount_1 <= reserve_1, Error::<T>::InsufficientPairReserve);

        if amount_0 > Zero::zero() {
            Self::asset_transfer(&pair.token_0, &pair.account, to, amount_0)?;
        }
        if amount_1 > Zero::zero() {
            Self::asset_transfer(&pair.token_1, &pair.account, to, amount_1)?;
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

    fn asset_transfer(
        token: &AssetId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: TokenBalance,
    ) -> DispatchResult {
        match token {
            AssetId::NativeCurrency => {
                let amount = <<<T as Config>::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance as TryFrom<u128>>::try_from(amount).map_err(|_| {
                    Error::<T>::Overflow
                })?;
                <T as Config>::NativeCurrency::transfer(&from, &to, amount, KeepAlive)
            }
            AssetId::ParaCurrency(_) => Self::inner_transfer(*token, &from, &to, amount),
        }
    }

    pub fn asset_balance_of(token: &AssetId, owner: &T::AccountId) -> TokenBalance {
        match token {
            AssetId::NativeCurrency => {
                <T as Config>::NativeCurrency::free_balance(&owner)
                    .saturated_into::<TokenBalance>()
            },
            AssetId::ParaCurrency(_) => Self::balance_of(*token, &owner),
        }
    }

    fn inner_execute_xcm(
        who: &T::AccountId,
        xcm: Xcm,
    ) -> DispatchResult {
        let xcm_origin = T::AccountIdConverter::try_into_location(who.clone())
            .map_err(|_| Error::<T>::AccountIdBadLocation)?;

        T::XcmExecutor::execute_xcm(xcm_origin, xcm)
            .map_err(|_| Error::<T>::ExecutionFailed)?;
        Ok(())
    }
}

impl<T: Config> OperateAsset<T::AccountId, TokenBalance> for Module<T> {
    fn add_liquidity(
        who: &T::AccountId,
        token_0: &AssetId,
        token_1: &AssetId,
        amount_0_desired: TokenBalance,
        amount_1_desired: TokenBalance,
    ) -> DispatchResult {
        let pair = Self::get_pair_from_asset_id(token_0, token_1);
        if pair.is_none() {
            Self::inner_create_pair(token_0, token_1)?;
        }
        Self::inner_add_liquidity_local(
            who,
            token_0,
            token_1,
            amount_0_desired,
            amount_1_desired,
            Zero::zero(),
            Zero::zero(),
        )?;

        let reserve_0 = Self::asset_balance_of(&token_0, &who);
        let reserve_1 = Self::asset_balance_of(&token_1, &who);

        Self::restore_parachain_asset(&who, reserve_0, &token_0)?;
        Self::restore_parachain_asset(&who, reserve_1, &token_1)?;

        Ok(())
    }

    fn swap_in(
        who: &T::AccountId,
        amount_in: TokenBalance,
        amount_out_min: TokenBalance,
        path: &[AssetId],
    ) -> DispatchResult {
        Self::inner_swap_exact_tokens_for_tokens_local(who, amount_in, amount_out_min, path, who)?;

        Ok(())
    }

    fn swap_out(
        who: &T::AccountId,
        amount_out: TokenBalance,
        amount_in_max: TokenBalance,
        path: &[AssetId],
    ) -> DispatchResult {

        Self::inner_swap_tokens_for_exact_tokens_local(who, amount_out, amount_in_max, path, who)?;

        Ok(())
    }

    fn restore_parachain_asset(
        who: &T::AccountId,
        amount: TokenBalance,
        asset_id: &AssetId,
    ) -> DispatchResult {
        if amount == Zero::zero() {
            return Ok(());
        }
        if let AssetId::ParaCurrency(id) = *asset_id {
            let xcm = Self::make_xcm_transfer_to_parachain(
                asset_id,
                id.into(),
                who,
                amount,
            );
            Self::inner_execute_xcm(who, xcm)?;
        }
        Ok(())
    }
}
