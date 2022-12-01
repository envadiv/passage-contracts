# Passage Contracts

Passage smart contracts written in CosmWasm and deployed to Juno.

## Diagram

![Screen Shot 2022-09-29 at 3 06 37 PM](https://user-images.githubusercontent.com/6496257/193121168-9a5f52a5-4447-4732-9cea-caefc455063e.png)

## Contracts

### Mainnet (juno-1)

| Contract                | Code ID | Version      | Checksum                                                         | Cost           | Notes |
| ----------------------- | ------- | ------------ | ---------------------------------------------------------------- | -------------- | ----- |
| marketplace-legacy      | 89      |              |                                                                  | 0.840371juno   |       |
| whitelist               | 477     | v0.1.8-alpha | d851a25fa640692739e9d4b2e255905b3e6414e00232a134438081ca497aef74 | 2.598308ujuno  |       |
| pg721-metadata-onchain  | 478     | v0.1.8-alpha | be256a2235558736018a0bdb4ee9d5c80bf2a73f4411e29be771debe6c6b1c58 | 8.000106ujuno  |       |
| minter-metadata-onchain | 480     | v0.1.8-alpha | 3f8e3db3b53c8bd4229f22263842beb3477fa1350228f08e71d4c49bca33e5d6 | 4.0814907ujuno |       |
| marketplace-legacy      | 490     | v0.1.8-alpha | 46bc19eea386551aa58267c9844d4b1b77b32ac535326cae2897733ab610b35c | 3.360571ujuno  |       |
| pg721-metadata-onchain  | 1486    | ...          | 9587d0722d6a3ad19aa6623632ddc4b92592b70f7c6869fa754a3eeedbcad900 | 0.560861ujuno  |       |
| minter-metadata-onchain | 1489    | ...          | 598cc1f1a102e121d785bc3c58075892ef8f3f781c6084b5bf97ff628a888353 | 0.269856ujuno  |       |

### Testnet (uni-5)

| Contract                | Code ID | Version      | Checksum                                                         | Notes                                                 |
| ----------------------- | ------- | ------------ | ---------------------------------------------------------------- | ----------------------------------------------------- |
| marketplace-v2          | 682     | v0.1.9-alpha | 960f3460f66b3f786a71ebeac873bfb626296e816b4959576a2f31db9bc758fe | Pre-audit release                                     |
| pg721                   | 684     | v0.1.9-alpha | 858ad95e058eec1be83b03852ea7b60c8dfae67169927844f7633a1c911396c3 | Redeploy on uni-5                                     |
| marketplace-legacy      | 685     | v0.1.9-alpha | 6ac6a166f673930a274a919cdbf80fdaefdd2c0a9e491fa3cd8248334872271d | Redeploy on uni-5                                     |
| minter                  | 686     | v0.1.9-alpha | fb6e7b06536eed2283c5000300b43ffdd9b26665c5af68627a18b6c28a072d14 | Redeploy on uni-5                                     |
| auction-english         | 687     | v0.1.9-alpha | 32e9df3bbc4e15f7ae5d7be6d731ef8c58f0f70c59c8a34a4de914669393d99a | Pre-audit release                                     |
| nft-vault               | 688     | v0.1.9-alpha | 69dd5d7ed6c33a267fbe851554b5f781f2cbcc0c8a500ff2407db852440a1dc6 | Initial release                                       |
| minter-metadata-onchain | 2770    | v0.1.9-alpha | f3fb80a6764c803e6c313e78c89db8c64358e6522a96624fd0325f821b9e40f2 | Migration test (legacy version)                       |
| pg721-metadata-onchain  | 2806    | v0.1.9-alpha | 31aba788d98137b12d8690de852075a74c6516914a5dd64cf892b21db8614370 | Migration test (legacy version)                       |
| minter-metadata-onchain | 2899    | ...          | 42c2696a30581cfc8bca753e0add69a3356df8240ce316d17dca8aec7831d824 | Migration test (updated version, optimization failed) |
| pg721-metadata-onchain  | 2900    | ...          | 9587d0722d6a3ad19aa6623632ddc4b92592b70f7c6869fa754a3eeedbcad900 | Migration test (updated version)                      |
| minter-metadata-onchain | 2902    | ...          | 42c2696a30581cfc8bca753e0add69a3356df8240ce316d17dca8aec7831d824 | Migration test (updated version)                      |

| minter-metadata-onchain | 2805 | ... | d65ec2bd67f076b345c655483c888fb309f5ff52ed25074901a3f83add473a3a | Migration test (new version, not used) |
| pg721-metadata-onchain | 2822 | ... | 9587d0722d6a3ad19aa6623632ddc4b92592b70f7c6869fa754a3eeedbcad900 | Migration test |
| minter-metadata-onchain | 2823 | ... | a7433be1cdae32effea78c3826c107273371b40cb49ac1dd4d0abb75786f1d11 | Migration test (new version) |

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
| marketplace-v2          | 2121    | v0.1.9-alpha | 260f4ac512975897fcd448c7a8cb0d4513c1c922b7041884b1c5d56701119281 | Marketplace-v2 including Auctions                     |

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
junod tx wasm store artifacts/minter_metadata_onchain.wasm  --from juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e --chain-id=uni-5 \
  --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -b block -y
```

## Keys

- Tasio testnet address: `juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e`

junod tx wasm migrate juno1wzv8xr8qc4jtamqtjtj6te70vqvlzalnqrk4k78efhf0v9ufwzfqvltkdm 2805 '{"num_mintable_tokens":5000}' --from juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e --chain-id=uni-5 --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -b block -y

junod tx wasm set-contract-admin juno1wzv8xr8qc4jtamqtjtj6te70vqvlzalnqrk4k78efhf0v9ufwzfqvltkdm juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e --from juno19mmkdpvem2xvrddt8nukf5kfpjwfslrs7sgw8e --chain-id=uni-5 --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -b block -y
