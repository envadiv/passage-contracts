# Tesnet Contracts

| Contract  | Code ID | Commit Hash                              | Checksum                                                         |
| --------- | ------- | ---------------------------------------- | ---------------------------------------------------------------- |
| cw2981    | 787     | ed41690bc8cf64dd7d81daf235352ad1add3d7dd | 2f692fb6a6cda49f46447a7d4515126617ab0e819e8735296933b7fa3f7dc3de |
| whitelist | 786     | ed41690bc8cf64dd7d81daf235352ad1add3d7dd | eebc92ddd8008994579a65bc2958b91264a8ada216ecbd2be8fc1ef363240cb8 |

# Commands

**Deploy to testnet**

```bash
junod tx wasm store artifacts/<contract>.wasm  --from <address> --chain-id=<uni-2> \
  --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -b block -y
```

# Addresses

- Tasio testnet address: `juno1z0pmpswx9evje48nkkswhzme62llkc2r22ex2m`
