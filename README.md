# Address Artisan

Address Artisan is a vanity Bitcoin P2PKH address generator based on the [BIP32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) xpub key derivation.

This software is inspired by [Senzu](https://github.com/kaiwolfram/senzu) and aims to be:

- 🔒 **Secure**: using BIP32 key derivation you can generate vanity addresses for your hardware wallet! 🤯
- ⚡ **Fast**: match recognition algorithm that does not require the checksum calculation.
- 😎 **Cool**: "1There1sNoSpoon" is much cooler than "bc1qtheresn0sp00n". P2PKH for the win! 🎉


 
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