# Passage Contracts

Passage smart contracts written in CosmWasm and deployed to Juno.

## Diagram

![Screen Shot 2022-06-16 at 11 52 41 AM](https://user-images.githubusercontent.com/6496257/174113657-dbe6819a-bee7-401f-b9ff-b95049d3ea26.png)

## Contracts

### Mainnet (juno-1)

| Contract                | Code ID | Version      | Checksum                                                         | Cost           | Notes |
| ----------------------- | ------- | ------------ | ---------------------------------------------------------------- | -------------- | ----- |
| marketplace-legacy      | 89      |              |                                                                  | 0.840371juno   |       |
| whitelist               | 477     | v0.1.8-alpha | d851a25fa640692739e9d4b2e255905b3e6414e00232a134438081ca497aef74 | 2.598308ujuno  |       |
| pg721-metadata-onchain  | 478     | v0.1.8-alpha | be256a2235558736018a0bdb4ee9d5c80bf2a73f4411e29be771debe6c6b1c58 | 8.000106ujuno  |       |
| minter-metadata-onchain | 480     | v0.1.8-alpha | 3f8e3db3b53c8bd4229f22263842beb3477fa1350228f08e71d4c49bca33e5d6 | 4.0814907ujuno |       |
| marketplace-legacy      | 490     | v0.1.8-alpha | 46bc19eea386551aa58267c9844d4b1b77b32ac535326cae2897733ab610b35c | 3.360571ujuno  |       |

### Testnet (uni-3)

| Contract                | Code ID | Version      | Checksum                                                         | Notes                                                 |
| ----------------------- | ------- | ------------ | ---------------------------------------------------------------- | ----------------------------------------------------- |
| pg721                   | 67      | v0.1.2-alpha | f5f6bf30ccdaadfa440036437600ac3a98999cc4707f20a2b1e80842563e4384 |                                                       |
| minter                  | 68      | v0.1.2-alpha | c018628958d0d4e169ece7d415eda4840a29a8a7ddde0ea1f62153cd72a764e4 |                                                       |
| royalty-group           | 69      | v0.1.2-alpha | 6c86f2f1f37eab7b1c2e94f4718e4f8449e5d094f5b8dbb5a96f6c2f000e45ba |                                                       |
| marketplace-legacy      | 70      | v0.1.2-alpha | c166e8c6060da4bdf30e42126afc3c08128f59fca65ba73c1c70400284a5145e |                                                       |
| marketplace-v2          | 212     | v0.1.3-alpha | defd21d5f150d7744f71af2d2b934171968bc5b7f8396ea9041acd71e4fc9012 | Initial marketplace-v2 deploy                         |
| minter                  | 256     | v0.1.4-alpha | 292dc2a924e56d393a922f2f694503863293f2f173896fa0afd2b42b4ef53a11 | Generates pseudorandom token ids                      |
| whitelist               | 257     | v0.1.4-alpha | be256a2235558736018a0bdb4ee9d5c80bf2a73f4411e29be771debe6c6b1c58 |                                                       |
| pg721-metadata-onchain  | 364     | v0.1.5-alpha | d851a25fa640692739e9d4b2e255905b3e6414e00232a134438081ca497aef74 |                                                       |
| minter-metadata-onchain | 390     | v0.1.5-alpha | 4bb6c74f35cf8bacb6b631420578032b1abdddacc0bb557f20ebbcbefb9f5d8f |                                                       |
| minter-metadata-onchain | 556     | v0.1.6-alpha | dfb92c39d2332a6ec83df262c4e18d621bd4cc9702dc08b81e66945c69a353fb | Removed base_token_uri config var                     |
| pg721_legacy            | 557     | v0.1.6-alpha | 84f04434f55a73096e908b093e75153bfb637eb9091a7b267f7a516ad36ad49c | For deploying Town 1 to testnet                       |
| minter-metadata-onchain | 801     | v0.1.7-alpha | f6bfc05ba1d5ea2dedabe3bae69ddc5d00dba6c032f3bc078821daf476a9d133 | Refactored MintInfo query, added SetAdmin execute msg |
| minter-metadata-onchain | 866     | v0.1.8-alpha | 3f8e3db3b53c8bd4229f22263842beb3477fa1350228f08e71d4c49bca33e5d6 | Added recipient to Withdraw msg                       |

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

**Deploy to mainnet**

```bash
junod tx wasm store artifacts/marketplace_legacy.wasm  --from juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e --chain-id=juno-1 --node https://rpc.juno-1.deuslabs.fi:443 --gas-prices 0.1ujuno --gas auto --gas-adjustment 1.3 -b block
```

**Deploy to testnet**

```bash
junod tx wasm store artifacts/minter_metadata_onchain.wasm  --from juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e --chain-id=uni-3 \
  --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -b block -y
```

## Keys

- Tasio testnet address: `juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e`
