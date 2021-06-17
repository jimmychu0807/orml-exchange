# ORML Workshop

This workshop will give a brief overview on [ORML](https://github.com/open-web3-stack/open-runtime-module-library), and show two examples of how to integrate ORML into your own Substrate runtime development. One is enabling multi-currency handling, and another one is adding NFT feature in Substrate runtime.

This workshop is modified from [xlc/orml-workshop](https://github.com/xlc/orml-workshop).

## Introduction to ORML (Open Runtime Module Library)

This is an open source project coordinated by Acala team with different contributors from the community. It is consisted of a series pallets that can be integrated to suit developer purposes, including:

- `orml-currencies`: enabling multi-currency support, implemented `BasicCurrency*` and `MultiCurrency*` traits. They extended `pallet-assets` functionality.

- `orml-tokens`: This is where the multi-currency balance is defined. It extends the functionality of `pallet-balances`.

- `orml-nft`: enabling the helper functions of NFT creation and NFT transfer, but does not define any extrinsics.

- `orml-oracle`: defines the logic for oracle operator of getting external data and writing it back on-chain.

- `orml-vesting`: define functionality for performing asset vesting to user accounts.

Let's look at two examples below.

## ORML Example #1：Multi-currency Exchange

### Preface

A simple way to implement an exchange is to create an order book. Those who want to exchange can submit an order (submit\_order) specifying how much of token X I want to exchange for token Y. Now when another user look at the order list and spot an order he is interested, he can take the order (take_order) on-chain. A transaction occur and X and Y assets change hands.

Now Substrate [`pallet-balances`](https://substrate.dev/rustdocs/v3.0.0-monthly-2021-05/pallet_balances/index.html) only support a single currency. We need to extend `pallet-balances` to make user accounts link to multiple currencies.

### Pallet Extrinsics

- `submit_order(from, from_currency_id, from_currency_qty, to_currency_id, to_currency_qty)`
- `take_order(from, order_id)`
- `cancel_order(from, order_id)`

### Data Structure and Storage

Order struct

```rust
pub struct Order<T: Config> {
  pub owner:      T::AccountId,
  pub from_cid:   CurrencyIdOf<T>,
  pub from_bal:   BalanceOf<T>,
  pub to_cid:     CurrencyIdOf<T>,
  pub to_bal:     BalanceOf<T>,
  pub status:     OrderStatus,
  pub executed_with: Option<T::AccountId>,
  pub created_at: T::BlockNumber,
  pub cancelled_at: Option<T::BlockNumber>,
  pub executed_at: Option<T::BlockNumber>,
}
```

OrderStatus struct

```rust
pub enum OrderStatus {
  Pending,
  Alive,
  Executed,
  Cancelled,
  Invalid,
}
```

Storage

```rust
pub(super) type Orders<T> = StorageMap<_, Blake2_128Concat, OrderId, Order<T>>;
pub(super) type UserOrders<T> = StorageMap<_, Blake2_128Concat, AccountOf<T>, Vec<OrderId>>;
```

### Actual Implementation

- [`pallets-exchange`](pallets/exchange/src/lib.rs)

  - mainly using the helper functions of `reserve()`，`repatriate_reserved()`，and `transfer()` of `orml-currency`.
  - The multi-currency balance is stored in the storage defined by `orml-tokens` pallet.


- [Runtime](runtime/src/lib.rs)

  - The multiple currencies are actually defined in runtime `CurrencyId` enum.
  - The runtime implements `orml_tokens::Config` and `orml_currencies::Config` trait.
  - `orml_currencies` gains access to `orml_tokens` functions via `MultiCurrency` associated type.
  - `pallet_exchange` gains access to `orml_currencies` functions via `Currency` associated type.

This example demonstrates how one pallet is loosely-coupled to another pallet to access the function call of another pallet.

## ORML Example #2：NFT

### Preface

NFT has became very hot recently. Two of the most common spec to implement NFT are [ERC 721 spec](https://eips.ethereum.org/EIPS/eip-721) and [ERC 1155 spec](https://eips.ethereum.org/EIPS/eip-1155). And ERC 1155 spec is more general than ERC 721 spec that it supports both fungible assets and non-fungible tokens.

We will look at `orml-nft` that is similar to the ERC 721 spec.

### `orml-nft` function calls

The following functions are available for pallet to call but we need to define our extrinsics ourselves.

- `create_class(owner, class_metadata, class_data)` - create an NFT class
- `destroy_class(owner, class_id)` - destroy an NFT class
- `mint(owner, class_id, token_metadata, token_data)` - create an NFT token
- `burn(owner, (class_id, token_id))` - burn an NFT token
- `transfer(from, to (class_id, token_id))` - transfer an NFT token
- `is_owner(owner, (class_id, token_id))` - check if a certain token is owned by the user.

### Integrated with Runtime

- [Runtime](runtime/src/lib.rs)

  ```rust
  impl orml_nft::Config for Runtime ...
  ```

- In the runtime, we define `ClassId`, `TokenId`, `ClassData`, `TokenData`, `MaxClassMetadata`, `MaxTokenMetadata`.

- The item implementation is defined at [`runtime/src/items.rs`](runtime/src/items.rs)。

- There is a `items_genesis` function, that is called in [`node/src/chain_spec.rs`](node/src/chain_spec.rs) to initialize the genesis block in testnet.

- [`pallets/items`](pallet/items/src/lib.rs) define the extrinsics. To integrate it in runtime, we have the config:

  ```rust
  #[pallet::config]
  pub trait Config: frame_system::Config + orml_nft::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
  }
  ```

- This is how we call `orml_nft` functions in `pallet_items`.

  ```rust
  orml_nft::Pallet::<T>::mint(&who, cid, metadata.clone(), data)
  ```

  The way we define `pallet_items` Config trait, that it is bounded by `orml_nft`, is a tightly-coupled way for one pallet calling another pallet functions. This also means that the runtime also need to implement `orml_nft` Config when implementing this pallet config.

- If we want to use polkadot-js App to interact with the chain, remember to load the custom types, stored in [here](runtime/types.json), in `Settings > Developers`.

## Conclusion

In this workshop we go through ORML library and demonstrate how you can integrate some of its pallet into your runtime.

Be aware that if you are using ORML, please ensure the Substrate version it depends on matches exactly as the Substrate version your chain depends on. Otherwise there will be some weird errors.This is also why I forked the `orml` repo and used it as the dependency instead of having the project linking directly to the repo under [`open-web3-stack`](https://github.com/open-web3-stack/open-runtime-module-library).
