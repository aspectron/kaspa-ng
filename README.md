# kaspa-ng

## Destop p2p node and wallet for Kaspa built on top of [Rusty Kaspa](https://github.com/kaspanet/rusty-kaspa) core framework

## Building

For prerequisites, please follow the Rusty Kaspa [build instructions](https://github.com/kaspanet/rusty-kaspa#getting-started).

## Running Native
```
cargo run --release
```

## Running Web
```
cargo install trunk
trunk serve
```
Access via [https://localhost:8080](https://localhost:8080)

## Browser Extension

This project currently supports Chrome browser extension target, but this component of the project is under heavy development and is not ready for use.
```
./build-chrome
```