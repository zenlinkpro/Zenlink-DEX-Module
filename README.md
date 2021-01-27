# Integration

##  Supported dependencies

It should at least work until the following commits

-   rustc 1.49.0-nightly (beb5ae474 2020-10-04)
-   polkadot.js.org (apps v0.77.2-11)
-   Polkadot rococo-v1 @ `fd414f6995ee78c051b44e2413b7d924e969fbb9`
-   Cumulus rococo-v1 @ `095f127bafe1ad6f9df64d589fd103b348c80da4`
-   Substrate rococo-v1 (newer than 2.0.0) @ `d0723f186662fd69ee37601367f51095d132df92`


## Precondition
* Open Native Currency asset control to Zenlink Dex Module. BThe ZenLink DEX Module can call both the Withdraw and Deposit methods.
* Parachain’s MessageBroker routes XCM messages to the Zenlink Dex Module.
* Establish HRMP channels between parachains.

## Example
The following shows how to deploy the ZenLink Dex Module using Rococo as an example.
The current stable version of ZenLink Dex Module is based on Cumulus rococo-v1@48558 a2bf666eba6be5b3a3b0d9ce9459bc4a714.

### Clone cumulus code

```
git clone https://github.com/paritytech/cumulus.git
cd cumulus
git checkout rococo-v1
git checkout 48558a2bf666eba6be5b3a3b0d9ce9459bc4a714
```

### Clone Zenlink DEX Module code
```
git clone git@github.com:zenlinkpro/Zenlink-DEX-Module.git
```

### Add Zenlink DEX Module in runtime
1. Copy the Zenlink Dex Module into the Cumulus code folder
2. Modify the project file
```
// runtime/Cargo.toml
[package]
…

[dependencies]
…
zenlink-protocol = { path = ‘../zenlink-protocol’, default-features = false }
zenlink-protocol-runtime-api = { path = “../zenlink-protocol/rpc/runtime-api”, default-features = false }
…
[features]
default = [‘std’]
std = [
…
	'zenlink-protocol/std',
	'zenlink-protocol-runtime-api/std',
...
]


// node/Cargo.toml

[[bin]]
...
[dependencies]
...
zenlink-protocol-runtime-api = { path = "../zenlink-protocol/rpc/runtime-api" }
zenlink-protocol-rpc = { path = "../zenlink-protocol/rpc" }
...

/// Add RPC API to node
# node/src/service.rs

...
pub async fn start_node(
	parachain_config: Configuration,
	collator_key: CollatorPair,
	polkadot_config: Configuration,
	id: polkadot_primitives::v0::Id,
	validator: bool,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)> {
	start_node_impl(
		parachain_config,
		collator_key,
		polkadot_config,
		id,
		validator,
		|client| {
			let mut io = jsonrpc_core::IoHandler::default();

			use zenlink_protocol_rpc::{ZenlinkProtocol, ZenlinkProtocolApi};
			io.extend_with(ZenlinkProtocolApi::to_delegate(ZenlinkProtocol::new(
				client,
			)));
			io
		},
	)
	.await
}
...

// Add Zenlink DEX Module to runtime
// runtime/src/lib.rs

...
// use Zenlink DEX Module
use zenlink_protocol::{
	AccountId32Aliases, AssetId, FilterAssetLocation, Junction, LocationInverter, MultiAsset,
	MultiLocation, NetworkId, Origin as ZenlinkOrigin, PairInfo, ParentIsDefault,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SovereignSignedViaLocation, TokenBalance, Transactor, XcmCfg,
	XcmExecutor,
};
...

// Zenlink DEX Module instantiation
impl zenlink_protocol::Config for Runtime {
	type Event = Event;
	type TokenBalance = TokenBalance;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type UpwardMessageSender = MessageBroker;
	type HrmpMessageSender = MessageBroker;

	type NativeCurrency = Balances;
	type AccountIdConverter = LocationConverter;
	type AccountId32Converter = AccountId32Converter;
	type ParaId = ParachainInfo;
	type ModuleId = DEXModuleId;
}
...

// Set XCM message processor as Zenlink DEX Module
impl cumulus_message_broker::Config for Runtime {
	type DownwardMessageHandlers = ZenlinkProtocol;
	type HrmpMessageHandlers = ZenlinkProtocol;
}

...

// Add Zenlink DEX Module to runtime
construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = dev_parachain_primitives::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		...
		// ASSETS_PALLET_INDEX
		ZenlinkProtocol: zenlink_protocol::{Module, Origin, Call, Storage, Event<T>} = 10,
    ...
	}
}
...

impl_runtime_apis! {
...
	// RPC implementation
	impl zenlink_protocol_runtime_api::ZenlinkProtocolApi<Block, AccountId, TokenBalance> for Runtime {
		fn get_assets() -> Vec<AssetId> {
			ZenlinkProtocol::assets_list()
		}

		fn get_balance(
			asset_id: AssetId,
			owner: AccountId
		) -> TokenBalance {
			ZenlinkProtocol::asset_balance_of(&asset_id, &owner)
		}

		fn get_all_pairs() -> Vec<PairInfo<AccountId, TokenBalance>> {
			ZenlinkProtocol::get_all_pairs()
		}

		fn get_owner_pairs(
			owner: AccountId
		) -> Vec<PairInfo<AccountId, TokenBalance>> {
			ZenlinkProtocol::get_owner_pairs(&owner)
		}

		//buy amount token price
		fn get_amount_in_price(
			path: Vec<AssetId>
		) -> TokenBalance {
			ZenlinkProtocol::get_in_price(path)
		}

		//sell amount token price
		fn get_amount_out_price(
			path: Vec<AssetId>
		) -> TokenBalance {
			ZenlinkProtocol::get_out_price(path)
		}
	}
}
...
```

### Compile Relaychain
```
git clone https://github.com/paritytech/polkadot.git
git checkout rococo-v1
git checkout 0022f1f7bfd0799c8a45a8a3749ce4e8fc556bed
cd polkadot & cargo build --release --features=real-overseer
```
### Run testnet
In this case, the test network consists of 5 Relaychain nodes and 2 Parachain Collectors. The Parachain IDs are 200 and 300, respectively.
Execute the following shell script to generate the required files, including the Relaychain configuration file, the Parachain Genesis file, and the WASM file.

```
#!/bin/bash

# polkadot
cp ../polkadot/target/release/polkadot .
mkdir config

./polkadot build-spec --chain rococo-local --disable-default-bootnode > ./config/rococo-custom-local.json
sed -i 's/"validation_upgrade_frequency": 600/"validation_upgrade_frequency": 10/g' ./config/rococo-custom-local.json
sed -I ’s/“validation_upgrade_delay”: 300/“validation_upgrade_delay”: 5/g’ ./config/rococo-custom-local.json
./polkadot build-spec —chain ./config/rococo-custom-local.json —disable-default-bootnode —raw > ./config/rococo-local.json


# dev-parachain
cp ../dev-parachain/target/release/dev-parachain .

./dev-parachain build-spec —disable-default-bootnode > ./config/200para.json
sed -I ’s/“para_id”: 500,/“para_id”: 200,/g’ ./config/200para.json
sed -I ’s/“parachainId”: 500/“parachainId”: 200/g’ ./config/200para.json
./dev-parachain export-genesis-state —parachain-id 200 > ./config/200.genesis

./dev-parachain build-spec —disable-default-bootnode > ./config/300para.json
sed -I ’s/“para_id”: 500,/“para_id”: 300,/g’ ./config/300para.json
sed -I ’s/“parachainId”: 500/“parachainId”: 300/g’ ./config/300para.json
./dev-parachain export-genesis-state —parachain-id 300 > ./config/300.genesis

./dev-parachain export-genesis-wasm > ./config/200-300.wasm
```

Execute the following shell script to start the testnet.

```
#!/bin/bash

cp config/rococo-custom-local.json config/rococo-custom-local-5.json
sed ‘/palletSession/,/palletGrandpa/{/palletSession/!{/palletGrandpa/!d}}’ -I config/rococo-custom-local-5.json
sed ‘/palletSession/ r palletSession.txt’ -I config/rococo-custom-local-5.json

./polkadot build-spec —chain config/rococo-custom-local-5.json —disable-default-bootnode —raw > config/rococo-local-5.json

mkdir data

./polkadot  —ws-port 9944 —rpc-port 10025 —port 30033  -d ./data/data-a —validator —chain config/rococo-local-5.json —Alice   > ./data/log.a 2>&1 &
./polkadot  —ws-port 9946 —rpc-port 10026 —port 30034  -d ./data/data-b —validator —chain config/rococo-local-5.json —bob     > ./data/log.b 2>&1 &
./polkadot  —ws-port 9947 —rpc-port 10027 —port 30035  -d ./data/data-c —validator —chain config/rococo-local-5.json —Charlie > ./data/log.c 2>&1 &
./polkadot  —ws-port 9948 —rpc-port 10028 —port 30036  -d ./data/data-d —validator —chain config/rococo-local-5.json —Dave    > ./data/log.d 2>&1 &
./polkadot  —ws-port 9949 —rpc-port 10029 —port 30037  -d ./data/data-e —validator —chain config/rococo-local-5.json —eve     > ./data/log.e 2>&1 &


./dev-parachain -d ./data/data-200  —ws-port 9977 —port 30336 —rpc-port 10002 —rpc-cors all —unsafe-ws-external  —unsafe-rpc-external  —parachain-id 200  —validator —execution=native -lruntime=debug,cumulus-collator=trace,cumulus-network=trace,cumulus-consensus=trace —chain ./config/200para.json  —   —chain ./config/rococo-local-5.json  —discover-local —execution=wasm > ./data/log.200 2>&1 &

./dev-parachain -d ./data/data-300  —ws-port 9988 —port 30337 —rpc-port 10003 —rpc-cors all —unsafe-ws-external  —unsafe-rpc-external  —parachain-id 300  —validator —execution=native -lruntime=debug,cumulus-collator=trace,cumulus-network=trace,cumulus-consensus=trace —chain ./config/300para.json  —   —chain ./config/rococo-local-5.json  —discover-local —execution=wasm > ./data/log.300 2>&1 &
```

### Register Parachain & Establish HRMP channel
Connect Relaychain with endpoint 'ws://127.0.0.1:9944' on https://polkadot.js.org/apps
- Register Parachain200

![Register Parachain200](./images/register_parachain200.png)

- Register Parachain300

![Register Parachain300](./images/register_parachain300.png)

- Establish HRMP channel, Parachain200 -> Parachain300

![Establish HRMP 200->300](./images/establish_200_300.png)

- Establish HRMP channel, Parachain300 -> Parachain200

![Establish HRMP 300->200](./images/establish_300_200.png)

Transaction type

```
{
  "Address": "AccountId",
  "LookupSource": "AccountId",
  "RefCount": "u32",
  "Keys": "(AccountId,AccountId,AccountId,AccountId,AccountId,AccountId)",
  "AccountInfo": "AccountInfoWithRefCount",
  "PairId": "u32",
  "PerDispatchClass": {
    "normal": "u32",
    "operational": "u32",
    "mandatory": "u32"
  },
  "BlockLength": {
    "max": "PerDispatchClass"
  },
  "Pair": {
    "token_0": "AssetId",
    "token_1": "AssetId",
    "account": "AccountId",
    "total_liquidity": "TokenBalance"
  },
  "PairInfo": {
    "token_0": "AssetId",
    "token_1": "AssetId",
    "account": "AccountId",
    "total_liquidity": "TokenBalance",
    "holding_liquidity": "TokenBalance",
    "reserve_0": "TokenBalance",
    "reserve_1": "TokenBalance"
  },
  "AssetId": {
    "_enum": {
      "NativeCurrency": null,
      "ParaCurrency": "u32"
    }
  },
  "Id": "u32",
  "TokenBalance": "u128",
  "OriginKind": {
    "_enum": {
      "Native": null,
      "SovereignAccount": null,
      "Superuser": null
    }
  },
  "NetworkId": {
    "_enum": {
      "Any": null,
      "Named": "Vec<u8>",
      "Polkadot": null,
      "Kusama": null
    }
  },
  "MultiLocation": {
    "_enum": {
      "Null": null,
      "X1": "Junction",
      "X2": "(Junction, Junction)",
      "X3": "(Junction, Junction, Junction)",
      "X4": "(Junction, Junction, Junction, Junction)"
    }
  },
  "AccountId32Junction": {
    "network": "NetworkId",
    "id": "AccountId"
  },
  "AccountIndex64Junction": {
    "network": "NetworkId",
    "index": "Compact<u64>"
  },
  "AccountKey20Junction": {
    "network": "NetworkId",
    "index": "[u8; 20]"
  },
  "Junction": {
    "_enum": {
      "Parent": null,
      "Parachain": "Compact<u32>",
      "AccountId32": "AccountId32Junction",
      "AccountIndex64": "AccountIndex64Junction",
      "AccountKey20": "AccountKey20Junction",
      "PalletInstance": "u8",
      "GeneralIndex": "Compact<u128>",
      "GeneralKey": "Vec<u8>",
      "OnlyChild": null
    }
  },
  "VersionedMultiLocation": {
    "_enum": {
      "V0": "MultiLocation"
    }
  },
  "AssetInstance": {
    "_enum": {
      "Undefined": null,
      "Index8": "u8",
      "Index16": "Compact<u16>",
      "Index32": "Compact<u32>",
      "Index64": "Compact<u64>",
      "Index128": "Compact<u128>",
      "Array4": "[u8; 4]",
      "Array8": "[u8; 8]",
      "Array16": "[u8; 16]",
      "Array32": "[u8; 32]",
      "Blob": "Vec<u8>"
    }
  },
  "AbstractFungible": {
    "id": "Vec<u8>",
    "instance": "Compact<u128>"
  },
  "AbstractNonFungible": {
    "class": "Vec<u8>",
    "instance": "AssetInstance"
  },
  "ConcreteFungible": {
    "id": "MultiLocation",
    "amount": "Compact<u128>"
  },
  "ConcreteNonFungible": {
    "class": "MultiLocation",
    "instance": "AssetInstance"
  },
  "MultiAsset": {
    "_enum": {
      "None": null,
      "All": null,
      "AllFungible": null,
      "AllNonFungible": null,
      "AllAbstractFungible": "Vec<u8>",
      "AllAbstractNonFungible": "Vec<u8>",
      "AllConcreteFungible": "MultiLocation",
      "AllConcreteNonFungible": "MultiLocation",
      "AbstractFungible": "AbstractFungible",
      "AbstractNonFungible": "AbstractNonFungible",
      "ConcreteFungible": "ConcreteFungible",
      "ConcreteNonFungible": "ConcreteNonFungible"
    }
  },
  "VersionedMultiAsset": {
    "_enum": {
      "V0": "MultiAsset"
    }
  },
  "DepositAsset": {
    "assets": "Vec<MultiAsset>",
    "dest": "MultiLocation"
  },
  "DepositReserveAsset": {
    "assets": "Vec<MultiAsset>",
    "dest": "MultiLocation",
    "effects": "Vec<Order>"
  },
  "ExchangeAsset": {
    "give": "Vec<MultiAsset>",
    "receive": "Vec<MultiAsset>"
  },
  "InitiateReserveWithdraw": {
    "assets": "Vec<MultiAsset>",
    "reserve": "MultiLocation",
    "effects": "Vec<Order>"
  },
  "InitiateTeleport": {
    "assets": "Vec<MultiAsset>",
    "dest": "MultiLocation",
    "effects": "Vec<Order>"
  },
  "QueryHolding": {
    "query_id": "Compact<u64>",
    "dest": "MultiLocation",
    "assets": "Vec<MultiAsset>"
  },
  "Order": {
    "_enum": {
      "Null": null,
      "DepositAsset": "DepositAsset",
      "DepositReserveAsset": "DepositReserveAsset",
      "ExchangeAsset": "ExchangeAsset",
      "InitiateReserveWithdraw": "InitiateReserveWithdraw",
      "InitiateTeleport": "InitiateTeleport",
      "QueryHolding": "QueryHolding"
    }
  },
  "WithdrawAsset": {
    "assets": "Vec<MultiAsset>",
    "effects": "Vec<Order>"
  },
  "ReserveAssetDeposit": {
    "assets": "Vec<MultiAsset>",
    "effects": "Vec<Order>"
  },
  "TeleportAsset": {
    "assets": "Vec<MultiAsset>",
    "effects": "Vec<Order>"
  },
  "Balances": {
    "query_id": "Compact<u64>",
    "assets": "Vec<MultiAsset>"
  },
  "Transact": {
    "origin_type": "OriginKind",
    "call": "Vec<u8>"
  },
  "RelayTo": {
    "dest": "MultiLocation",
    "inner": "VersionedXcm"
  },
  "RelayedFrom": {
    "superorigin": "MultiLocation",
    "inner": "VersionedXcm"
  },
  "Xcm": {
    "_enum": {
      "WithdrawAsset": "WithdrawAsset",
      "ReserveAssetDeposit": "ReserveAssetDeposit",
      "TeleportAsset": "TeleportAsset",
      "Balances": "Balances",
      "Transact": "Transact",
      "RelayTo": "RelayTo",
      "RelayedFrom": "RelayedFrom"
    }
  },
  "VersionedXcm": {
    "_enum": {
      "V0": "Xcm"
    }
  },
  "XcmError": {
    "_enum": [
      "Undefined",
      "Unimplemented",
      "UnhandledXcmVersion",
      "UnhandledXcmMessage",
      "UnhandledEffect",
      "EscalationOfPrivilege",
      "UntrustedReserveLocation",
      "UntrustedTeleportLocation",
      "DestinationBufferOverflow",
      "CannotReachDestination",
      "MultiLocationFull",
      "FailedToDecode",
      "BadOrigin"
    ]
  },
  "XcmResult": {
    "_enum": {
      "Ok": "()",
      "Err": "XcmError"
    }
  },
  "HrmpChannelId": {
    "sender": "u32",
    "receiver": "u32"
  },
  "AvailabilityBitfield": "BitVec",
  "SignedAvailabilityBitfield": {
    "payload": "BitVec",
    "validator_index": "u32",
    "signature": "Signature"
  },
  "SignedAvailabilityBitfields": "Vec<SignedAvailabilityBitfield>",
  "ValidatorSignature": "Signature",
  "HeadData": "Vec<u8>",
  "CandidateDescriptor": {
    "paraId": "u32",
    "relayParent": "Hash",
    "collator": "Hash",
    "persistedValidationDataHash": "Hash",
    "povHash": "Hash",
    "erasureRoot": "Hash",
    "signature": "Signature"
  },
  "CandidateReceipt": {
    "descriptor": "CandidateDescriptor",
    "commitments_hash": "Hash"
  },
  "UpwardMessage": "Vec<u8>",
  "OutboundHrmpMessage": {
    "recipient": "u32",
    "data": "Vec<u8>"
  },
  "ValidationCode": "Vec<u8>",
  "CandidateCommitments": {
    "upward_messages": "Vec<UpwardMessage>",
    "horizontal_messages": "Vec<OutboundHrmpMessage>",
    "new_validation_code": "Option<ValidationCode>",
    "head_data": "HeadData",
    "processed_downward_messages": "u32",
    "hrmp_watermark": "BlockNumber"
  },
  "CommittedCandidateReceipt": {
    "descriptor": "CandidateDescriptor",
    "commitments": "CandidateCommitments"
  },
  "ValidityAttestation": {
    "_enum": {
      "DummyOffsetBy1": "Raw",
      "Implicit": "ValidatorSignature",
      "Explicit": "ValidatorSignature"
    }
  },
  "BackedCandidate": {
    "candidate": "CommittedCandidateReceipt",
    "validity_votes": "Vec<ValidityAttestation>",
    "validator_indices": "BitVec"
  },
  "CandidatePendingAvailablility": {
    "core": "u32",
    "descriptor": "CandidateDescriptor",
    "availability_votes": "BitVec",
    "relay_parent_number": "BlockNumber",
    "backed_in_number": "BlockNumber"
  },
  "BufferedSessionChange": {
    "apply_at": "BlockNumber",
    "validators": "Vec<ValidatorId>",
    "queued": "Vec<ValidatorId>",
    "session_index": "SessionIndex"
  },
  "HostConfiguration": {
    "max_code_size": "u32",
    "max_head_data_size": "u32",
    "max_upward_queue_count": "u32",
    "max_upward_queue_size": "u32",
    "max_upward_message_size": "u32",
    "max_upward_message_num_per_candidate": "u32",
    "hrmp_max_message_num_per_candidate": "u32",
    "validation_upgrade_frequency": "u32",
    "validation_upgrade_delay": "u32",
    "max_pov_size": "u32",
    "max_downward_message_size": "u32",
    "preferred_dispatchable_upward_messages_step_weight": "Weight",
    "hrmp_max_parachain_outbound_channels": "u32",
    "hrmp_max_parathread_outbound_channels": "u32",
    "hrmp_open_request_ttl": "u32",
    "hrmp_sender_deposit": "Balance",
    "hrmp_recipient_deposit": "Balance",
    "hrmp_channel_max_capacity": "u32",
    "hrmp_channel_max_total_size": "u32",
    "hrmp_max_parachain_inbound_channels": "u32",
    "hrmp_max_parathread_inbound_channels": "u32",
    "hrmp_channel_max_message_size": "u32",
    "acceptance_period": "u32",
    "parathread_cores": "u32",
    "parathread_retries": "u32",
    "group_rotation_frequency": "u32",
    "chain_availability_period": "u32",
    "thread_availability_period": "u32",
    "scheduling_lookahead": "u32",
    "max_validators_per_core": "Option<u32>",
    "dispute_period": "u32",
    "no_show_slots": "u32",
    "n_delay_tranches": "u32",
    "zeroth_delay_tranche_width": "u32",
    "needed_approvals": "u32",
    "relay_vrf_modulo_samples": "u32"
  },
  "InboundDownwardMessage": {
    "sent_at": "u32",
    "msg": "Vec<u8>"
  },
  "InboundHrmpMessage": {
    "sent_at": "u32",
    "data": "Vec<u8>"
  },
  "MessageIngestionType": {
    "dmp": "Vec<InboundDownwardMessage>",
    "hrmp": "BTreeMap<u32, Vec<InboundHrmpMessage>>"
  },
  "HrmpChannel": {
    "max_capacity": "u32",
    "max_total_size": "u32",
    "max_message_size": "u32",
    "msg_count": "u32",
    "total_size": "u32",
    "mqc_head": "Option<Hash>",
    "sender_deposit": "Balance",
    "recipient_deposit": "Balance"
  },
  "PersistedValidationData": {
    "parent_head": "HeadData",
    "block_number": "BlockNumber",
    "relay_storage_root": "Hash",
    "hrmp_mqc_heads": "Vec<(Id, Hash)>",
    "dmq_mqc_head": "Hash",
    "max_pov_size": "u32"
  },
  "TransientValidationData": {
    "max_code_size": "u32",
    "max_head_data_size": "u32",
    "balance": "Balance",
    "code_upgrade_allowed": "Option<BlockNumber>",
    "dmq_length": "u32"
  },
  "ValidationData": {
    "persisted": "PersistedValidationData",
    "transient": "TransientValidationData"
  },
  "StorageProof": {
    "trie_nodes": "Vec<Vec<u8>>"
  },
  "ValidationDataType": {
    "validation_data": "ValidationData",
    "relay_chain_state": "StorageProof"
  }
}
```

Switch to 'Settings' -> 'Developer', input the above json, then save.

![Setting Type](./images/setting.png)

### Add liquidity
There is a trading pair on Parachain300. Call the following method.
![Add Liquidity](./images/add_liquidity.png)


### Swap
![Swap](./images/swap.png)
