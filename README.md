# Address Artisan

Address Artisan is a vanity Bitcoin P2PKH address generator based on the [BIP32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) xpub key derivation.

This software is inspired by [Senzu](https://github.com/kaiwolfram/senzu) and aims to be:

- 🔒 **Secure**: using BIP32 key derivation you can generate vanity addresses for your hardware wallet! 🤯
- ⚡ **Fast**: match recognition algorithm that does not require the checksum calculation.
- 😎 **Cool**: "1There1sNoSpoon" is much cooler than "bc1qtheresn0sp00n". P2PKH for the win! 🎉

## Get the tool

You can get the tool with:

```
cargo install address-artisan
```

or clone the repository and run:

```
cargo build --release
```

## Usage

This tool can be called with 2 required arguments (`xpub` and `prefix`), 1 optional argument (`max-depth`), and 1 optional flag (`i-am-boring`).

- `xpub`: extended public key. Can be obtained from almost any Bitcoin wallet. Example in # TODO
- `prefix`: prefix of the address to be generated. Must start with "1".
- `max-depth`: maximum depth of the last derivation path. A larger max-depth means better use of the key space and cache. However, an address may get buried in a large gap, and since [account discovery](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki#user-content-Account_discovery) is designed to be sequential, it may take a while for the wallet to find it after the gap limit is increased. After some testing, 100,000 seems to be a good value, the wallet freezes for 3 seconds, and then it’s all set.
- `i-am-boring`: if you are not a cool person and don't have friends, this flag can be used to make the tool's logs more serious (but the Artisan will be mad).

You can always call the help flag to get more information:

```
address-artisan --help
```

For a complete walkthrough, showing all steps and details, check the [Example](link-to-the-example-session) section.

## BIP44

This tool is not (and cannot be) compliant with [BIP44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki). But can be used on wallets that are BIP 44 compliant. Here is how it works:

BIP 44 standardizes 5 levels of derivation:

```
m / purpose' / coin_type' / account' / change / address_index
```
where:
- `m` is the master key
- `purpose'` is the constant 44' (0x8000002C) - respecting [BIP43](https://github.com/bitcoin/bips/blob/master/bip-0043.mediawiki)
- `coin_type'` is 0' (0x80000000) for Bitcoin
- `account'` is the account number so user can organize the funds like in a bank account. Greater than 0' (0x80000000)
- `change` 0 (0x00) for receive addresses, 1 (0x01) for change addresses
- `address_index` is the index of the address in the account 

This tool brute-forces a path of the form:

```
xpub_path' / random_number / <n derivation paths> / 0 / address_index
```

where:
- `xpub_path'` is the derivation path provided by the user (that should be hardened)
- `random_number` is a number randomly generated to get a different key space for each run. Less than 0' (0x80000000)
- `<n derivation paths>` 1 or more derivation paths that will expand when exausting the key space. Each less than 0' (0x80000000)
- `0` is the constant 0 (0x00)
- `address_index` is the index of the address in the account, less or equal to `max_depth` cli argument

By leaving the second last derivation path as 0, BIP44 compliant wallets will correctly recognize the vanity address as the `address_index`th receive address if the inputed path in the wallet is `xpub_path / radom_number / <n derivation paths>`

## Example

The example below shows from start to finish how to generate and use a vanity address with this tool. The chosen wallet is [Sparrow](https://github.com/sparrowwallet/sparrow) and the seed phrase is the satoshi legend one.

### Get the xpub

First it is required to get a working wallet for the P2PKH script. It is recommended to do this step even if a P2PKH wallet is already set, as the recommended derivation path is not the default one.

First create a new P2PKH wallet. Make sure to select `P2PKH` on the script type.

![New wallet](./assets/new_p2pkh_wallet.png)

On the next screen, choose a derivation path. It is recommended to use one for this purpose and I purpose the following:

```
m/1034543799'/0'/0'
```

- purpose 1034543799' (0xBDA9E2B7) for vanity 
- coin type 0' (0x80000000) for Bitcoin
- account 0' (0x80000000)

Make sure to use the ' after each number to make the derivation hardened.

![New wallet derivation path](./assets/new_wallet_derivation_path.png)

The next screen will show the xpub, in this case `xpub6DK1UMgy8RpXQYaE6PmRfEMf2tkTzz8wBHreDSriH5bXQb2KE4f9MzEnAMMbpoQ4HcaUyMytM7d2cBLXvtEMJXgmofNCaRh8Ah5HzwiRHLD`. Copy it to your clipboard.

![New wallet xpub](./assets/new_wallet_xpub.png)

### Generate the vanity address

Call the tool with the xpub and the desired prefix.

```
address-artisan --xpub xpub6DK1UMgy8RpXQYaE6PmRfEMf2tkTzz8wBHreDSriH5bXQb2KE4f9MzEnAMMbpoQ4HcaUyMytM7d2cBLXvtEMJXgmofNCaRh8Ah5HzwiRHLD --prefix 1Test
```

The result will be in the form of the address, a derivation path and an receive address index.

```
Address: 1TestbXeUg2HDciy2Va5gYyoN8SbL51jt
Derivation path: xpub'/335682406/36995
Receive address index: 436
```

- `xpub'`: the derivation path that was used to generate the xpub in [Get the xpub](#get-the-xpub) section, `m/1034543799'/0'/0'` if the section was followed
- The receive address index is the index of the address that goes after `/0` in the derivation path.

In this case, the full path of the address is `m/1034543799'/0'/0'/335682406/36995/0/436`.

### Import the address in the wallet

In the wallet, create a new P2PKH wallet with the same seed phrase or hardware wallet and use the derivation path the tool returned.

![New wallet derivation path](./assets/new_wallet_derivation_path.png)

To be clear, in this case the derivation path is `m/1034543799'/0'/0'/335682406/36995` 

The default gap for wallets is 20, in Advanced make sure the gap is more than the address index.

![Change gap limit](./assets/vanity_wallet_gap_limit.png)

### Confirm the address

In Addresses tab, go down to the receive address index and confirm that you have the vanity address. Do not put funds in the address without confirming that it is in your addresses list.

![Confirm the address](./assets/vanity_wallet_confirm_the_address.png)

Address confirmed, it can be used securely to receive funds. With a double click, the default Receive screen will open.

![Receive screen](./assets/vanity_wallet_receive.png)






























