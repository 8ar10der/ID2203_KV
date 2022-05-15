# A Key-Value In-Memory Store Software Based on Omni-Paxos Protocol

## How to run server

```shell
# Two node example
cargo run --bin server -- --pid 1 --peers 2
cargo run --bin server -- --pid 2 --peers 1
```

### Modify configs

Please enter `configs` folder, you can change the port number, the client address and the enable the debug mode by change the data in the code.

## How to run client

```shell
cargo run --bin client
```

## How to run tests

```shell
# Please ensure server node is already fresh  reboot
cargo test
```
