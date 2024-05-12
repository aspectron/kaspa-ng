# Changelog

# 0.2.5
- Update Rusty Kaspa p2p client (kaspad) to `0.14.1`.
- WASM SDK is now available that allows developers using TypeScript and JavaScript to access and interface with wallets created using Kaspa NG and Rusty Kaspa CLI - [https://aspectron.org/en/projects/kaspa-wasm.html](https://aspectron.org/en/projects/kaspa-wasm.html)

# 0.2.4
- Add `Settings > Node > Custom Data Folder` option
- Preserve current language setting between restarts
- Add Fonts for various languages (AR,HE,JA,KR,SC)

# 0.2.3 - 2024-01-24
- Fix maximize and full-screen button handling

# 0.2.2 - 2024-01-24
- Miscellaneous updates to release processes and CI workflows

# 0.2.1 - 2024-01-22
- User Interface scale in Display settings (in addition to `Ctrl`+`+` and `Ctrl`+`-` shortcuts, `⌘` on MacOS)
- Offer alternate public node in case of random node connection failure
- Prevent saving settings when no public node is selected
- Data folder size display in Overview and management in Settings > Storage
- Fix edge conditions in the wallet when changing networks
- Support for cache management `ram-scale` option in the node configuration
- Add `--version` command line argument

# 0.2.0 - 2024-01-14
- Dedicated persistent popup notification panel for error, warning and info messages
- Various improvements to CI processes and binary redistributables generation
- Linux DEB package generation
- Custom window frame and caption bar in desktop environments
- Network load detection and automatic transaction priority fee prompt in Wallet > Send
- Random server option in connection settings
- Network and public node selection in the status bar
- Visualizer settings presets and automatic load based on the network selection

# 0.1.0 - 2023-12-27
Initial technology preview alpha release for testing with the upcoming Testnet 11. 
