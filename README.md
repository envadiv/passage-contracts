# Passage Contracts

Passage smart contracts written in CosmWasm and deployed to Juno.

## Diagram

![Screen Shot 2022-04-29 at 1 24 18 PM](https://user-images.githubusercontent.com/6496257/165993415-0ca10d74-f875-47b6-b85e-00928bbd3f7a.png)

## Contracts

### Testnet

| Contract      | Code ID | Commit Hash                              | Checksum                                                         |
| ------------- | ------- | ---------------------------------------- | ---------------------------------------------------------------- |
| pg721         | -       | 108caf2b5677da186e08c17a04f45bc01a04b73a | f5f6bf30ccdaadfa440036437600ac3a98999cc4707f20a2b1e80842563e4384 |
| whitelist     | -       | 108caf2b5677da186e08c17a04f45bc01a04b73a | e62979c720855cac3cccb9026beaee806490a2655e17a3d88febfdd441d30297 |
| minter        | -       | 108caf2b5677da186e08c17a04f45bc01a04b73a | c018628958d0d4e169ece7d415eda4840a29a8a7ddde0ea1f62153cd72a764e4 |
| royalty-group | -       | 108caf2b5677da186e08c17a04f45bc01a04b73a | bee354500a63c0e4c43fccb5ffc2a83e62da08f32af40c7e7b010d24817d7ae0 |

## Commands

**Deploy to testnet**

```bash
junod tx wasm store artifacts/pg721.wasm  --from juno1z0pmpswx9evje48nkkswhzme62llkc2r22ex2m --chain-id=uni-2 \
  --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -b block -y
```

## Keys

- Tasio testnet address: `juno1z0pmpswx9evje48nkkswhzme62llkc2r22ex2m`
