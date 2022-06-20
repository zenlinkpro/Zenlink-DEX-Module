// Copyright 2021-2022 Zenlink
// Licensed under GPL-3.0.

#![allow(clippy::type_complexity)]

use super::*;

impl<T: Config> Pallet<T> {
	pub fn get_virtual_price(pool_id: T::PoolId) -> Balance {
		Self::calculate_virtual_price(pool_id).unwrap_or_default()
	}

	pub fn get_a(pool_id: T::PoolId) -> Balance {
		if let Some(pool) = Self::pools(pool_id) {
			return Self::get_a_precise(&pool).unwrap_or_default() / A_PRECISION;
		};
		Balance::default()
	}

	pub fn get_a_precise_by_id(pool_id: T::PoolId) -> Balance {
		if let Some(pool) = Self::pools(pool_id) {
			return Self::get_a_precise(&pool).unwrap_or_default();
		};
		Balance::default()
	}

	pub fn get_currencies(pool_id: T::PoolId) -> Vec<T::CurrencyId> {
		if let Some(pool) = Self::pools(pool_id) {
			return pool.currency_ids;
		};
		Vec::new()
	}

	pub fn get_currency(pool_id: T::PoolId, index: u32) -> Option<T::CurrencyId> {
		if let Some(pool) = Self::pools(pool_id) {
			if pool.currency_ids.len() < index as usize {
				return Some(pool.currency_ids[index as usize]);
			}
		};
		None
	}

	pub fn get_lp_currency(pool_id: T::PoolId) -> Option<T::CurrencyId> {
		if let Some(pool) = Self::pools(pool_id) {
			return Some(pool.lp_currency_id);
		};
		None
	}

	pub fn get_currency_precision_multipliers(pool_id: T::PoolId) -> Vec<Balance> {
		if let Some(pool) = Self::pools(pool_id) {
			return pool.token_multipliers;
		};
		Vec::new()
	}

	pub fn get_currency_balances(pool_id: T::PoolId) -> Vec<Balance> {
		if let Some(pool) = Self::pools(pool_id) {
			return pool.balances;
		};
		Vec::new()
	}

	pub fn get_number_of_currencies(pool_id: T::PoolId) -> u32 {
		if let Some(pool) = Self::pools(pool_id) {
			return pool.currency_ids.len() as u32;
		};
		0
	}

	pub fn get_admin_balances(pool_id: T::PoolId) -> Vec<Balance> {
		let mut balances = Vec::new();
		if let Some(pool) = Self::pools(pool_id) {
			for (i, _) in pool.currency_ids.iter().enumerate() {
				balances.push(Self::get_admin_balance(pool_id, i).unwrap_or_default());
			}
		};
		balances
	}
}
