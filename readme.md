# Rust CLI Tool - Training Session
### This program sends a specified amount of uosmo to a destination wallet
* Compile your project: `cargo build`
* For documentation, run: `cargo doc --open`

### Before running this tool, follow these steps:
1. Go to [Levana Testnet](https://testnet-trade.levana.finance/).
2. Connect a wallet:
   - I recommend **Keplr**. Set up the wallet.
   - Now you need to create an environment variable named `mnemonic`.    
     This variable should contain the secret key for your wallet.
     ```bash
     export mnemonic="place your secret wallet key or wont be valid for the program"
     ```
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