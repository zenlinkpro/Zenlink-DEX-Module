use super::*;

pub trait ValidateCurrency<CurrencyId> {
	fn validate_pooled_currency(a: &[CurrencyId]) -> bool;
	fn validate_pool_lp_currency(a: CurrencyId) -> bool;
}

pub trait StableAmmApi<PoolId, CurrencyId, AccountId, Balance> {
	fn stable_amm_calculate_currency_amount(pool_id: PoolId, amounts: &[Balance], deposit: bool)
											-> Result<Balance, DispatchError>;

	fn stable_amm_calculate_swap_amount(pool_id: PoolId, i: usize, j: usize, in_balance: Balance) -> Option<Balance>;

	fn stable_amm_calculate_remove_liquidity_one_currency(pool_id: PoolId, amount: Balance, index: u32) -> Option<Balance>;

	fn add_liquidity(
		who: &AccountId,
		pool_id: PoolId,
		amounts: &[Balance],
		min_mint_amount: Balance,
	) -> Result<Balance, sp_runtime::DispatchError>;

	fn swap(
		who: &AccountId,
		poo_id: PoolId,
		from_index: u32,
		to_index: u32,
		in_amount: Balance,
		min_out_amount: Balance,
	) -> Result<Balance, sp_runtime::DispatchError>;

	fn remove_liquidity(who: &AccountId, poo_id: PoolId, lp_amount: Balance, min_amounts: &[Balance])
		-> DispatchResult;

	fn remove_liquidity_one_currency(
		who: &AccountId,
		poo_id: PoolId,
		lp_amount: Balance,
		index: u32,
		min_amount: Balance,
	) -> DispatchResult;

	fn remove_liquidity_imbalance(
		who: &AccountId,
		pool_id: PoolId,
		amounts: &[Balance],
		max_burn_amount: Balance,
	) -> DispatchResult;
}

impl<T: Config> StableAmmApi<T::PoolId, T::CurrencyId, T::AccountId, Balance> for Pallet<T> {
	fn stable_amm_calculate_currency_amount(
		pool_id: T::PoolId,
		amounts: &[Balance],
		deposit: bool,
	) -> Result<Balance, DispatchError> {
		Self::calculate_currency_amount(pool_id, amounts.to_vec(), deposit)
	}

	fn stable_amm_calculate_swap_amount(pool_id: T::PoolId, i: usize, j: usize, in_balance: Balance) -> Option<Balance> {
		if let Some(pool) = Self::pools(pool_id) {
			return Self::calculate_swap_amount(&pool, i, j, in_balance);
		}
		None
	}

	fn stable_amm_calculate_remove_liquidity_one_currency(pool_id: T::PoolId, amount: Balance, index: u32) -> Option<Balance> {
		if let Some(pool) = Self::pools(pool_id) {
			if let Some(res) = Self::calculate_remove_liquidity_one_token(&pool, amount, index) {
				return Some(res.0);
			}
		}
		None
	}

	fn add_liquidity(
		who: &T::AccountId,
		pool_id: T::PoolId,
		amounts: &[Balance],
		min_mint_amount: Balance,
	) -> Result<Balance, sp_runtime::DispatchError> {
		Self::inner_add_liquidity(who, pool_id, amounts, min_mint_amount)
	}

	fn swap(
		who: &T::AccountId,
		poo_id: T::PoolId,
		from_index: u32,
		to_index: u32,
		in_amount: Balance,
		min_out_amount: Balance,
	) -> Result<Balance, sp_runtime::DispatchError> {
		Self::inner_swap(
			who,
			poo_id,
			from_index as usize,
			to_index as usize,
			in_amount,
			min_out_amount,
		)
	}

	fn remove_liquidity(
		who: &T::AccountId,
		poo_id: T::PoolId,
		lp_amount: Balance,
		min_amounts: &[Balance],
	) -> DispatchResult {
		Self::inner_remove_liquidity(poo_id, who, lp_amount, min_amounts)
	}

	fn remove_liquidity_one_currency(
		who: &T::AccountId,
		poo_id: T::PoolId,
		lp_amount: Balance,
		index: u32,
		min_amount: Balance,
	) -> DispatchResult {
		Self::inner_remove_liquidity_one_currency(poo_id, who, lp_amount, index, min_amount)
	}

	fn remove_liquidity_imbalance(
		who: &T::AccountId,
		pool_id: T::PoolId,
		amounts: &[Balance],
		max_burn_amount: Balance,
	) -> DispatchResult {
		Self::inner_remove_liquidity_imbalance(who, pool_id, amounts, max_burn_amount)
	}
}
