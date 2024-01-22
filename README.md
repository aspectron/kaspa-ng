# `Kaspa NG`

[<img alt="github" src="https://img.shields.io/badge/github-aspectron/kaspa--ng-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/aspectron/kaspa-ng)
<img src="https://img.shields.io/badge/platform-native-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/badge/platform-wasm32-informational?style=for-the-badge&color=50a0f0" height="20">
<img src="https://img.shields.io/github/actions/workflow/status/aspectron/kaspa-ng/ci.yaml?style=for-the-badge" height="20">

<p align="center" style="margin:32px auto 0px auto;text-align:center;font-size:10px;color:#888;">
<img src="https://aspectron.com/images/projects/kaspa-ng-screen-01.png" style="display:block;max-height:320px;max-width:524px;width:524px;height:auto;object-fit:cover;margin: 0px auto 0px auto;"><br/><sup>RUSTY KASPA P2P NODE &bull; KASPA WALLET &bull; BLOCKDAG VISUALIZER</sup></p>

<p align="center" style="margin:4px 0px;text-align:center;font-size:10px;color:#800;">
&bull; ALPHA RELEASE &bull;
</p>

## Features

This software incorporates the following functionality:
- Rusty Kaspa p2p Node
- Kaspa wallet based on the Rusty Kaspa SDK
- Rusty Kaspa CLI wallet
- BlockDAG visualizer
- Remote node connectivity

This project is built on top of and incorporates the [Rusty Kaspa](https://github.com/kaspanet/rusty-kaspa) core framework.

This software is ideological in nature with a strong focus on architecture and decentralization. It is a unified codebase tightly coupled with the Rusty Kaspa project. Fully written in Rust, it is available as a high-performance desktop application on all major operating systems (Windows, Linux and MacOS) as well as in major web browsers. It does not rely on any JavaScript or Web frameworks, which greatly strengthens its security profile. The Web Browser extension based on this infrastructure is currently under development.

## Releases

- You can obtain the latest binary redistributables from the [Releases](https://github.com/aspectron/kaspa-ng/releases) page.
- You can access the official Web App online at [https://kaspa-ng.org](https://kaspa-ng.org).

## Building

To build this project, you need to be able to build Rusty Kaspa. If you have not built Rusty Kaspa before, please follow the Rusty Kaspa [build instructions](https://github.com/kaspanet/rusty-kaspa/blob/master/README.md).

In addition, on linux, you need to install the following dependencies:

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

## License

Licensed under a [PROPRIETARY MIT-style Open Source LICENSE](LICENSE) with the following restrictions: 
_You are expressly prohibited from using, adapting, or integrating this software into any cryptocurrency network or related technology other than the specified intended network for which it is developed - The Kaspa BlockDAG cryptocurrency network._

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any
additional terms or conditions.

## Donations

If you are a Kaspa investor, please consider supporting this project. The funds will be used to cover operational costs and further the project's functionality. 

`kaspa:qq2efzv0j7vt9gz9gfq44e6ggemjvvcuewhzqpm4ekf4fs5smruvs3c8ur9rp`
