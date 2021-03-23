# zenlink-protocol-rpc

#### 1. introduction (参数和返回值类型参见下文)

- `zenlinkProtocol_getAllAssets`:

  查询Zenlink Module中的所有资产ID。通常有映射的资产和LP token

    ```bash
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
              "chain_id": 200,
              "module_index": 8
          },
          {
              "asset_index": 1,
              "chain_id": 300,
              "module_index": 9
          }
      ],
      "id": 1
  }
  ```

- `zenlinkProtocol_getBalance`:

  查询资产余额

  - params[0]: AssetId。包括了本链的资产和映射的资产
  - params[1]: 查询账户
    
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
     "jsonrpc":"2.0",
     "id":1,
     "method":"zenlinkProtocol_getBalance",
     "params": [{"chain_id": 200,"module_index": 2, "asset_index": 0 }, "5H9dcB3Z4NYrpiDLYXghUZoJWdPWaFoPimBwuuncK9hBaBWA"]
   }'  
  ```
- `zenlinkProtocol_getAllPairs`：

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
  - reserve1: 交易池中token1的数量
  - token0 & token1: 组成交易对的两种资产
  - totalLiquidity：lptoken 总量
  - lpAssetId: 交易中的LP token的 AssetId
  
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
        {
            "account": "5EYCAe5kj35jpW98CjJD3uDVvYH3fzm9qeT43mzL6AaStHqH",
            "holdingLiquidity": "0x0",
            "lpAssetId": {
                "asset_index": 1,
                "chain_id": 300,
                "module_index": 9
            },
            "reserve0": "0x3d4",
            "reserve1": "0x3d4",
            "token0": {
                "asset_index": 0,
                "chain_id": 200,
                "module_index": 8
            },
            "token1": {
                "asset_index": 0,
                "chain_id": 300,
                "module_index": 2
            },
            "totalLiquidity": "0x3d4"
        }
    ],
    "id": 1
  }
  ```

- `zenlinkProtocol_getOwnerPairs`：

  查询某个账户拥有的交易对
  
  - params: 查询账户
  
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getOwnerPairs",
        "params": ["5ExnkKkHG1xfgoCLjQ2DTHBkRxdoUrsnCsVorWKPsESrtpmd", null]
  }'
  ```
  
  **Reponse:**
  
  - account:交易对的账户
  - holdingLiquidity: 此账户持有的lptoken量
  - reserve0: 交易池中token0的数量
  - reserve1: 交易池中token1的数量
  - token0 & token1: 组成交易对的两种资产
  - totalLiquidity：lptoken 总量
  - lpAssetId: 交易中的LP token的 AssetId
  
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      {
        "account": "5EYCAe5kj35jpW98CjJD3uDVvYH3fzm9qeT43mzL6AaStHqH",
        "holdingLiquidity": "0x3d4",
        "lpAssetId": {
          "asset_index": 1,
          "chain_id": 300,
          "module_index": 9
        },
        "reserve0": "0x3d4",
        "reserve1": "0x3d4",
        "token0": {
          "asset_index": 0,
          "chain_id": 200,
          "module_index": 8
        },
        "token1": {
          "asset_index": 0,
          "chain_id": 300,
          "module_index": 2
        },
        "totalLiquidity": "0x3d4"
      }
    ],
    "id": 1
  }
  ```
  
- `zenlinkProtocol_getPairByAssetId`：

  获取指定资产ID的交易对信息
  
  - params: (200,8,0)(300,2,0)和组成交易对
  
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
        "jsonrpc":"2.0",
        "id":1,
        "method":"zenlinkProtocol_getPairByAssetId",
        "params": [{"chain_id": 200,"module_index": 8, "asset_index": 0 }, 
                   {"chain_id": 300,"module_index": 2, "asset_index": 0 }]
  }'
  ```

  **Response**
    - account:交易对的账户
    - holdingLiquidity:持有的LPtoken。实际上，由于lp token全部由交易者持有，这里通常为0.
    - reserve0: 交易池中token0的数量
    - reserve1: 交易池中token1的数量
    - token0 & token1: 组成交易对的两种资产
    - totalLiquidity：lptoken总量
    - lpAssetId: 交易中的LP token的 AssetId
    
  ```json
  {
        "jsonrpc": "2.0",
        "result": {
            "account": "5EYCAe5kj35jpW98CjJD3uDVvYH3fzm9qeT43mzL6AaStHqH",
            "holdingLiquidity": "0x0",
            "lpAssetId": {
                "asset_index": 1,
                "chain_id": 300,
                "module_index": 9
            },
            "reserve0": "0x3d4",
            "reserve1": "0x3d4",
            "token0": {
                "asset_index": 0,
                "chain_id": 200,
                "module_index": 8
            },
            "token1": {
                "asset_index": 0,
                "chain_id": 300,
                "module_index": 2
            },
            "totalLiquidity": "0x3d4"
        },
        "id": 1
  }
  ```
  
- `zenlinkProtocol_getAmountInPrice`： 
  
  查询买入汇率（固定交易对右边）
  
  - params[0]: "100": 买入的量
  - params[1]: 兑换路径。
  
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
       "jsonrpc": "2.0",
       "id": 0,
       "method": "zenlinkProtocol_getAmountInPrice",
       "params": [100,[{"chain_id": 200,"module_index": 8, "asset_index": 0 }, 
                       {"chain_id": 300,"module_index": 2, "asset_index": 0 } ], null]
   }'  
    ```
  
  **Response:**
  - result: 99226799： 表示的是用100000000个（200,8,0）兑换出99226799个(300,2,0)。
    
  ```json
  {
        "jsonrpc": "2.0",
        "result": "0x2f8a597",
        "id": 0
  }
  ```
  
- `zenlinkProtocol_getAmountOutPrice`：
  
  查询卖出汇率（固定交易对左边）
  
  - params[0]: "100000000": 卖出的量
  - params[1]: 交易路径。
  
  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
       "jsonrpc": "2.0",
       "id": 0,
       "method": "zenlinkProtocol_getAmountOutPrice",
       "params": [100,[{"chain_id": 200,"module_index": 8, "asset_index": 0 }, 
                       {"chain_id": 300,"module_index": 2, "asset_index": 0 } ], null]
   }'  
  ```

    **Response:**
    ```json
    {
        "jsonrpc": "2.0",
        "result": "0x2f87",
        "id": 0
    }
    ```
    
- `zenlinkProtocol_getEstimateLptoken`:

  ```bash
  curl -H "Content-Type: application/json" http://localhost:11111 -d \
  '{
       "jsonrpc":"2.0",
       "id":1,
       "method":"zenlinkProtocol_getEstimateLptoken",
       "params": [{"chain_id": 200,"module_index": 8, "asset_index": 0 }, 
                  {"chain_id": 300,"module_index": 2, "asset_index": 0 }, 
                  1000000000000000,  100000000, 0, 0 ]
   }'  
  ```
  
  **Response:**
 ```json
    {
        "jsonrpc": "2.0",
        "result": "0x5f5e100",
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
    },
    "getEstimateLptoken":{
            "description": "zenlinkProtocol getEstimateLptoken",
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
          "name": "amount_0_desired",
          "type": "TokenBalance"
        },
                {
          "name": "amount_1_desired",
          "type": "TokenBalance"
        },
                {
          "name": "amount_0_min",
          "type": "TokenBalance"
        },
                {
          "name": "amount_1_min",
          "type": "TokenBalance"
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
  "PairId": "u32",
  "Pair": {
    "token_0": "AssetId",
    "token_1": "AssetId",
    "account": "AccountId",
    "total_liquidity": "TokenBalance",
    "lp_asset_id": "AssetId"
  },
  "PairInfo": {
    "token_0": "AssetId",
    "token_1": "AssetId",
    "account": "AccountId",
    "total_liquidity": "TokenBalance",
    "holding_liquidity": "TokenBalance",
    "reserve_0": "TokenBalance",
    "reserve_1": "TokenBalance",
    "lp_asset_id": "AssetId"
  },
  "AssetId": {
    "chain_id": "u32",
    "module_index": "u8",
    "asset_index": "u32"
  },
  "TokenId": "u32",
  "AssetProperty":{
    "_enum": {
        "FOREIGN": null,
        "LP": "LpProperty"
      }
  },
  "LpProperty": {
    "token_0": "AssetId",
    "token_1": "AssetId"
  },
  "TokenBalance": "u128"
}
```
