# zenlink-protocol-rpc

#### 1. introduction (参数和返回值类型参见下文)

- `zenlinkProtocol_getAllAssets`:

  获取当前链上的Parachain资产列表

    ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
         "jsonrpc":"2.0",
         "id":1,
         "method":"zenlinkProtocol_getAllAssets",
         "params": [null]
  }'  
    ```
- `zenlinkProtocol_getBalance`:

  获取指定Parachain资产ID和账户ID的余额

    ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getBalance",
        "params": [{"ParaCurrency": 200}, "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"]
  }'  
    ```
- `zenlinkProtocol_getAllPairs`：

  获取当前链上所有的交易对列表

    ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getAllPairs",
        "params": [null]
  }'  
    ```
- `zenlinkProtocol_getOwnerPairs`：

  获取和指定账户ID相关的交易对列表

    ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getOwnerPairs",
        "params": ["5ExnkKkHG1xfgoCLjQ2DTHBkRxdoUrsnCsVorWKPsESrtpmd", null]
  }'  
    ```
- `zenlinkProtocol_getPairByAssetId`：

  获取指定资产ID的交易对信息
    ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getPairByAssetId",
        "params": [{"ParaCurrency": 200}, "NativeCurrency"]
  }'  
    ```    
- `zenlinkProtocol_getAmountInPrice`： Path中的第一个元素代表持有的资产， 最后一个元素代表要兑换的目标资产指定要支付的持有资产数量， 获取能兑换的目标资产数量.

    ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getAmountInPrice",
        "params": [1000, [{"ParaCurrency": 200}, "NativeCurrency"], null]
  }'  
    ```
- `zenlinkProtocol_getAmountOutPrice`： Path中的第一个元素代表持有的资产， 最后一个元素代表要兑换的目标资产指定要兑换的目标资产数量， 获取要支付的持有资产数量.

    ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getAmountOutPrice",
        "params": [1000, ["NativeCurrency", {"ParaCurrency": 200}], null]
  }'  
    ```

#### 2. rpc calls

```json
{
  "zenlinkProtocol": {
    "getAllAssets": {
      "description": "zenlinkProtocol getAllAssets",
      "params": [
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "Vec<AssetId>"
    },
    "getBalance": {
      "description": "zenlinkProtocol getBalance",
      "params": [
        {
          "name": "asset",
          "type": "AssetId"
        },
        {
          "name": "owner",
          "type": "AccountID"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "string"
    },
    "getAllPairs": {
      "description": "zenlinkProtocol getAllPairs",
      "params": [
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "Vec<PairInfo>"
    },
    "getOwnerPairs": {
      "description": "zenlinkProtocol getOwnerPairs",
      "params": [
        {
          "name": "owner",
          "type": "AccountID"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "Vec<PairInfo>"
    },
    "getPairByAssetId": {
      "description": "zenlinkProtocol getPairByAssetId",
      "params": [
        {
          "name": "token_0",
          "type": "AssetId"
        },
        {
          "name": "token_1",
          "type": "AssetId"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "PairInfo"
    },
    "getAmountInPrice": {
      "description": "zenlinkProtocol getAmountInPrice",
      "params": [
        {
          "name": "amount_out",
          "type": "TokenBalance"
        },
        {
          "name": "path",
          "type": "Vec<AssetId>"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "string"
    },
    "getAmountOutPrice": {
      "description": "zenlinkProtocol getAmountOutPrice",
      "params": [
        {
          "name": "amount_in",
          "type": "TokenBalance"
        },
        {
          "name": "path",
          "type": "Vec<AssetId>"
        },
        {
          "name": "at",
          "type": "Hash",
          "isOptional": true
        }
      ],
      "type": "string"
    }
  }
}
```

#### 3. type

```json
{
  "Address": "AccountId",
  "LookupSource": "AccountId",
  "RefCount": "u32",
  "Keys": "(AccountId,AccountId,AccountId,AccountId,AccountId,AccountId)",
  "AccountInfo": "AccountInfoWithRefCount",
  "PairId": "u32",
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
