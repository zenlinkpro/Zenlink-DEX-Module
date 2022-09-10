#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use sp_std::{convert::TryFrom, prelude::*};
use zenlink_protocol::*;

pub const PARA_CHAIN_ID: u32 = 1000;

macro_rules! impl_tokensymbol {
	($(#[$meta:meta])*
	$vis:vis enum TokenSymbol {
		$($(#[$vmeta:meta])* $symbol:ident($name:expr, $deci:literal) = $val:literal,)*
	}) => {
		$(#[$meta])*
		$vis enum TokenSymbol {
			$($(#[$vmeta])* $symbol = $val,)*
		}

		impl TryFrom<u8> for TokenSymbol {
			type Error = ();

			fn try_from(v: u8) -> Result<Self, Self::Error> {
				match v {
					$($val => Ok(TokenSymbol::$symbol),)*
					_ => Err(()),
				}
			}
		}
	}
}

impl_tokensymbol! {
#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
	Dev("dev",12) = 0,
	LOCAL1("local1",12) = 1,
	LOCAL2("local2",12) = 2,
	LOCAL3("local3",12) = 3,
	LOCAL4("local4",12) = 4,
	LOCAL5("local5",12) = 5,
	LOCAL6("local6",12) = 6,
	LOCAL7("local7",12) = 7,
}
}

#[derive(
	Encode,
	Decode,
	MaxEncodedLen,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum CurrencyId {
	Native(TokenSymbol),
	Foregin(TokenSymbol),
	Token(TokenSymbol),
	LPToken(TokenSymbol, TokenSymbol),
	StableLpToken(u32),
}

pub type ZenlinkAssetId = zenlink_protocol::AssetId;
const LP_DISCRIMINANT: u64 = 6u64;
const TOKEN_DISCRIMINANT: u64 = 2u64;

impl TryFrom<CurrencyId> for ZenlinkAssetId {
	type Error = ();

	fn try_from(currency_id: CurrencyId) -> Result<Self, Self::Error> {
		match currency_id {
			CurrencyId::Native(symbol) => Ok(ZenlinkAssetId {
				chain_id: PARA_CHAIN_ID,
				asset_type: NATIVE,
				asset_index: symbol as u64,
			}),
			CurrencyId::Token(symbol) => Ok(ZenlinkAssetId {
				chain_id: PARA_CHAIN_ID,
				asset_type: LOCAL,
				asset_index: TOKEN_DISCRIMINANT << 8 + symbol as u64,
			}),
			CurrencyId::LPToken(symbol0, symbol1) => Ok(ZenlinkAssetId {
				chain_id: PARA_CHAIN_ID,
				asset_type: LOCAL,
				asset_index: (LP_DISCRIMINANT << 8) +
					((symbol0 as u64 & 0xffff) << 16) +
					((symbol1 as u64 & 0xffff) << 32),
			}),
			_ => Err(()),
		}
	}
}

impl TryFrom<ZenlinkAssetId> for CurrencyId {
	type Error = ();
	fn try_from(asset_id: ZenlinkAssetId) -> Result<Self, Self::Error> {
		if asset_id.is_native(PARA_CHAIN_ID) {
			return Ok(CurrencyId::Native(TokenSymbol::try_from(asset_id.asset_index as u8)?))
		}

		let discriminant = (asset_id.asset_index & 0x0000_0000_0000_ff00) >> 8;
		return if discriminant == LP_DISCRIMINANT {
			let token0_id = ((asset_id.asset_index & 0x0000_0000_ffff_0000) >> 16) as u8;
			let token1_id = ((asset_id.asset_index & 0x0000_ffff_0000_0000) >> 16) as u8;
			Ok(CurrencyId::LPToken(
				TokenSymbol::try_from(token0_id)?,
				TokenSymbol::try_from(token1_id)?,
			))
		} else if discriminant == TOKEN_DISCRIMINANT {
			let token_id = asset_id.asset_index as u8;
			Ok(CurrencyId::Token(TokenSymbol::try_from(token_id)?))
		} else {
			Err(())
		}
	}
}
