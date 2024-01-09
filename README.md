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

#### Running as Native App
```bash
cargo run --release
```

#### Running as Web App
```bash
cargo install trunk
trunk serve --release
```
Access via [https://localhost:8080](https://localhost:8080)

While the application is a static serve, you can not load it from the local file system due to CORS restrictions. Due to this, a web server is required. This application is designed to be built with [Trunk](https://trunkrs.dev/) and is served from the `dist/` folder.  This is a self-contained client-side application - once the application is loaded, the web server is no longer required.

#### Running as a Browser Extension

This project currently supports Chrome browser extension target, but this part of the project is under development and is not ready for use.

```bash
./build-chrome
```

### Testing

This is an alpha release, please help us test this software.

One of the best ways to test this application is to build both desktop and web versions and run them side-by-side, connecting the web app to the node running within the desktop app. You can then create a wallet in both instances and transfer funds between them.

### Known Issues

- [ ] The Web App does not currently preserve transaction history. Reloading the page or the wallet will result in the blank transaction list (this does not affect the wallet functionality).  The Web Browser transaction history storage backend is currently under development in the Rusty Kaspa wallet framework.
- [ ] When the Web App goes off screen or in the background tab, browser will suspend it preventing it from processing updates. There is a workaround for this, but it is not yet implemented.


## License

Licensed under a [PROPRIETARY MIT-style Open Source LICENSE](LICENSE) with the following restrictions: 
_You are expressly prohibited from using, adapting, or integrating this software into any cryptocurrency network or related technology other than the specified intended network for which it is developed - The Kaspa BlockDAG cryptocurrency network._

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any
additional terms or conditions.
