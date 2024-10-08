# Rust CLI Tool - Training Session
### This program sends a specified amount of uosmo to a destination wallet

* Add dependencies in your `Cargo.toml` file.
* Compile your project: `cargo build`
* For documentation, run: `cargo doc --open`

### Before running this tool, follow these steps:
1. Go to [Levana Testnet](https://testnet-trade.levana.finance/).
2. Connect a wallet:
   - I recommend **Keplr**. Set up the wallet.
   - Save the wallet mnemonic in the root of this tool in the path `wallet/wallet.key`.
   - Click on the wallet address and you will see the Faucet.
     > **Note**: A faucet in the blockchain is a tool or service that distributes small amounts of cryptocurrency for free to users (training purposes).
3. Complete the security challenge and get some free funds.

### Running the Tool
Run the tool using the following format:
```bash
cli-tool [amount][token] [address]
```
Example:
```bash
cli-tool 1000uosmo osmoojplkwejfiuoniuwoefiuwnbeefeccvkk
```