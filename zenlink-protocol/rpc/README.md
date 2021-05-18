# zenlink-protocol-rpc(v0.4.0)

#### 1. introduction (参数和返回值类型参见下文)

- 1.`zenlinkProtocol_getAllAssets`:

  查询Zenlink Module中的所有资产ID。通常有映射的资产和LP token
  - {"chain_id":200,"asset_type":0,"asset_index":0}: ParaId=200, Native Currency
  - {"chain_id":200,"asset_type":1,"asset_index":0}: ParaId=300, Liquidity Asset

  ```
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
    '{
      "jsonrpc":"2.0",
      "id":1,
      "method":"zenlinkProtocol_getAllAssets",
      "params": [null]
    }'
  ```
  
  **Response:**
  
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      {
        "asset_index": 0,
        "asset_type": 0,
        "chain_id": 200
      },
      {
        "asset_index": 0,
        "asset_type": 1,
        "chain_id": 300
      }
    ],
    "id": 1
  }
  ```
   
- 2.`zenlinkProtocol_getBalance`:

  查询资产余额

  - params[0]: AssetId。包括了本链的资产和映射的资产
  - params[1]: 查询账户
    
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
     "jsonrpc":"2.0",
     "id":1,
     "method":"zenlinkProtocol_getBalance",
     "params": [{"chain_id": 200,"asset_type": 0, "asset_index": 0 }, "5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL"]
   }'  
  ```
  
  **Response:**
  
  ```json
  {
    "jsonrpc": "2.0",
    "result": "0x3fffffffffc1ed7f40",
    "id": 1
  }
  ```
  
- 3.`zenlinkProtocol_getSovereignsInfo`：
  查询跨链转出资产的原始信息,
  返回Vec<(paraid, sovereign_account, balance)>
  
  ```bash
    curl -H "Content-Type: application/json" http://localhost:11111 -d \
    '{
       "jsonrpc":"2.0",
       "id":1,
       "method":"zenlinkProtocol_getSovereignsInfo",
       "params": [{"chain_id": 200,"asset_type": 0, "asset_index":0}]
     }'  
    ```
  
  **Response:**
  
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      [
        200,
        "5Eg2fntGQpyQv5X5d8N5qxG4sX5UBMLG77xEBPjZ9DTxxtt7",
        "0x0"
      ],
      [
        300,
        "5Eg2fnsj9u3qQZcwEtTDxFqWFHsUcYqupaS8MtEPoeHKAXA4",
        "0x0"
      ]
    ],
    "id": 1
  }
  ```
  
- 4.`zenlinkProtocol_getAllPairs`：

  查询所有的交易对

    ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getAllPairs",
        "params": [null]
  }'  
    ```
  
  **Response:**
  - account:交易对的账户
  - holdingLiquidity: 此账户持有的lptoken量
  - reserve0: 交易池中token0的数量
  - reserve1: 交易池中asset1的数量
  - asset0 & asset1: 组成交易对的两种资产
  - totalLiquidity：lptoken 总量
  - lpAssetId: 交易中的LP token的 AssetId
  
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      {
        "account": "5EYCAe5ViNAoHnU1ZZVit8ymcR39EP5fyU6Zv3GV7HD5MN9d",
        "asset0": {
          "asset_index": 0,
          "asset_type": 0,
          "chain_id": 200
        },
        "asset1": {
          "asset_index": 0,
          "asset_type": 0,
          "chain_id": 300
        },
        "holdingLiquidity": "0x0",
        "lpAssetId": {
          "asset_index": 0,
          "asset_type": 1,
          "chain_id": 300
        },
        "reserve0": "0x1d91d9f5",
        "reserve1": "0x29d7f22d",
        "totalLiquidity": "0x232aaf80"
      }
    ],
    "id": 1
  }
  ```

- 5.`zenlinkProtocol_getOwnerPairs`：

  查询某个账户拥有的交易对
  
  - params: 查询账户
  
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getOwnerPairs",
        "params": ["5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL", null]
  }'
  ```
  
  **Reponse:**
  
  - account:交易对的账户
  - holdingLiquidity: 此账户持有的lptoken量
  - reserve0: 交易池中asset0的数量
  - reserve1: 交易池中asset1的数量
  - asset0 & asset1: 组成交易对的两种资产
  - totalLiquidity：lptoken 总量
  - lpAssetId: 交易中的LP token的 AssetId
  
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      {
        "account": "5EYCAe5ViNAoHnU1ZZVit8ymcR39EP5fyU6Zv3GV7HD5MN9d",
        "asset0": {
          "asset_index": 0,
          "asset_type": 0,
          "chain_id": 200
        },
        "asset1": {
          "asset_index": 0,
          "asset_type": 0,
          "chain_id": 300
        },
        "holdingLiquidity": "0x232aaf80",
        "lpAssetId": {
          "asset_index": 0,
          "asset_type": 1,
          "chain_id": 300
        },
        "reserve0": "0x1d91d9f5",
        "reserve1": "0x29d7f22d",
        "totalLiquidity": "0x232aaf80"
      }
    ],
    "id": 1
  }
  ```
  
- 6.`zenlinkProtocol_getPairByAssetId`：

  获取指定资产ID的交易对信息
  
  - params: (200,0,0)(300,0,0)和组成交易对
  
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
     "jsonrpc":"2.0",
     "id":1,
     "method":"zenlinkProtocol_getPairByAssetId",
     "params": [
       {"chain_id": 200,"asset_type": 0, "asset_index":0}, 
       {"chain_id": 300,"asset_type": 0, "asset_index":0},
       null
     ]
   }'
  ```

  **Response**
    - account:交易对的账户
    - holdingLiquidity:持有的LPtoken。实际上，由于lp token全部由交易者持有，这里通常为0.
    - reserve0: 交易池中asset0的数量
    - reserve1: 交易池中asset1的数量
    - asset0 & asset1: 组成交易对的两种资产
    - totalLiquidity：lptoken总量
    - lpAssetId: 交易中的LP token的 AssetId
    
  ```json
  {
    "jsonrpc": "2.0",
    "result": {
      "account": "5EYCAe5ViNAoHnU1ZZVit8ymcR39EP5fyU6Zv3GV7HD5MN9d",
      "asset0": {
        "asset_index": 0,
        "asset_type": 0,
        "chain_id": 200
      },
      "asset1": {
        "asset_index": 0,
        "asset_type": 0,
        "chain_id": 300
      },
      "holdingLiquidity": "0x0",
      "lpAssetId": {
        "asset_index": 0,
        "asset_type": 1,
        "chain_id": 300
      },
      "reserve0": "0x1d91d9f5",
      "reserve1": "0x29d7f22d",
      "totalLiquidity": "0x232aaf80"
    },
    "id": 1
  }
  ```
  
- 7.`zenlinkProtocol_getAmountInPrice`： 
  
  查询买入汇率（固定交易对右边）
  
  - params[0]: "100": 买入的量
  - params[1]: 兑换路径。
  
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
     "jsonrpc":"2.0",
     "id":1,
     "method":"zenlinkProtocol_getAmountInPrice",
     "params": [
       100000000,
       [
         {"chain_id": 200,"asset_type": 0, "asset_index":0},
         {"chain_id": 300,"asset_type": 0, "asset_index":0}
       ],
       null
     ]
   }'  
    ```
  
  **Response:**
  - result: 99226799： 表示的是用10000000个（200,0,0）兑换出82653754个(300,0,0)。
    
  ```json
  {
    "jsonrpc": "2.0",
    "result": "0x4ed323a",
    "id": 1
  }
  ```
  
- 8.`zenlinkProtocol_getAmountOutPrice`：
  
  查询卖出汇率（固定交易对左边）
  
  - params[0]: "100000000": 卖出的量
  - params[1]: 交易路径。
  
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
     "jsonrpc":"2.0",
     "id":1,
     "method":"zenlinkProtocol_getAmountOutPrice",
     "params": [
       100000000,
       [
         {"chain_id": 200,"asset_type": 0, "asset_index":0},
         {"chain_id": 300,"asset_type": 0, "asset_index":0}
       ],
       null
     ]
   }'  
  ```

  **Response:**
  
  ```json
  {
    "jsonrpc": "2.0",
    "result": "0x70085cc",
    "id": 1
  }
  ```
    
- 9.`zenlinkProtocol_getEstimateLptoken`:

  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
     "jsonrpc":"2.0",
     "id":1,
     "method":"zenlinkProtocol_getEstimateLptoken",
     "params": [
       {"chain_id": 200,"asset_type": 0, "asset_index":0},
       {"chain_id": 300,"asset_type": 0, "asset_index":0},
       10000000,
       40000000,
       1,
       1,
       null
     ]
   }'  
  ```
  
  **Response:**
  
 ```json
  {
    "jsonrpc": "2.0",
    "result": "0xb5784f",
    "id": 1
  }
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
          "name": "asset_id",
          "type": "AssetId"
        },
        {
          "name": "account",
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
          "name": "account",
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
          "name": "asset_0",
          "type": "AssetId"
        },
        {
          "name": "asset_1",
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
          "type": "AssetBalance"
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
          "type": "AssetBalance"
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
    "getEstimateLptoken":{
            "description": "zenlinkProtocol getEstimateLptoken",
            "params": [
        {
          "name": "asset_0",
          "type": "AssetId"
        },
        {
          "name": "asset_1",
          "type": "AssetId"
        },
                {
          "name": "amount_0_desired",
          "type": "AssetBalance"
        },
                {
          "name": "amount_1_desired",
          "type": "AssetBalance"
        },
                {
          "name": "amount_0_min",
          "type": "AssetBalance"
        },
                {
          "name": "amount_1_min",
          "type": "AssetBalance"
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
  "Address": "MultiAddress",
  "LookupSource": "MultiAddress",
  "AssetId": {
    "chain_id": "u32",
    "asset_type": "u8",
    "asset_index": "u32"
  },
  "AssetBalance": "u128",
  "PairInfo": {
    "token_0": "AssetId",
    "token_1": "AssetId",
    "account": "AccountId",
    "total_liquidity": "AssetBalance",
    "holding_liquidity": "AssetBalance",
    "reserve_0": "AssetBalance",
    "reserve_1": "AssetBalance",
    "lp_asset_id": "AssetId"
  },
  "TokenId": "u32"
}
```
