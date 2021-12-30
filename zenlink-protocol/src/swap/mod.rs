// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! # SWAP Module
//!
//! ## Overview
//!
//! Built-in decentralized exchange modules in Substrate network, the swap
//! mechanism refers to the design of Uniswap V2.

use super::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

impl<T: Config> Pallet<T> {
	pub(crate) fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}
	/// The account ID of a pair account
	/// only use two byte prefix to support 16 byte account id (used by test)
	/// "modl" ++ "/zenlink" is 12 bytes, and 4 bytes remaining for hash of AssetId pair.
	/// for AccountId32, 20 bytes remaining for hash of AssetId pair.
	pub fn pair_account_id(asset_0: AssetId, asset_1: AssetId) -> T::AccountId {
		let (asset_0, asset_1) = Self::sort_asset_id(asset_0, asset_1);
		let pair_hash: T::Hash = T::Hashing::hash_of(&(asset_0, asset_1));

		T::PalletId::get().into_sub_account(pair_hash.as_ref())
	}

	/// Sorted the foreign id of assets pair
	pub fn sort_asset_id(asset_0: AssetId, asset_1: AssetId) -> (AssetId, AssetId) {
		if asset_0 < asset_1 {
			(asset_0, asset_1)
		} else {
			(asset_1, asset_0)
		}
	}

	pub(crate) fn mutate_lp_pairs(asset_0: AssetId, asset_1: AssetId) {
		LiquidityPairs::<T>::insert(
			Self::sort_asset_id(asset_0, asset_1),
			Some(Self::lp_asset_id(&asset_0, &asset_1)),
		)
	}

	pub fn lp_asset_id(asset_0: &AssetId, asset_1: &AssetId) -> AssetId {
		let (asset_0, asset_1) = Self::sort_asset_id(*asset_0, *asset_1);
		let currency_0 = (asset_0.asset_index & 0x0000_0000_0000_ffff) << 16;
		let currency_1 = (asset_1.asset_index & 0x0000_0000_0000_ffff) << 32;
		let discr = 6u64 << 8;

		let index = currency_0 + currency_1 + discr;

		AssetId {
			chain_id: T::SelfParaId::get(),
			asset_type: LOCAL,
			asset_index: index,
		}
	}

	#[allow(clippy::too_many_arguments)]
	pub(crate) fn inner_add_liquidity(
		who: &T::AccountId,
		asset_0: AssetId,
		asset_1: AssetId,
		amount_0_desired: AssetBalance,
		amount_1_desired: AssetBalance,
		amount_0_min: AssetBalance,
		amount_1_min: AssetBalance,
	) -> DispatchResult {
		let pair = Self::sort_asset_id(asset_0, asset_1);
		PairStatuses::<T>::try_mutate(pair, |status| {
			if let Trading(parameter) = status {
				let reserve_0 = T::MultiAssetsHandler::balance_of(asset_0, &parameter.pair_account);
				let reserve_1 = T::MultiAssetsHandler::balance_of(asset_1, &parameter.pair_account);

				let (amount_0, amount_1) = Self::calculate_added_amount(
					amount_0_desired,
					amount_1_desired,
					amount_0_min,
					amount_1_min,
					reserve_0,
					reserve_1,
				)?;

				let balance_asset_0 = T::MultiAssetsHandler::balance_of(asset_0, who);
				let balance_asset_1 = T::MultiAssetsHandler::balance_of(asset_1, who);
				ensure!(
					balance_asset_0 >= amount_0 && balance_asset_1 >= amount_1,
					Error::<T>::InsufficientAssetBalance
				);

				let lp_asset_id = Self::lp_pairs(pair).ok_or(Error::<T>::InsufficientAssetBalance)?;

				let mint_fee = Self::mint_protocol_fee(reserve_0, reserve_1, asset_0, asset_1, parameter.total_supply);
				if let Some(fee_to) = Self::fee_meta().0 {
					if mint_fee > 0 && Self::fee_meta().1 > 0 {
						T::MultiAssetsHandler::deposit(lp_asset_id, &fee_to, mint_fee).map(|_| mint_fee)?;
						parameter.total_supply = parameter
							.total_supply
							.checked_add(mint_fee)
							.ok_or(Error::<T>::Overflow)?;
					}
				}

				let mint_liquidity =
					Self::calculate_liquidity(amount_0, amount_1, reserve_0, reserve_1, parameter.total_supply);
				ensure!(mint_liquidity > Zero::zero(), Error::<T>::Overflow);

				parameter.total_supply = parameter
					.total_supply
					.checked_add(mint_liquidity)
					.ok_or(Error::<T>::Overflow)?;

				T::MultiAssetsHandler::deposit(lp_asset_id, who, mint_liquidity).map(|_| mint_liquidity)?;

				T::MultiAssetsHandler::transfer(asset_0, who, &parameter.pair_account, amount_0)?;
				T::MultiAssetsHandler::transfer(asset_1, who, &parameter.pair_account, amount_1)?;

				if let Some(_fee_to) = Self::fee_meta().0 {
					if Self::fee_meta().1 > 0 {
						// update reserve_0 and reserve_1
						let reserve_0 = T::MultiAssetsHandler::balance_of(asset_0, &parameter.pair_account);
						let reserve_1 = T::MultiAssetsHandler::balance_of(asset_1, &parameter.pair_account);

						Self::mutate_k_last(
							asset_0,
							asset_1,
							U256::from(reserve_0).saturating_mul(U256::from(reserve_1)),
						);
					}
				}

				Self::deposit_event(Event::LiquidityAdded(
					who.clone(),
					asset_0,
					asset_1,
					amount_0,
					amount_1,
					mint_liquidity,
				));

				Ok(())
			} else {
				Err(Error::<T>::PairNotExists.into())
			}
		})
	}

	#[allow(clippy::too_many_arguments)]
	pub(crate) fn inner_remove_liquidity(
		who: &T::AccountId,
		asset_0: AssetId,
		asset_1: AssetId,
		remove_liquidity: AssetBalance,
		amount_0_min: AssetBalance,
		amount_1_min: AssetBalance,
		recipient: &T::AccountId,
	) -> DispatchResult {
		let pair = Self::sort_asset_id(asset_0, asset_1);
		PairStatuses::<T>::try_mutate(pair, |status| {
			if let Trading(parameter) = status {
				let reserve_0 = T::MultiAssetsHandler::balance_of(asset_0, &parameter.pair_account);
				let reserve_1 = T::MultiAssetsHandler::balance_of(asset_1, &parameter.pair_account);

				let amount_0 = Self::calculate_share_amount(remove_liquidity, parameter.total_supply, reserve_0);
				let amount_1 = Self::calculate_share_amount(remove_liquidity, parameter.total_supply, reserve_1);

				ensure!(
					amount_0 >= amount_0_min && amount_1 >= amount_1_min,
					Error::<T>::InsufficientTargetAmount
				);

				let lp_asset_id = Self::lp_pairs(pair).ok_or(Error::<T>::InsufficientAssetBalance)?;

				let mint_fee = Self::mint_protocol_fee(reserve_0, reserve_1, asset_0, asset_1, parameter.total_supply);
				if let Some(fee_to) = Self::fee_meta().0 {
					if mint_fee > 0 && Self::fee_meta().1 > 0 {
						//Self::mutate_liquidity(asset_0, asset_1, &fee_to, mint_fee, true)?;
						T::MultiAssetsHandler::deposit(lp_asset_id, &fee_to, mint_fee).map(|_| mint_fee)?;
						parameter.total_supply = parameter
							.total_supply
							.checked_add(mint_fee)
							.ok_or(Error::<T>::Overflow)?;
					}
				}

				parameter.total_supply = parameter
					.total_supply
					.checked_sub(remove_liquidity)
					.ok_or(Error::<T>::InsufficientLiquidity)?;

				// Self::mutate_liquidity(asset_0, asset_1, who, remove_liquidity, false)?;
				T::MultiAssetsHandler::withdraw(lp_asset_id, who, remove_liquidity).map(|_| remove_liquidity)?;

				T::MultiAssetsHandler::transfer(asset_0, &parameter.pair_account, recipient, amount_0)?;
				T::MultiAssetsHandler::transfer(asset_1, &parameter.pair_account, recipient, amount_1)?;

				if let Some(_fee_to) = Self::fee_meta().0 {
					if Self::fee_meta().1 > 0 {
						// update reserve_0 and reserve_1
						let reserve_0 = T::MultiAssetsHandler::balance_of(asset_0, &parameter.pair_account);
						let reserve_1 = T::MultiAssetsHandler::balance_of(asset_1, &parameter.pair_account);

						Self::mutate_k_last(
							asset_0,
							asset_1,
							U256::from(reserve_0).saturating_mul(U256::from(reserve_1)),
						);
					}
				}

				Self::deposit_event(Event::LiquidityRemoved(
					who.clone(),
					recipient.clone(),
					asset_0,
					asset_1,
					amount_0,
					amount_1,
					remove_liquidity,
				));

				Ok(())
			} else {
				Err(Error::<T>::PairNotExists.into())
			}
		})
	}

	#[allow(clippy::too_many_arguments)]
	pub(crate) fn inner_swap_exact_assets_for_assets(
		who: &T::AccountId,
		amount_in: AssetBalance,
		amount_out_min: AssetBalance,
		path: &[AssetId],
		recipient: &T::AccountId,
	) -> Result<AssetBalance, DispatchError> {
		let amounts = Self::get_amount_out_by_path(amount_in, path)?;
		let calculate_amount_out = amounts[amounts.len() - 1];
		ensure!(
			calculate_amount_out >= amount_out_min,
			Error::<T>::InsufficientTargetAmount
		);

		let pair_account = Self::pair_account_id(path[0], path[1]);

		T::MultiAssetsHandler::transfer(path[0], who, &pair_account, amount_in)?;
		Self::swap(&amounts, path, recipient)?;

		Self::deposit_event(Event::AssetSwap(
			who.clone(),
			recipient.clone(),
			Vec::from(path),
			amounts,
		));

		Ok(calculate_amount_out)
	}

	#[allow(clippy::too_many_arguments)]
	pub(crate) fn inner_swap_assets_for_exact_assets(
		who: &T::AccountId,
		amount_out: AssetBalance,
		amount_in_max: AssetBalance,
		path: &[AssetId],
		recipient: &T::AccountId,
	) -> DispatchResult {
		let amounts = Self::get_amount_in_by_path(amount_out, path)?;

		ensure!(amounts[0] <= amount_in_max, Error::<T>::ExcessiveSoldAmount);

		let pair_account = Self::pair_account_id(path[0], path[1]);

		T::MultiAssetsHandler::transfer(path[0], who, &pair_account, amounts[0])?;
		Self::swap(&amounts, path, recipient)?;

		Self::deposit_event(Event::AssetSwap(
			who.clone(),
			recipient.clone(),
			Vec::from(path),
			amounts,
		));

		Ok(())
	}

	fn calculate_share_amount(
		amount_0: AssetBalance,
		reserve_0: AssetBalance,
		reserve_1: AssetBalance,
	) -> AssetBalance {
		U256::from(amount_0)
			.saturating_mul(U256::from(reserve_1))
			.checked_div(U256::from(reserve_0))
			.and_then(|n| TryInto::<AssetBalance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero)
	}

	pub(crate) fn calculate_liquidity(
		amount_0: AssetBalance,
		amount_1: AssetBalance,
		reserve_0: AssetBalance,
		reserve_1: AssetBalance,
		total_liquidity: AssetBalance,
	) -> AssetBalance {
		if total_liquidity == Zero::zero() {
			let calculated_liquidity = U256::from(amount_0).saturating_mul(U256::from(amount_1)).integer_sqrt();
			TryInto::<AssetBalance>::try_into(calculated_liquidity)
				.ok()
				.unwrap_or_else(Zero::zero)
		} else {
			core::cmp::min(
				Self::calculate_share_amount(amount_0, reserve_0, total_liquidity),
				Self::calculate_share_amount(amount_1, reserve_1, total_liquidity),
			)
		}
	}

	/// Refer: https://github.com/Uniswap/uniswap-v2-core/blob/master/contracts/UniswapV2Pair.sol#L88
	/// Take as a [0, 100%] cut of the exchange fees earned by liquidity providers
	pub(crate) fn mint_protocol_fee(
		reserve_0: AssetBalance,
		reserve_1: AssetBalance,
		asset_0: AssetId,
		asset_1: AssetId,
		total_liquidity: AssetBalance,
	) -> AssetBalance {
		let new_k_last = Self::k_last(Self::sort_asset_id(asset_0, asset_1));
		let mut mint_fee: AssetBalance = 0;

		if let Some(_fee_to) = Self::fee_meta().0 {
			if !new_k_last.is_zero() && Self::fee_meta().1 > 0 {
				let root_k = U256::from(reserve_0)
					.saturating_mul(U256::from(reserve_1))
					.integer_sqrt();

				let root_k_last = new_k_last.integer_sqrt();
				if root_k > root_k_last {
					let fee_point = Self::fee_meta().1;
					let fix_fee_point = (30 - fee_point) / fee_point;
					let numerator = U256::from(total_liquidity).saturating_mul(root_k.saturating_sub(root_k_last));
					let denominator = root_k
						.saturating_mul(U256::from(fix_fee_point))
						.saturating_add(root_k_last);

					let liquidity = numerator
						.checked_div(denominator)
						.and_then(|n| TryInto::<AssetBalance>::try_into(n).ok())
						.unwrap_or_else(Zero::zero);

					if liquidity > 0 {
						mint_fee = liquidity
					}
				}
			}
		} else if !new_k_last.is_zero() {
			Self::mutate_k_last(asset_0, asset_1, U256::zero())
		}

		mint_fee
	}

	pub(crate) fn mutate_k_last(asset_0: AssetId, asset_1: AssetId, last: U256) {
		KLast::<T>::mutate(Self::sort_asset_id(asset_0, asset_1), |k| *k = last)
	}

	pub(crate) fn calculate_added_amount(
		amount_0_desired: AssetBalance,
		amount_1_desired: AssetBalance,
		amount_0_min: AssetBalance,
		amount_1_min: AssetBalance,
		reserve_0: AssetBalance,
		reserve_1: AssetBalance,
	) -> Result<(AssetBalance, AssetBalance), DispatchError> {
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

	// 0.3% exchange fee rate
	fn get_amount_in(
		output_amount: AssetBalance,
		input_reserve: AssetBalance,
		output_reserve: AssetBalance,
	) -> AssetBalance {
		if input_reserve.is_zero() || output_reserve.is_zero() || output_amount.is_zero() {
			return Zero::zero();
		}

		let numerator = U256::from(input_reserve)
			.saturating_mul(U256::from(output_amount))
			.saturating_mul(U256::from(1000));

		let denominator =
			(U256::from(output_reserve).saturating_sub(U256::from(output_amount))).saturating_mul(U256::from(997));

		numerator
			.checked_div(denominator)
			.and_then(|r| r.checked_add(U256::one()))
			.and_then(|n| TryInto::<AssetBalance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero)
	}

	// 0.3% exchange fee rate
	fn get_amount_out(
		input_amount: AssetBalance,
		input_reserve: AssetBalance,
		output_reserve: AssetBalance,
	) -> AssetBalance {
		if input_reserve.is_zero() || output_reserve.is_zero() || input_amount.is_zero() {
			return Zero::zero();
		}

		let input_amount_with_fee = U256::from(input_amount).saturating_mul(U256::from(997));

		let numerator = input_amount_with_fee.saturating_mul(U256::from(output_reserve));

		let denominator = U256::from(input_reserve)
			.saturating_mul(U256::from(1000))
			.saturating_add(input_amount_with_fee);

		numerator
			.checked_div(denominator)
			.and_then(|n| TryInto::<AssetBalance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero)
	}

	pub fn get_amount_in_by_path(
		amount_out: AssetBalance,
		path: &[AssetId],
	) -> Result<Vec<AssetBalance>, DispatchError> {
		let len = path.len();
		ensure!(len > 1, Error::<T>::InvalidPath);

		let mut i = len - 1;
		let mut out_vec = vec![amount_out];

		while i > 0 {
			let pair_account = Self::pair_account_id(path[i], path[i - 1]);
			let reserve_0 = T::MultiAssetsHandler::balance_of(path[i], &pair_account);
			let reserve_1 = T::MultiAssetsHandler::balance_of(path[i - 1], &pair_account);

			ensure!(
				reserve_1 > Zero::zero() && reserve_0 > Zero::zero(),
				Error::<T>::InvalidPath
			);

			let amount = Self::get_amount_in(out_vec[len - 1 - i], reserve_1, reserve_0);
			ensure!(amount > One::one(), Error::<T>::InvalidPath);

			// check K
			let invariant_before_swap: U256 = U256::from(reserve_0).saturating_mul(U256::from(reserve_1));
			let reserve_1_after_swap = reserve_1.checked_add(amount).ok_or(Error::<T>::Overflow)?;
			let reserve_0_after_swap = reserve_0
				.checked_sub(out_vec[len - 1 - i])
				.ok_or(Error::<T>::Overflow)?;

			let invariant_after_swap: U256 =
				U256::from(reserve_1_after_swap).saturating_mul(U256::from(reserve_0_after_swap));

			ensure!(
				invariant_after_swap >= invariant_before_swap,
				Error::<T>::InvariantCheckFailed,
			);
			out_vec.push(amount);
			i -= 1;
		}

		out_vec.reverse();
		Ok(out_vec)
	}

	pub fn get_amount_out_by_path(
		amount_in: AssetBalance,
		path: &[AssetId],
	) -> Result<Vec<AssetBalance>, DispatchError> {
		ensure!(path.len() > 1, Error::<T>::InvalidPath);

		let len = path.len() - 1;
		let mut out_vec = vec![amount_in];

		for i in 0..len {
			let pair_account = Self::pair_account_id(path[i], path[i + 1]);
			let reserve_0 = T::MultiAssetsHandler::balance_of(path[i], &pair_account);
			let reserve_1 = T::MultiAssetsHandler::balance_of(path[i + 1], &pair_account);

			ensure!(
				reserve_1 > Zero::zero() && reserve_0 > Zero::zero(),
				Error::<T>::InvalidPath
			);

			let amount = Self::get_amount_out(out_vec[i], reserve_0, reserve_1);
			ensure!(amount > Zero::zero(), Error::<T>::InvalidPath);

			// check K
			let invariant_before_swap: U256 = U256::from(reserve_0).saturating_mul(U256::from(reserve_1));

			let reserve_0_after_swap = reserve_0.checked_add(out_vec[i]).ok_or(Error::<T>::Overflow)?;
			let reserve_1_after_swap = reserve_1.checked_sub(amount).ok_or(Error::<T>::Overflow)?;

			let invariant_after_swap: U256 =
				U256::from(reserve_1_after_swap).saturating_mul(U256::from(reserve_0_after_swap));

			ensure!(
				invariant_after_swap >= invariant_before_swap,
				Error::<T>::InvariantCheckFailed,
			);

			out_vec.push(amount);
		}

		Ok(out_vec)
	}

	fn swap(amounts: &[AssetBalance], path: &[AssetId], recipient: &T::AccountId) -> DispatchResult {
		for i in 0..(amounts.len() - 1) {
			let input = path[i];
			let output = path[i + 1];
			let mut amount0_out: AssetBalance = AssetBalance::default();
			let mut amount1_out = amounts[i + 1];

			let (asset_0, asset_1) = Self::sort_asset_id(input, output);
			if input != asset_0 {
				amount0_out = amounts[i + 1];
				amount1_out = AssetBalance::default();
			}

			let pair_account = Self::pair_account_id(asset_0, asset_1);

			if i < (amounts.len() - 2) {
				let mid_account = Self::pair_account_id(output, path[i + 2]);
				Self::pair_swap(asset_0, asset_1, &pair_account, amount0_out, amount1_out, &mid_account)?;
			} else {
				Self::pair_swap(asset_0, asset_1, &pair_account, amount0_out, amount1_out, recipient)?;
			};
		}
		Ok(())
	}

	fn pair_swap(
		asset_0: AssetId,
		asset_1: AssetId,
		pair_account: &T::AccountId,
		amount_0: AssetBalance,
		amount_1: AssetBalance,
		recipient: &T::AccountId,
	) -> DispatchResult {
		let pair = Self::sort_asset_id(asset_0, asset_1);
		match Self::pair_status(pair) {
			Trading(_) => Ok(()),
			_ => Err(Error::<T>::PairNotExists),
		}?;

		let reserve_0 = T::MultiAssetsHandler::balance_of(asset_0, pair_account);
		let reserve_1 = T::MultiAssetsHandler::balance_of(asset_1, pair_account);

		ensure!(
			amount_0 <= reserve_0 && amount_1 <= reserve_1,
			Error::<T>::InsufficientPairReserve
		);

		if amount_0 > Zero::zero() {
			T::MultiAssetsHandler::transfer(asset_0, pair_account, recipient, amount_0)?;
		}

		if amount_1 > Zero::zero() {
			T::MultiAssetsHandler::transfer(asset_1, pair_account, recipient, amount_1)?;
		}

		Ok(())
	}

	pub(crate) fn do_bootstrap_contribute(
		who: T::AccountId,
		asset_0: AssetId,
		asset_1: AssetId,
		amount_0_contribute: AssetBalance,
		amount_1_contribute: AssetBalance,
	) -> DispatchResult {
		let pair = Self::sort_asset_id(asset_0, asset_1);
		let mut bootstrap_parameter = match Self::pair_status(pair) {
			PairStatus::Bootstrap(bootstrap_parameter) => {
				ensure!(
					frame_system::Pallet::<T>::block_number() < bootstrap_parameter.end_block_number,
					Error::<T>::NotInBootstrap
				);
				bootstrap_parameter
			}
			_ => {
				return Err(Error::<T>::NotInBootstrap.into());
			}
		};
		let (mut amount_0_contribute, mut amount_1_contribute) = if pair.0 == asset_0 {
			(amount_0_contribute, amount_1_contribute)
		} else {
			(amount_1_contribute, amount_0_contribute)
		};

		if amount_0_contribute.saturating_add(bootstrap_parameter.accumulated_supply.0)
			> bootstrap_parameter.capacity_supply.0
		{
			amount_0_contribute = bootstrap_parameter
				.capacity_supply
				.0
				.saturating_sub(bootstrap_parameter.accumulated_supply.0);
		}

		if amount_1_contribute.saturating_add(bootstrap_parameter.accumulated_supply.1)
			> bootstrap_parameter.capacity_supply.1
		{
			amount_1_contribute = bootstrap_parameter
				.capacity_supply
				.1
				.saturating_sub(bootstrap_parameter.accumulated_supply.1);
		}

		ensure!(
			amount_0_contribute >= One::one() || amount_1_contribute >= One::one(),
			Error::<T>::InvalidContributionAmount
		);

		BootstrapPersonalSupply::<T>::try_mutate((pair, &who), |contribution| {
			contribution.0 = contribution
				.0
				.checked_add(amount_0_contribute)
				.ok_or(Error::<T>::Overflow)?;
			contribution.1 = contribution
				.1
				.checked_add(amount_1_contribute)
				.ok_or(Error::<T>::Overflow)?;

			let pair_account = Self::account_id();

			T::MultiAssetsHandler::transfer(pair.0, &who, &pair_account, amount_0_contribute)?;
			T::MultiAssetsHandler::transfer(pair.1, &who, &pair_account, amount_1_contribute)?;

			let accumulated_supply_0 = bootstrap_parameter
				.accumulated_supply
				.0
				.checked_add(amount_0_contribute)
				.ok_or(Error::<T>::Overflow)?;

			let accumulated_supply_1 = bootstrap_parameter
				.accumulated_supply
				.1
				.checked_add(amount_1_contribute)
				.ok_or(Error::<T>::Overflow)?;
			bootstrap_parameter.accumulated_supply = (accumulated_supply_0, accumulated_supply_1);
			PairStatuses::<T>::insert(pair, Bootstrap(bootstrap_parameter));

			Self::deposit_event(Event::BootstrapContribute(
				who.clone(),
				pair.0,
				amount_0_contribute,
				pair.1,
				amount_1_contribute,
			));
			Ok(())
		})
	}

	pub(crate) fn do_end_bootstrap(asset_0: AssetId, asset_1: AssetId) -> DispatchResult {
		let pair = Self::sort_asset_id(asset_0, asset_1);
		match Self::pair_status(pair) {
			Bootstrap(bootstrap_parameter) => {
				ensure!(
					frame_system::Pallet::<T>::block_number() >= bootstrap_parameter.end_block_number
						&& bootstrap_parameter.accumulated_supply.0 >= bootstrap_parameter.target_supply.0
						&& bootstrap_parameter.accumulated_supply.1 >= bootstrap_parameter.target_supply.1,
					Error::<T>::UnqualifiedBootstrap
				);

				let total_lp_supply = Self::calculate_liquidity(
					bootstrap_parameter.accumulated_supply.0,
					bootstrap_parameter.accumulated_supply.1,
					Zero::zero(),
					Zero::zero(),
					Zero::zero(),
				);

				ensure!(total_lp_supply > Zero::zero(), Error::<T>::Overflow);

				let pair_account = Self::pair_account_id(pair.0, pair.1);
				let lp_asset_id = Self::lp_pairs(pair).ok_or(Error::<T>::InsufficientAssetBalance)?;

				T::MultiAssetsHandler::transfer(
					pair.0,
					&bootstrap_parameter.pair_account,
					&pair_account,
					bootstrap_parameter.accumulated_supply.0,
				)?;

				T::MultiAssetsHandler::transfer(
					pair.1,
					&bootstrap_parameter.pair_account,
					&pair_account,
					bootstrap_parameter.accumulated_supply.1,
				)?;

				T::MultiAssetsHandler::deposit(lp_asset_id, &pair_account, total_lp_supply).map(|_| total_lp_supply)?;

				PairStatuses::<T>::insert(
					pair,
					Trading(PairMetadata {
						pair_account,
						total_supply: total_lp_supply,
					}),
				);

				BootstrapEndStatus::<T>::insert(pair, Bootstrap(bootstrap_parameter.clone()));

				Self::deposit_event(Event::BootstrapEnd(
					pair.0,
					pair.1,
					bootstrap_parameter.accumulated_supply.0,
					bootstrap_parameter.accumulated_supply.1,
					total_lp_supply,
				));

				Ok(())
			}
			_ => Err(Error::<T>::NotInBootstrap.into()),
		}
	}

	pub(crate) fn do_bootstrap_claim(
		who: T::AccountId,
		recipient: T::AccountId,
		asset_0: AssetId,
		asset_1: AssetId,
	) -> DispatchResult {
		let pair = Self::sort_asset_id(asset_0, asset_1);
		match Self::pair_status(pair) {
			Trading(_) => BootstrapPersonalSupply::<T>::try_mutate_exists((pair, &who), |contribution| {
				if let Some((amount_0_contribute, amount_1_contribute)) = contribution.take() {
					if let Bootstrap(bootstrap_parameter) = Self::bootstrap_end_status(pair) {
						ensure!(
							!Self::bootstrap_disable(&bootstrap_parameter),
							Error::<T>::DisableBootstrap
						);
						let exact_amount_0 = U256::from(amount_0_contribute)
							.saturating_mul(U256::from(bootstrap_parameter.accumulated_supply.1))
							.checked_add(
								U256::from(amount_1_contribute)
									.saturating_mul(U256::from(bootstrap_parameter.accumulated_supply.0)),
							)
							.and_then(|r| {
								r.checked_div(
									U256::from(bootstrap_parameter.accumulated_supply.1).saturating_mul(U256::from(2)),
								)
							})
							.ok_or(Error::<T>::Overflow)?;

						let exact_amount_1 = U256::from(amount_1_contribute)
							.saturating_mul(U256::from(bootstrap_parameter.accumulated_supply.0))
							.checked_add(
								U256::from(amount_0_contribute)
									.saturating_mul(U256::from(bootstrap_parameter.accumulated_supply.1)),
							)
							.and_then(|r| {
								r.checked_div(
									U256::from(bootstrap_parameter.accumulated_supply.0).saturating_mul(U256::from(2)),
								)
							})
							.ok_or(Error::<T>::Overflow)?;

						let claim_liquidity = TryInto::<AssetBalance>::try_into(
							exact_amount_0.saturating_mul(exact_amount_1).integer_sqrt(),
						)
						.map_err(|_| Error::<T>::Overflow)?;

						let pair_account = Self::pair_account_id(pair.0, pair.1);
						let lp_asset_id = Self::lp_pairs(pair).ok_or(Error::<T>::InsufficientAssetBalance)?;

						T::MultiAssetsHandler::transfer(lp_asset_id, &pair_account, &recipient, claim_liquidity)?;

						let bootstrap_total_liquidity = TryInto::<AssetBalance>::try_into(
							U256::from(bootstrap_parameter.accumulated_supply.0)
								.saturating_mul(U256::from(bootstrap_parameter.accumulated_supply.1))
								.integer_sqrt(),
						)
						.map_err(|_| Error::<T>::Overflow)?;

						Self::bootstrap_distribute_reward(
							&who,
							&bootstrap_parameter.pair_account,
							pair.0,
							pair.1,
							claim_liquidity,
							bootstrap_total_liquidity,
						)?;

						Self::deposit_event(Event::BootstrapClaim(
							pair_account,
							who.clone(),
							recipient,
							pair.0,
							pair.1,
							amount_0_contribute,
							amount_1_contribute,
							claim_liquidity,
						));

						Ok(())
					} else {
						Err(Error::<T>::NotInBootstrap.into())
					}
				} else {
					Err(Error::<T>::ZeroContribute.into())
				}
			}),
			_ => Err(Error::<T>::NotInBootstrap.into()),
		}
	}

	pub(crate) fn do_bootstrap_refund(who: T::AccountId, asset_0: AssetId, asset_1: AssetId) -> DispatchResult {
		let pair = Self::sort_asset_id(asset_0, asset_1);

		match Self::pair_status(pair) {
			Bootstrap(params) => {
				ensure!(Self::bootstrap_disable(&params), Error::<T>::DenyRefund);
			}
			_ => {
				if let Bootstrap(bootstrap_parameter) = Self::bootstrap_end_status(pair) {
					ensure!(Self::bootstrap_disable(&bootstrap_parameter), Error::<T>::DenyRefund);
				} else {
					return Err(Error::<T>::DenyRefund.into());
				}
			}
		};

		BootstrapPersonalSupply::<T>::try_mutate_exists((pair, &who), |contribution| -> DispatchResult {
			if let Some((amount_0_contribute, amount_1_contribute)) = contribution.take() {
				let pair_account = Self::account_id();
				T::MultiAssetsHandler::transfer(pair.0, &pair_account, &who, amount_0_contribute)?;
				T::MultiAssetsHandler::transfer(pair.1, &pair_account, &who, amount_1_contribute)?;

				PairStatuses::<T>::mutate(pair, |status| {
					if let Bootstrap(parameter) = status {
						parameter.accumulated_supply.0 =
							parameter.accumulated_supply.0.saturating_sub(amount_0_contribute);
						parameter.accumulated_supply.1 =
							parameter.accumulated_supply.1.saturating_sub(amount_1_contribute);
					}
				});
				*contribution = None;

				Self::deposit_event(Event::BootstrapRefund(
					pair_account,
					who.clone(),
					pair.0,
					pair.1,
					amount_0_contribute,
					amount_1_contribute,
				));

				Ok(())
			} else {
				Err(Error::<T>::ZeroContribute.into())
			}
		})?;

		Ok(())
	}

	// After end block, bootstrap has not enough asset. Is will become disable.
	pub(crate) fn bootstrap_disable(params: &BootstrapParameter<AssetBalance, T::BlockNumber, T::AccountId>) -> bool {
		let now = frame_system::Pallet::<T>::block_number();
		if now > params.end_block_number
			&& (params.accumulated_supply.0 < params.target_supply.0
				|| params.accumulated_supply.1 < params.target_supply.1)
		{
			return true;
		}
		false
	}

	pub(crate) fn bootstrap_check_limits(asset_0: AssetId, asset_1: AssetId, account: &T::AccountId) -> bool {
		let pair = Self::sort_asset_id(asset_0, asset_1);
		let limits = Self::get_bootstrap_limits(pair);

		for (asset_id, limit) in limits.into_iter() {
			if T::MultiAssetsHandler::balance_of(asset_id, account) < limit {
				return false;
			}
		}

		true
	}

	pub(crate) fn bootstrap_distribute_reward(
		owner: &T::AccountId,
		reward_holder: &T::AccountId,
		asset_0: AssetId,
		asset_1: AssetId,
		share_lp: AssetBalance,
		total_lp: AssetBalance,
	) -> DispatchResult {
		let pair = Self::sort_asset_id(asset_0, asset_1);
		let rewards = Self::get_bootstrap_rewards(pair);

		let mut distribute_rewards = Vec::<(AssetId, AssetBalance)>::new();
		for (asset_id, reward_amount) in rewards.into_iter() {
			let owner_reward = U256::from(share_lp)
				.saturating_mul(U256::from(reward_amount))
				.checked_div(U256::from(total_lp))
				.and_then(|n| TryInto::<AssetBalance>::try_into(n).ok())
				.ok_or(Error::<T>::Overflow)?;

			T::MultiAssetsHandler::transfer(asset_id, reward_holder, owner, owner_reward)?;

			distribute_rewards.push((asset_id, owner_reward));
		}

		if distribute_rewards.len() > 0 {
			Self::deposit_event(Event::DistributeReward(
				pair.0,
				pair.1,
				reward_holder.clone(),
				distribute_rewards,
			));
		}

		Ok(())
	}
}

impl<T: Config> ExportZenlink<T::AccountId> for Pallet<T> {
	fn get_amount_in_by_path(amount_out: AssetBalance, path: &[AssetId]) -> Result<Vec<AssetBalance>, DispatchError> {
		Self::get_amount_in_by_path(amount_out, path)
	}

	fn get_amount_out_by_path(amount_in: AssetBalance, path: &[AssetId]) -> Result<Vec<AssetBalance>, DispatchError> {
		Self::get_amount_out_by_path(amount_in, path)
	}

	fn inner_swap_assets_for_exact_assets(
		who: &T::AccountId,
		amount_out: AssetBalance,
		amount_in_max: AssetBalance,
		path: &[AssetId],
		recipient: &T::AccountId,
	) -> DispatchResult {
		Self::inner_swap_assets_for_exact_assets(who, amount_out, amount_in_max, path, recipient)
	}

	fn inner_swap_exact_assets_for_assets(
		who: &T::AccountId,
		amount_in: AssetBalance,
		amount_out_min: AssetBalance,
		path: &[AssetId],
		recipient: &T::AccountId,
	) -> DispatchResult {
		Self::inner_swap_exact_assets_for_assets(who, amount_in, amount_out_min, path, recipient)?;
		Ok(())
	}

	fn inner_add_liquidity(
		who: &T::AccountId,
		asset_0: AssetId,
		asset_1: AssetId,
		amount_0_desired: AssetBalance,
		amount_1_desired: AssetBalance,
		amount_0_min: AssetBalance,
		amount_1_min: AssetBalance,
	) -> DispatchResult {
		Self::inner_add_liquidity(
			who,
			asset_0,
			asset_1,
			amount_0_desired,
			amount_1_desired,
			amount_0_min,
			amount_1_min,
		)
	}

	fn inner_remove_liquidity(
		who: &T::AccountId,
		asset_0: AssetId,
		asset_1: AssetId,
		remove_liquidity: AssetBalance,
		amount_0_min: AssetBalance,
		amount_1_min: AssetBalance,
		recipient: &T::AccountId,
	) -> DispatchResult {
		Self::inner_remove_liquidity(
			who,
			asset_0,
			asset_1,
			remove_liquidity,
			amount_0_min,
			amount_1_min,
			recipient,
		)
	}
}

impl<AccountId> ExportZenlink<AccountId> for () {
	fn get_amount_in_by_path(_amount_out: AssetBalance, _path: &[AssetId]) -> Result<Vec<AssetBalance>, DispatchError> {
		unimplemented!()
	}

	fn get_amount_out_by_path(_amount_in: AssetBalance, _path: &[AssetId]) -> Result<Vec<AssetBalance>, DispatchError> {
		unimplemented!()
	}

	fn inner_swap_assets_for_exact_assets(
		_who: &AccountId,
		_amount_out: AssetBalance,
		_amount_in_max: AssetBalance,
		_path: &[AssetId],
		_recipient: &AccountId,
	) -> DispatchResult {
		unimplemented!()
	}

	fn inner_swap_exact_assets_for_assets(
		_who: &AccountId,
		_amount_in: AssetBalance,
		_amount_out_min: AssetBalance,
		_path: &[AssetId],
		_recipient: &AccountId,
	) -> DispatchResult {
		unimplemented!()
	}

	fn inner_add_liquidity(
		_who: &AccountId,
		_asset_0: AssetId,
		_asset_1: AssetId,
		_amount_0_desired: AssetBalance,
		_amount_1_desired: AssetBalance,
		_amount_0_min: AssetBalance,
		_amount_1_min: AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn inner_remove_liquidity(
		_who: &AccountId,
		_asset_0: AssetId,
		_asset_1: AssetId,
		_remove_liquidity: AssetBalance,
		_amount_0_min: AssetBalance,
		_amount_1_min: AssetBalance,
		_recipient: &AccountId,
	) -> DispatchResult {
		unimplemented!()
	}
}
