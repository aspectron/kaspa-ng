# kaspa-ng

_ALPHA RELEASE - This project is work in progress and and has been pre-released for testing purposes only._
_It is not intended for production use at this time._

### Desktop p2p node and wallet for the Kaspa Network

This project is built on top of an incorporates the [Rusty Kaspa](https://github.com/kaspanet/rusty-kaspa) core framework.

This software is ideological in nature with strong focus on the architecture and decentralization. It is a unified codebase tightly coupled with the Rusty Kaspa project. It is written 100% in Rust. It runs as a high-performance desktop application on all major platforms (Windows, Linux and MacOS) as well as in major web browsers through the magic of WebAssembly. It does not rely on any JavaScript or Web frameworks, which greatly reduces its security profile. It can also run as a mobile application and a browser extension, albeit these components are currently under development.

Since this software is built on top of the Rusty Kaspa Core Wallet framework, it is fully compatible with any applications developed on top of the Kaspa Core Wallet framework including Rusty Kaspa WASM32 framework usable in web browsers and NodeJS environments.

### Building

To build this project, you need to be able to build Rusty Kaspa. If you have not built Rusty Kaspa before, please follow the Rusty Kaspa [build instructions](https://github.com/kaspanet/rusty-kaspa/blob/master/README.md).

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
