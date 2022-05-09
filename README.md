# Passage Contracts

Passage smart contracts written in CosmWasm and deployed to Juno.

## Diagram

![Screen Shot 2022-04-29 at 1 24 18 PM](https://user-images.githubusercontent.com/6496257/165993415-0ca10d74-f875-47b6-b85e-00928bbd3f7a.png)

## Contracts

### Testnet

| Contract      | Code ID | Version      | Checksum                                                         | Notes                      |
| ------------- | ------- | ------------ | ---------------------------------------------------------------- | -------------------------- |
| pg721         | 829     | v0.1.0-alpha | f5f6bf30ccdaadfa440036437600ac3a98999cc4707f20a2b1e80842563e4384 |                            |
| whitelist     | 830     | v0.1.0-alpha | e62979c720855cac3cccb9026beaee806490a2655e17a3d88febfdd441d30297 |                            |
| minter        | 831     | v0.1.0-alpha | c018628958d0d4e169ece7d415eda4840a29a8a7ddde0ea1f62153cd72a764e4 |                            |
| royalty-group | 832     | v0.1.0-alpha | bee354500a63c0e4c43fccb5ffc2a83e62da08f32af40c7e7b010d24817d7ae0 |                            |
| royalty-group | 878     | v0.1.1-alpha | 6c86f2f1f37eab7b1c2e94f4718e4f8449e5d094f5b8dbb5a96f6c2f000e45ba | reworked distribute method |

## Commands

**Deploy to testnet**

```bash
junod tx wasm store artifacts/<contract>.wasm  --from juno1z0pmpswx9evje48nkkswhzme62llkc2r22ex2m --chain-id=uni-2 \
  --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -b block -y
```

## Keys

- Tasio testnet address: `juno1z0pmpswx9evje48nkkswhzme62llkc2r22ex2m`
