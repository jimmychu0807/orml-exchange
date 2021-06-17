# ORML 工作坊

本工作坊对 [ORML](https://github.com/open-web3-stack/open-runtime-module-library) 作一个简单介绍。并且有两个示例展示如何把 ORML 篏入到自己开发的模块中，一个是多币种交易，另一个是 NFT 生成及转帐。

本工作坊改编自 [xlc/orml-workshop](https://github.com/xlc/orml-workshop)。

## ORML (Open Runtime Module Library) 介绍

由 Acala 主导及社区开发的开源项目，里面包含很多不同的组件，用户可因应不同场景篏入不同组件来使用这些功能。包括：

- `orml-currencies`: 提供多币种支持，实现了 `BasicCurrency*` 及 `MultiCurrency*` 的 trait. 扩充了 Substrate `pallet-assets` 的功能。

- `orml-tokens`: 把多币种功能与用户帐号挂勾。扩充了 Substrate 里 `pallet-balances` 的功能。

- `orml-nft`: 提供了创建 NFT 及 NFT 转帐的函数，但没有外部接口 (extrinsics)。

- `orml-oracle`: 提供了 oracle 模块功能让 oracle 营运者允许成员们提交链外数据到链上。

- `orml-vesting`: 提供了分期把代币逐渐返回到帐户上。

我们具体看以下两个例子。

## ORML 篏入实例 1：多币种交易

### 引言

若要實現交易所功能，最简单的做法就是建立一个订单簿 (order book)，不同用户可提交订单信息 (submit\_order)，说明他愿意用 多少 X 币来购买 Y 币。而另一用户看到后，若手持足够 Y 币，觉得兑换率合理，就可提交一个成交交易 (take\_order) 到链上。双方就此成交。

现在 Substrate [`pallet-balances`](https://substrate.dev/rustdocs/v3.0.0-monthly-2021-05/pallet_balances/index.html) 只支持单币种，所以我们需要扩展 `pallet-balances` 来使用户可与多币种挂勾。

### 外部接口

- `submit_order(from, from_currency_id, from_currency_qty, to_currency_id, to_currency_qty)`
- `take_order(from, order_id)`
- `cancel_order(from, order_id)`

### 数据结构及存储

订单结构

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

订单状态

```rust
pub enum OrderStatus {
  Alive,
  Executed,
  Cancelled,
}
```

存储方面：

```rust
pub(super) type Orders<T> = StorageMap<_, Blake2_128Concat, OrderId, Order<T>>;
pub(super) type UserOrders<T> = StorageMap<_, Blake2_128Concat, AccountOf<T>, Vec<OrderId>>;
```

### 代码具体实现

- [`pallets-exchange`](pallets/exchange/src/lib.rs)

  - 主要用到 `orml-currency` 的 `reserve()`，`repatriate_reserved()`，及 `transfer()`。
  - 而用户帐号多币种的余额则在 `orml-tokens` 的存储内。

- [Runtime](runtime/src/lib.rs)

  - 具体不同的币种是定义在 runtime `CurrencyId` enum 里。
  - runtime 实现了 `orml_tokens::Config` 及 `orml_currencies::Config` 的 trait。
  - `orml_currencies` 透过 `MultiCurrency` 来调用 `orml_tokens` 模块，而用 `NativeCurrency` 来调用原本 `pallet-balances` 内的函数。
  - 而 `pallet_exchange` 透过 `Currency` 来调用 `orml_currencies` 模块

这是一种松耦合的绑定。透过 associated type 来定义 trait bound, 而另一模块则去实现这 trait。

## ORML 实例：NFT

### 引言

最近 NFT 变得越来越红火。而实现 NFT 合约一般就是跟着 [ERC 721](https://eips.ethereum.org/EIPS/eip-721) 和 [ERC 1155](https://eips.ethereum.org/EIPS/eip-1155) 规格。ERC 1155 比 ERC 721 更具通用性，可在一张合约内同时支援 fungible assets 及 non-fungible tokens.

今天看的 orml-nft 更接近 ERC 721. 看看它有哪些接口，及如何使用。

### `orml-nft` 接口

这里的接口仅供调用模块使用，但不提供链外外部接口。

- `create_class(owner, class_metadata, class_data)` - 创建一个 NFT 类别
- `destroy_class(owner, class_id)` - 注销一个 NFT 类别
- `mint(owner, class_id, token_metadata, token_data)` - 打造一枚 NFT 代币
- `burn(owner, (class_id, token_id))` - 注销一枚 NFT 代币
- `transfer(from, to (class_id, token_id))` - 转移一枚 NFT 币
- `is_owner(owner, (class_id, token_id))` - 检查是否代币持有人

所以具体 nft 功能已经实现，但如何和你的 runtime 接入呢？

### 与 runtime 接入

- [Runtime](runtime/src/lib.rs)

  ```rust
  impl orml_nft::Config for Runtime ...
  ```

- 在 runtime 裡，我們定义了 `ClassId`, `TokenId`, `ClassData`, `TokenData`, `MaxClassMetadata`, `MaxTokenMetadata`

- 这里实际定义放在了 [`runtime/src/items.rs`](runtime/src/items.rs)。

- 里面的 `items_genesis` 函数，是在 [`node/src/chain_spec.rs`](node/src/chain_spec.rs) 里运行，用在运行测试网时初始化链数据。

- [`pallets/items`](pallet/items/src/lib.rs) 定义了链的外部调用。而它整合的方法是

  ```rust
  #[pallet::config]
  pub trait Config: frame_system::Config + orml_nft::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
  }
  ```

- 接着我們用以下方法在 `pallet_items` 裡調用 `orml_nft` 函數.

  ```rust
  orml_nft::Pallet::<T>::mint(&who, cid, metadata.clone(), data)
  ```

  这是一种紧耦合的绑定。透过 pallet `Config` 需要符合 `orml_nft` 的 trait bound 来使用这个模块。这说明要用这 pallet 的话，runtime 也需要同时加载 `orml_nft` 这个 pallet。

- 如需要用 polkadot-js App 来互动，则需要把自订义的结构加载到 `Settings > Developers` 里。这里可把 [`runtime/types.json`](runtime/types.json) 文件的内容贴上去。

## 结语

这个工作坊简单介绍了 ORML 库，及展示如何加载 ORML 内的模块到 runtime 里来支持多币种及 NFT。大家也可看看其他 ORML 的库，并使用这些已成熟实的功能助你们完成工作。

要注意一点，ORML 所指定的 Substrate 版本得与你的链的 Substrate 版本相一致，不然会有一些奇怪的错误信息。
