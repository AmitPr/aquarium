# aquarium

Aquarium is a toolkit and monorepo framework for building, deploying, and managing Cosmwasm smart contracts.

## Installation

```bash
cargo install --git https://github.com/AmitPr/aquarium.git
```

This should add the `aq` binary to a folder on your path.

## Usage

Print help:

```bash
aq
```

**Initialize a new project:**

```
> aq init --help
Initialize a new project

Usage: aq init [OPTIONS] <NAME>

Arguments:
  <NAME>

Options:
  -d, --dir <dir>
  -h, --help       Print help
```

Without specifying a directory (such as `.`), the project will be initialized in a new folder with the same name as the project name. This will also create a `scripts` crate in the project root. This is intended to be the crate in which you will write your deployment and management scripts.

## `Aquarium.toml`

The `Aquarium.toml` file is the configuration file for your project. It is used to specify the contracts in your project, as well as the deployment and management scripts.

```toml
project = <project_name> # The name of your project
scripts_path = "scripts" # The path to the scripts crate, can be changed if wanted.
hd_path = "m/44'/118'/0'/0/0" # The derivation path to use for accounts defined in this project.
default_network = "devnet" # The default network to use for commands that require a network.

[networks.devnet] # A network definition
chain_id = "harpoon-4" # The chain id of the network
lcd_addr = "http://localhost:1317" # The LCD address of the network
gas_price = 0.00125 # The gas price to use for transactions on this network
gas_adjustment = 1.25 # How much to pad the gas estimates by
gas_denom = "ukuji" # The gas denom to use for transactions on this network
account_prefix = "kujira" # The account prefix to use for transactions on this network

... # More networks can be defined here

[accounts.from_mnemonic] # defined an account called "from_mnemonic"
mnemonic = <mnemonic> # The mnemonic to use for this account

[accounts.from_env] # defined an account called "from_env"
env = <env_var_name> # The environment variable to use as the source for the account mnemonic

... # More accounts can be defined here
```

Aquarium will automatically load a `.env` file in the project root, and can use mnemonics defined in environment variables from any source. This is the recommended way to store mnemonics for accounts that you do not want to commit to source control.

## Writing scripts

The `scripts` crate is where you will write your deployment and management scripts. The `scripts` crate is a normal Rust crate, and can be used to write any Rust code you want. However, it is intended to be used for writing `aquarium` scripts that interact with your contracts.

To set up this crate, first add the `aquarium` crate as a dependency in your `scripts/Cargo.toml`:

```toml
[dependencies]
aquarium = { ... } # Most likely the git dependency specification, but perhaps from crates.io in the future.
```

We will be defining multiple binaries, each of which will be a separate script. To do this, we will use the `[[bin]]` entries that can be added to `Cargo.toml`. For example, to define a script called `example-script`:

```toml
[[bin]]
name = "example-script"
path = "src/example-script.rs"
```

Then in our `src/example-script.rs` file, we can write our script. This example script will store a code ID, and instantiate a dummy contract from it:

```rust
use aquarium::{Env, Executor, Querier, ContractInstance};
use aquarium::utils::{parse_code_id, parse_instantiated_address};
#[aquarium::task]
async fn deploy_vault(env: &mut Env) {
    // Get the bytecode from the compiled contract
    let bytecode = include_bytes!("../../artifacts/dummy-optimized.wasm");
    let hash = env
        .executor
        .store_code(bytecode.to_vec(), Some("tx memo".to_string()))
        .await
        .unwrap();
    println!("Waiting for storecode hash {hash}");
    let receipt = env.executor.wait_for_transaction(hash).await.unwrap();
    let code_id = parse_code_id(&receipt).unwrap();
    println!("Storecode code id {code_id}");
    env.refs.add_code_id("dummy", code_id);
    let instantiate_msg = dummy::InstantiateMsg {
        param: "test".to_string(),
        ... // Other fields
    };
    let hash = env
        .executor
        .instantiate(
            code_id,
            &instantiate_msg,
            vec![],
            Some("dummy label".to_string()),
            Some("admin".to_string()),
            Some("tx memo".to_string()),
        )
        .await
        .unwrap();
    println!("Waiting for instantiate hash {hash}");
    let receipt = env.executor.wait_for_transaction(hash).await.unwrap();
    let contract_addr = parse_instantiated_address(&receipt).unwrap();
    println!("Instantiated contract address {contract_addr}");
    // Add the contract instance to the environment refs
    let mut instance = ContractInstance::new(code_id, contract_addr);
    instance.attrs.insert("some attribute".to_string(), json!("We can use arbitrary JSON here!"));
    env.refs.add_contract_instance("dummy", instance);
}
```

As you can see, aquarium provides a number of utilities for interacting with contracts, as well as a `Env` object that contains an `executor`, for signing and broadcasting transactions, a `querier`, for querying the network, a `refs` object that should be used to keep track of contract deployments, code IDs, and other useful information.

## Running scripts

To run a script, use the `aq task` command:

```
> aq task --help
Run tasks/scripts

Usage: aq task <COMMAND>

Commands:
  list  List all tasks
  run   Run a task
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

Running `aq task list` will list all tasks defined in the `scripts` crate:

```
> aq task list
Available tasks:
  - "example-script"
```

Then we can run this task with `aq task run example-script`. We can change the account and network used as follows:

```
> aq task run example-script -- --account from_env --network mainnet
```

## Contract Refs

The `refs` object is actually saved to a file in the project root called `contracts.json`, which stores a JSON object with the following structure:

```json
{
  "networks": {
    "devnet": {},
    "testnet": {
      "contract-name": {
        "code_ids": [1, 2, 3, ...],
        "instances": [
          {
            "code_id": 1,
            "address": "kujira1example",
            "any_other_attributes": "that you want to store",
            "here": ["can be stored", "as JSON"]
          },
          ... // More instances
        ]
      },
      ... // More contract types
    },
    ... // More networks that have been deployed to
  }
}
```