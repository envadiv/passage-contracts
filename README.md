# Passage Contracts

Passage smart contracts written in CosmWasm and deployed to Juno.

## Diagram

![Screen Shot 2022-04-29 at 1 24 18 PM](https://user-images.githubusercontent.com/6496257/165993415-0ca10d74-f875-47b6-b85e-00928bbd3f7a.png)

## Contracts

### Testnet (uni-3)

| Contract                | Code ID | Version      | Checksum                                                         | Notes                            |
| ----------------------- | ------- | ------------ | ---------------------------------------------------------------- | -------------------------------- |
| pg721                   | 67      | v0.1.2-alpha | f5f6bf30ccdaadfa440036437600ac3a98999cc4707f20a2b1e80842563e4384 |                                  |
| minter                  | 68      | v0.1.2-alpha | c018628958d0d4e169ece7d415eda4840a29a8a7ddde0ea1f62153cd72a764e4 |                                  |
| royalty-group           | 69      | v0.1.2-alpha | 6c86f2f1f37eab7b1c2e94f4718e4f8449e5d094f5b8dbb5a96f6c2f000e45ba |                                  |
| marketplace-legacy      | 70      | v0.1.2-alpha | c166e8c6060da4bdf30e42126afc3c08128f59fca65ba73c1c70400284a5145e |                                  |
| marketplace-v2          | 212     | v0.1.3-alpha | defd21d5f150d7744f71af2d2b934171968bc5b7f8396ea9041acd71e4fc9012 | Initial marketplace-v2 deploy    |
| minter                  | 256     | v0.1.4-alpha | 292dc2a924e56d393a922f2f694503863293f2f173896fa0afd2b42b4ef53a11 | Generates pseudorandom token ids |
| whitelist               | 257     | v0.1.4-alpha | be256a2235558736018a0bdb4ee9d5c80bf2a73f4411e29be771debe6c6b1c58 |                                  |
| pg721-metadata-onchain  | 364     | v0.1.5-alpha | d851a25fa640692739e9d4b2e255905b3e6414e00232a134438081ca497aef74 |                                  |
| minter-metadata-onchain | 365     | v0.1.5-alpha | 0dc2c5230d044b76ebab90e7d95b9a63970c871f7376c68eee68df2c9410bd03 |                                  |

### Testnet (uni-2)

| Contract           | Code ID | Version      | Checksum                                                         | Notes                                   |
| ------------------ | ------- | ------------ | ---------------------------------------------------------------- | --------------------------------------- |
| pg721              | 829     | v0.1.0-alpha | f5f6bf30ccdaadfa440036437600ac3a98999cc4707f20a2b1e80842563e4384 |                                         |
| whitelist          | 830     | v0.1.0-alpha | e62979c720855cac3cccb9026beaee806490a2655e17a3d88febfdd441d30297 |                                         |
| minter             | 831     | v0.1.0-alpha | c018628958d0d4e169ece7d415eda4840a29a8a7ddde0ea1f62153cd72a764e4 |                                         |
| royalty-group      | 832     | v0.1.0-alpha | bee354500a63c0e4c43fccb5ffc2a83e62da08f32af40c7e7b010d24817d7ae0 |                                         |
| royalty-group      | 878     | v0.1.1-alpha | 6c86f2f1f37eab7b1c2e94f4718e4f8449e5d094f5b8dbb5a96f6c2f000e45ba | reworked distribute method              |
| legacy-marketplace | 1029    | v0.1.2-alpha | c166e8c6060da4bdf30e42126afc3c08128f59fca65ba73c1c70400284a5145e | includes admin NFT registration bug fix |

## Commands

**Deploy to testnet**

```bash
junod tx wasm store artifacts/minter_metadata_onchain.wasm  --from juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e --chain-id=uni-3 \
  --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -b block -y
```

## Keys

- Tasio testnet address: `juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e`
