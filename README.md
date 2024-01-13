# kaspa-ng

_ALPHA RELEASE - This project is work in progress and and has been pre-released for testing purposes only._
_It is not intended for production use at this time._

### Desktop p2p node and wallet for the Kaspa BlockDAG Network

This project is built on top of an incorporates the [Rusty Kaspa](https://github.com/kaspanet/rusty-kaspa) core framework.

This software is ideological in nature with a strong focus on architecture and decentralization. It is a unified codebase tightly coupled with the Rusty Kaspa project. Fully written in Rust, it runs as a high-performance desktop application on all major operating systems (Windows, Linux and MacOS) as well as in major web browsers through the magic of WebAssembly. It does not rely on any JavaScript or Web frameworks, which greatly strengthens its security profile. It can also run as a mobile application and a browser extension, albeit these components are currently under development.

Since this software is built on top of the Rusty Kaspa Core Wallet framework, it is fully compatible with any applications developed on top of the Kaspa Core Wallet framework including the Rusty Kaspa WASM32 framework that is usable in web browsers and NodeJS environments.

With Kaspa-ng you can run a full node and a wallet on your desktop as well as connect to remote/public nodes. This functionality makes the wallet infrastructure immune to any potential DDoS attacks on the public nodes.

### Building

To build this project, you need to be able to build Rusty Kaspa. If you have not built Rusty Kaspa before, please follow the Rusty Kaspa [build instructions](https://github.com/kaspanet/rusty-kaspa/blob/master/README.md).

In addition, on linux, you need to perform the following installs:

#### Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install libglib2.0-dev libatk1.0-dev libgtk-3-dev librust-atk-dev
```

#### Fedora:
```bash
sudo dnf install glib2-devel atk-devel gtk3-devel
```

Once you have Rusty Kaspa built, you will be able to build and run this project as follows:

```bash
cargo run --release
```

## License

Licensed under a [PROPRIETARY MIT-style Open Source LICENSE](LICENSE) with the following restrictions: 
_You are expressly prohibited from using, adapting, or integrating this software into any cryptocurrency network or related technology other than the specified intended network for which it is developed - The Kaspa BlockDAG cryptocurrency network._

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any
additional terms or conditions.
