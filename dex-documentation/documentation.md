# Iroha2 DEX Technical Documentation

## Glossary
DEX - Decentralized Exchange\
ISI - Iroha Special Instruction

## Overview

DEX is implemented as an optional module of Iroha 2 and can be enabled in compile time. It introduces new core ISI's and Queries in order to support interaction with DEX and liquidity sources. According to Iroha 2 architecture, there are domains to which set of Accounts and Assets are linked. Following this approach there could be multiple DEX'es registered in the system, but at most one for each domain. This does not limit exchange capabilities, as DEX in one domain could register token pairs with assets of different domains. Current version of DEX requires all pairs (and therefore associated pool reserves) to contain base asset - which is XOR in order to mitigate liquidity provision issues. Users of exchange do not need to control this details, as api both in queries and swaps provides an abstraction, automatically building paths through base token between arbitrary tokens.

## Permissions

Interaction with DEX can be logcially seen in 3 types: Management ISI's, User ISI's and Queries.
Management ISI's require special permissions to operate since they act upon behavior and parameters of DEX, underlying token pairs and liquidity sources. Those permissions are `CanInitializeDEX` and `CanManageDEX(DEX_ID)`, former belongs to system owner account and is used to initialize new DEX in a domain also indicating particular DEX owner account which receives latter permission allowing to call Management ISI's.
User ISi's do not require special permissions but only general permissions needed to control their assets (e.g. transfers).
Queries are not permissioned in given version, allowing any account to receive information about DEX.

## Implemented features

*Management ISI's:*
- **InitializeDEX** - create DEX in Domain, set DEX owner account.
- **CreateTokenPair** - create pair A-B.
- **CreateLiquiditySource** - create source for pair, e.g. XYKPool for pair A-B.
- **SetFeeOnXYKPool** - set fee deduced from XOR part during swaps, default is 30 basis points.
- **SetProtocolFeePartOnXYKPool** - set protocol fee deduced from regular fee, and sent to protocol owner account, e.g. DEX owner. Disabled by default.

*User ISI's:*
- **AddLiquidityToXYKPool** - deposit tokens into pool: deposit A, deposit B, receive Pool Tokens.
- **RemoveLiquidityFromXYKPool** - burn Pool Tokens, receive A and B.
- **SwapExactTokensForTokensOnXYKPool** - exchange, deposit A receive B, indicate desired amount for A.
- **SwapTokensForExactTokensOnXYKPool** - exchange, deposit A receive B, indicate desired amount for B.

NOTE: Behavior for **RemoveTokenPair** and **RemoveLiquiditySource** is currently unspecified. Removal of associated entities should induce return of locked liquidity and removal of pool tokens from liquidity providers to avoid misuse if pool is re-initialized.

*Queries:*
- **GetDEX**, **GetDEXList** - get info about particular DEX, list initialized DEX'es.
- **GetTokenPair**, **GetTokenPairList**, **GetTokenPairCount** - get info about TokenPairs.
- **GetXYKPoolInfo** - get info about state of particular XYK Pool.
- **GetFeeOnXYKPool**, **GetProtocolFeePartOnXYKPool** - get info about current fees.
- **GetPriceForInputTokensOnXYKPool** - get price of output token, if exact input amount is specified.
- **GetPriceForOutputTokensOnXYKPool** - get price of input token,  if exact output amount is specified.
- **GetOwnedLiquidityOnXYKPool** - specified amount of pool tokens, get corresponding amounts of pair tokens.

*Debug ISI's*
- **AddTransferPermissionForAccount** - allow user to transfer own tokens.

## Code Structure

Core logic of DEX is contained in a `dex` rust module (`dex.rs`) in `iroha` crate. That approach was done in order to keep iroha, which avoids being monolithic with all features, isolated from dex. Although some minor proposals were accepted into iroha core because of new functionality requirements, but they are of general use.

`iroha/src/dex.rs`:
- (lines 12-267) Entities: DEX, TokenPair, LiquiditySource, XYKPoolData
- (lines 268-3199) `isi` module:
    - (lines 268-427) ISI's definition
    - (lines 428-600) DEX management logic
    - (lines 601-1717) `xyk_pool` module with implementation for pools logic
    - (lines 1718-1855) helper functions for transfers/burn/mint
    - (lines 1856-3199) `tests` module with all dex-related tests
- (lines 3200-3860) `query` module
    - (lines 3200-3319) inner helper functions for internal queries, e.g. 
    - (lines 3320-3860) structs representing data passed in queries, used in `query.rs`

`iroha/src/query.rs`
- Iroha Queries definitions with DEX extensions

`iroha_client_cli/src/main.rs`:
- Iroha core client features
- `dex` module with DEX-specific syntax definition

## Entities

All Entities consist of Data struct and separate Id struct because mostly identification is a composite of other identification structs. There are following structures in DEX:

- `DEX` struct contains data such as owner account, base token and registered token pairs. Identification is a domain name.
- `TokenPair` contains registered liquidity sources. Indentification is a containing DEX Id and pair of assets.
- `LiquiditySource` contains data related to particular source type. Identification is a containing TokenPair Id and liquidity source type. 
- `LiquiditySourceData` enum, used in LiquiditySource, wraps concrete liquidity source data struct.
- `LiquiditySourceType` enum with all liquidity source types, used in api's and id's.
- `XYKPoolData` one of liquidity source data structs, contains pool info such as fee values, current reserve amounts, pool token definition and reserve storage account id.

Top-level DEX object is stored in Domain struct (Iroha Core), which itself is stored in WorldStateView. Therefore DEX data ultimately is stored in WorldStateView which ensures consensus reducing need for manual checks given that data-changing operations are performed via ISI's mechanism and queries via Iroha Queries.

Scheme:

<center><img src="imgs/iroha_dex_data_scheme.png"></center>

## Control Flow Examples

### Adding/ Liquidity

Firstly `AddLiquidityToXYKPool` ISI is constructed with parameters. When invoked, associated `execute()` function is run leading to logic execution: `get_optimal_deposit_amounts()` is called to determine if given amounts are good for existing proportions, if Ok then tokens are transferred via `transfer_from()` and pool tokens are minted via `mint_pool_token_with_fee`. 

### Swapping Tokens (with desired input)

Firstly `SwapExactTokensForTokensOnXYKPool` ISI is constructed with parameters. When invoked, associated `execute()` function is run leading to logic execution: `get_amounts_out()` goes through tokens in provided path, querying individual pools of corresponding pairs, thus resulting in a swap amounts chain containing input amount, all intermediary pool amounts and fees, and output amount. This chain is passed to `swap_tokens_execute()` which transfers caller tokens into first pool and initiates chain of swaps via `swap_all()` and underlying `swap()`. Final swap output is redirected to recepient account, e.g. original sender.  


## Math (XYK Pool)

Math of the pool is based on classic model, with a modification of fee deduction - fee is not deduced from fixed direction, e.g. always input tokens, but rather from fixed base token, i.e. always from XOR part of exchange. This is done to provide basis for further model development, i.e. alternative fee usage scenarios.

### Adding Liquidity for initial provider (when pool is empty)
X and Y are amounts of pair tokens deposited, minimum liquidity is permanently locked (this is needed to reduce possibility of attack which results in pool being too expensive for small providers to add liquidity). Here resulting amount of Pool Tokens is derived via sqrt of reserves product, this is done to mitigate dependency between pool token quantity and actual price of either of pair tokens (dependency should be on both pair tokens, rather than only one because they present different value).

<center><img src="imgs/add_liquidity_initial.png"></center>

### Adding Liquidity for further providers
X and Y are amounts of pair tokens deposited. After initial proportions were set, further deposits need to follow them. Amount of pool tokens is also derived from existing proportions:

<center><img src="imgs/add_liquidity_secondary.png"></center>

### Swap Tokens with Desired Input amount (input is XOR)
If input (user) token is XOR, output (target) amount is derived as follows:

<center><img src="imgs/swap_input_amount_input_base.png"></center>

>where y_out output tokens go from reserve to user, x_in - fee*x_in input tokens go from user to reserve and fee*x_in input tokens go from user to fee-storage.


### Swap Tokens with Desired Input amount (output is XOR)
If output (target) token is XOR, output (target) amount is derived as follows:

<center><img src="imgs/swap_input_amount_output_base.png"></center>

>where y_out output tokens go from of reserve to user, x_in input tokens go from user to reserve and fee*y1 output tokens go from reserve to fee-storage. 


### Swap Tokens with Desired Output amount (input is XOR)
If input (user) token is XOR, input (user) amount is derived as follows:

<center><img src="imgs/swap_output_amount_input_base.png"></center>

>where y_out output tokens go from reserve to user, x_in * (1 - fee) input tokens go from user to reserve and fee*x_in input tokens go from user to fee-storage.


### Swap Tokens with Desired Output amount (output is XOR)
If output (target) token is XOR, input (user) amount is derived as follows:

<center><img src="imgs/swap_output_amount_output_base.png"></center>

>where y_out output tokens go from of reserve to user, x_in input tokens go from user to reserve and y1-y_out output tokens go from reserve to fee-storage. 

## Build and Run

### Requirements:
- linux / unix-based OS
- docker, docker-compose
- rust environment, e.g. latest stable toolchain via [rustup](https://rustup.rs)
- CWD is iroha project root, e.g. `./projects/iroha/`

### Run tests with detailed logs of operations
```shell
cd ./iroha
cargo test --features "dex" -- --nocapture --test-threads=1
# or single exact test
cargo test test_xyk_pool_add_liquidity_should_pass --features "dex" -- --nocapture --test-threads=1
```
Tests output with detailed logs are provided as `tests.log` file for convenience.

### Run tests without logs
```shell
cd ./iroha
cargo test --features "dex"
```

Local testnet could be run with docker.

1. Build Iroha with DEX
```shell
cd ./iroha
cargo build --features "dex"
cd ..
docker-compose build
docker-compose up
# after use
docker-compose down
```
2. Build client

```shell
cd ./iroha_client_cli
cargo build
cd ..
mkdir client_test
cp ./target/debug/iroha_client_cli ./client_test/iroha_client_cli
cp ./iroha_client_cli/config.json ./client_test/config.json
```

## Tests

Tests are also contained in DEX module, comprised of unit-tests for critical components and integration tests to represent feature usecases. In order to make test cases compact and readable, an approach of incrementally modular tests was used. This means that there are full unit tests starting with basic features such as DEX init, creating token pairs, creating accounts, creating liquidity sources, and TestKit module with compact alternatives which are reused in further tests which depend more on existing state.

Currently existing tests:

1. `test_initialize_dex_should_pass`
    - tries to initialize DEX with appropriate account, then performs and validates query to retrieve new dex by id and query listing all dexes. 
2. `test_initialize_dex_should_fail_with_permission_not_found`
    - tries to initialize DEX with newly created account without any permissions, checks correctness of returned error and performs query listing all DEX'es which should remain empty.
3. `test_create_and_delete_token_pair_should_pass`
    - tries to register token pair with correct assets (XOR for base, other for target) for initialized DEX (with XOR as base), checks that after registration query by id returns correct pair, listing returns newly added pair and after removal listing query returns empty.
4. `test_xyk_pool_create_should_pass`
    - for initialized DEX and registered token pair, tries to create liquidity source with type of XYK Pool, queries the pool and checks that it contains default values for it's data store.
5. `test_xyk_pool_add_liquidity_should_pass`
    - for initialized DEX, registered token pair and registered liquidity source, creates new account with minted tokens and tries to invoke AddLiquidity ISI via it's identity, queries and checks pool state and state of created account.
6. `test_xyk_pool_remove_liquidity_should_pass`
    - for initialized DEX, registered token pair and registered liquidity source, creates new account with minted tokens, invokes AddLiquidity and RemoveLiquidity ISI's via it's identity, queries and checks pool state and state of created account.
7. `test_xyk_pool_optimal_liquidity_should_pass`
    - tests edge cases for low level function which given amounts of pair tokens, limits and pool reserves, calculates for existing pool propotion if it's possible to add liquidity with given amounts fulfilling limits or otherwise return error.
8. `test_xyk_pool_quote_should_pass`
    - tests edge cases for low level function which given amount of token and pair reserves, calculates equivalent amount of another token.
9. `test_xyk_pool_swap_assets_in_should_pass`
    - for initialized DEX, registered token pair, registered liquidity source and added liquidity to pool, tries to exchange tokens with desired input amount, queries and checks pool state, pool reserves account and trader account.
10. `test_xyk_pool_get_target_amount_out_should_pass`
    - tests edge cases for function calculating price for tokens given pool reserves (variant with desired input amount, when input token is XOR)
11. `test_xyk_pool_get_base_amount_out_should_pass`
    - tests edge cases for function calculating price for tokens given pool reserves (variant with desired input amount, when output token is XOR)
12. `test_xyk_pool_swap_assets_out_should_pass`
    - for initialized DEX, registered token pair, registered liquidity source and added liquidity to pool, tries to exchange tokens with desired output amount, queries and checks pool state, pool reserves account and trader account.
13. `test_xyk_pool_get_base_amount_in_should_pass`
    - tests edge cases for function calculating price for tokens given pool reserves (variant with desired output amount, when input token is XOR)
14. `test_xyk_pool_get_target_amount_in_should_pass`
    - tests edge cases for function calculating price for tokens given pool reserves (variant with desired output amount, when output token is XOR)
15. `test_xyk_pool_two_liquidity_providers_one_trader_should_pass`
    - uses all components of TestKit providing complete usecase, where dex is initialized, two token pairs are registered
16. `test_xyk_pool_get_price_should_pass`
    - for initialized entifies, adds liquidity to pool and queries price in 4 variants: XOR->DOT with desired input and output, DOT->XOR with desired input and output. 
17. `test_xyk_pool_get_owned_liquidity_should_pass`
    - for initialized entities, add liquidity to pool via 2 new accounts and queries and checks amounts of tokens they can claim by burning received pool tokens.



## CLI client

As an interaction example, cli client of Iroha2 was extended to support new DEX-specific syntax and logic also completing required helpers to access needed core features. New syntax is shown via demo usecase:

### Prepare

In order to run demo, a specific setup is needed. Firstly Iroha2 (with DEX enabled) needs to be running in docker. This is discussed in `Build and Run` section.

Secondly an executable of `iroha_client_cli` needs to be used, refer to `Build and Run` in order retrieve it.
Demo configs could be found in `demo` directory shipped with this documentation, for simplicity, built `iroha_client_cli` should be copied multiple times according to scheme:

```
/demo
    /root
        iroha_client_cli
        swap_setup_env_v2.sh
        config.json
    /user_a
        iroha_client_cli
        config.json
    /user_b
        iroha_client_cli
        config.json
    /user_c
        iroha_client_cli
        config.json
```
### Cli demo

### Step 1 - Initialize Environment
With a root identity initialize DEX, create token pairs and liquidity sources. Also create accounts, and mint initial tokens for them. This could be done either manually, as shown further.
**Or via `swap_setup_env_v2.sh` script (cwd should be in /demo/root directory)**
```shell
cd /demo/root
# via root identity
./iroha_client_cli domain add --name="Soramitsu"
./iroha_client_cli domain add --name="Polkadot"
./iroha_client_cli domain add --name="Kusama"
./iroha_client_cli asset register --domain="Soramitsu" --name="XOR"
./iroha_client_cli asset register --domain="Polkadot" --name="DOT"
./iroha_client_cli asset register --domain="Kusama" --name="KSM"
./iroha_client_cli account register --domain="Soramitsu" --name="DEX Owner" --key="[120, 221, 193, 217, 83, 191, 157, 223, 1, 2, 205, 104, 209, 1, 180, 200, 29, 70, 220, 189, 221, 136, 221, 64, 31, 12, 44, 39, 179, 57, 141, 181]" 
./iroha_client_cli dex initialize --domain="Soramitsu" --owner_account_id="DEX Owner@Soramitsu" --base_asset_id="XOR#Soramitsu"
./iroha_client_cli dex token_pair --domain="Soramitsu" create --base_asset_id="XOR#Soramitsu" --target_asset_id="DOT#Polkadot"
./iroha_client_cli dex token_pair --domain="Soramitsu" create --base_asset_id="XOR#Soramitsu" --target_asset_id="KSM#Kusama"
./iroha_client_cli dex xyk_pool --domain="Soramitsu" --base_asset_id="XOR#Soramitsu" --target_asset_id="DOT#Polkadot" create
./iroha_client_cli dex xyk_pool --domain="Soramitsu" --base_asset_id="XOR#Soramitsu" --target_asset_id="KSM#Kusama" create
## setup User A Account
./iroha_client_cli account register --domain="Soramitsu" --name="User A" --key="[162, 172, 183, 13, 229, 237, 8, 113, 177, 22, 100, 41, 174, 202, 106, 25, 216, 241, 18, 226, 77, 138, 250, 103, 10, 16, 194, 56, 21, 198, 90, 148]"
./iroha_client_cli asset mint --account_id="User A@Soramitsu" --id="XOR#Soramitsu" --quantity="12000"
./iroha_client_cli asset mint --account_id="User A@Soramitsu" --id="DOT#Polkadot" --quantity="4000"
./iroha_client_cli asset mint --account_id="User A@Soramitsu" --id="KSM#Kusama" --quantity="3000"
./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="XOR#Soramitsu"
./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="DOT#Polkadot"
./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="KSM#Kusama"
./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="XYKPOOL XOR-Soramitsu/DOT-Polkadot#Soramitsu"
./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="XYKPOOL XOR-Soramitsu/KSM-Kusama#Soramitsu"
## setup User B Account
./iroha_client_cli account register --domain="Soramitsu" --name="User B" --key="[171, 23, 228, 169, 169, 132, 244, 86, 72, 152, 12, 41, 160, 86, 186, 81, 54, 241, 116, 40, 246, 106, 252, 36, 114, 156, 121, 228, 213, 136, 109, 153]"
./iroha_client_cli asset mint --account_id="User B@Soramitsu" --id="XOR#Soramitsu" --quantity="500"
./iroha_client_cli asset mint --account_id="User B@Soramitsu" --id="DOT#Polkadot" --quantity="500"
./iroha_client_cli account add_transfer_permission --id="User B@Soramitsu" --asset_id="XOR#Soramitsu"
./iroha_client_cli account add_transfer_permission --id="User B@Soramitsu" --asset_id="DOT#Polkadot"
./iroha_client_cli account add_transfer_permission --id="User B@Soramitsu" --asset_id="XYKPOOL XOR-Soramitsu/DOT-Polkadot#Soramitsu"
## setup User C Account
./iroha_client_cli account register --domain="Soramitsu" --name="User C" --key="[196, 239, 3, 91, 95, 202, 55, 187, 149, 152, 2, 30, 178, 165, 167, 193, 45, 239, 205, 216, 185, 213, 155, 161, 92, 147, 242, 254, 27, 112, 199, 189]"
./iroha_client_cli asset mint --account_id="User C@Soramitsu" --id="KSM#Kusama" --quantity="2000"
./iroha_client_cli account add_transfer_permission --id="User C@Soramitsu" --asset_id="KSM#Kusama"
```


### Step 2 - User A adds liquidity to pools XOR-DOT and XOR-KSM
State of User Account and Pool Reserve Accounts could be checked (e.g. before and after operation) via:
```shell
# CWD must be /demo/user_a
./iroha_client_cli account get --domain="Soramitsu" --name="User A"
./iroha_client_cli account get --domain="Soramitsu" --name="STORE XYKPOOL XOR-Soramitsu/DOT-Polkadot"
./iroha_client_cli account get --domain="Soramitsu" --name="STORE XYKPOOL XOR-Soramitsu/KSM-Kusama"
```

To perform liqudity operation use:
```shell
# CWD must be /demo/user_a
./iroha_client_cli dex xyk_pool --domain="Soramitsu" --base_asset_id="XOR#Soramitsu" --target_asset_id="DOT#Polkadot" add_liquidity --base_amount="6000" --target_amount="4000"

./iroha_client_cli dex xyk_pool --domain="Soramitsu" --base_asset_id="XOR#Soramitsu" --target_asset_id="KSM#Kusama" add_liquidity --base_amount="6000" --target_amount="3000"
```

### Step 3 - User B adds liqudity to pool XOR-DOT
State of User Account and Pool Reserve Account could be checked (e.g. before and after operation) via:
```shell
# CWD must be /demo/user_b
./iroha_client_cli account get --domain="Soramitsu" --name="User B"
./iroha_client_cli account get --domain="Soramitsu" --name="STORE XYKPOOL XOR-Soramitsu/DOT-Polkadot"
```

To perform liqudity operation use:
```shell
# CWD must be /demo/user_b
./iroha_client_cli dex xyk_pool --domain="Soramitsu" --base_asset_id="XOR#Soramitsu" --target_asset_id="DOT#Polkadot" add_liquidity --base_amount="500" --target_amount="500"
```

### Step 4 - User C swaps KSM for DOT
State of User Account and Pool Reserve Accounts could be checked (e.g. before and after operation) via:
```shell
# CWD must be /demo/user_c
./iroha_client_cli account get --domain="Soramitsu" --name="User C"
./iroha_client_cli account get --domain="Soramitsu" --name="STORE XYKPOOL XOR-Soramitsu/DOT-Polkadot"
./iroha_client_cli account get --domain="Soramitsu" --name="STORE XYKPOOL XOR-Soramitsu/KSM-Kusama"
```

Perform swap with input token quantity:
```shell
# CWD must be /demo/user_c
./iroha_client_cli dex xyk_pool_swap --domain="Soramitsu" --path="[\"KSM#Kusama\", \"XOR#Soramitsu\", \"DOT#Polkadot\"]" --input_amount="2000"
```

### Step 5 - User B removes liquidity from pool XOR-DOT
State of User Account and Pool Reserve Accounts could be checked via:
```shell
# CMD must be /demo/user_b
./iroha_client_cli account get --domain="Soramitsu" --name="User B"
./iroha_client_cli account get --domain="Soramitsu" --name="STORE XYKPOOL XOR-Soramitsu/DOT-Polkadot"
```

Perform liquidity operation with:
```shell
# CMD must be /demo/user_b
./iroha_client_cli dex xyk_pool --domain="Soramitsu" --base_asset_id="XOR#Soramitsu" --target_asset_id="DOT#Polkadot" remove_liquidity --liquidity="407"
```

All commands for this CLI Usecase can be also found in `/demo/swap_tokens_xyk_pool_v2.txt`.