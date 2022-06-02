pub trait ValidateCurrency<CurrencyId> {
	fn validate_pooled_currency(a: &[CurrencyId]) -> bool;
	fn validate_pool_lp_currency(a: CurrencyId) -> bool;
}
