# Changelog

# 1.0.1

- Resolve issues that came up during `1.0.0` pre-release testing.

# 1.0.0

- Rusty Kaspa 1.0.0 (Crescendo support)
- Remove legacy testnet-11 support.
- Update testnet-10 to 10 BPS (visualizer & load estimation).
- Fix occasional issues detecting previously used addresses when importing legacy wallets.

# 0.3.0

- Rusty Kaspa 0.15.1 
- Add `Settings > ... > Local p2p Node .. > Public wRPC (Borsh)` to allow for external wRPC connections.
- A new priority fee estimation algorithm based on the network load (Send panel).
- Add support for legacy wallets created with [KDX](https://kdx.app) and Web Wallet at [https://wallet.kaspanet.io](https://wallet.kaspanet.io).
- The ability to choose from a list of available public nodes is no longer available (public nodes are load-balanced).
- Kaspa NG has been updated to EGUI 0.28.0, which includes various improvements and bug fixes.
- Display addresses in the transaction history panel.
- Transaction history elements are now clickable leading to the Kaspa Explorer.
- Add experimental `Passive Sync` mode that allows connecting to a public node while synchronizing the local node in the background.

# 0.2.7

- Pagination in transaction history panel
- Resolve an issue with some transactions not being displayed in the correct sort order.
- Refactor public node connectivity (now using Rusty Kaspa public node resolver).
- Add `Settings > User Interface > Options > Disable Window Frame` options that allows the user to disable custom window frame. Custom window frame currently affects ability to resize KaspaNG window on the Windows operating system.

NOTE: This release includes the underlying changes to the wRPC Borsh protocol that breaks compatibility with older versions of Kaspa nodes (all versions before `0.14.2`). This change is necessary to support future features and improvements.

# 0.2.6

- Fix an issue in WASM32 (browser) where after the wallet is loaded, even though the balance is displayed correctly, attempt to send a transaction would result in the "Insufficient funds" error.
- Fix a bug in the node info panel reversing inbound and outbound peers (native/desktop only).
- Fix a crash in the Send panel when the user enters an invalid address, some amount and then hits the enter key.

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
