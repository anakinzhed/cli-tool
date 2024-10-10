/// SeedPhrase::from_str
use std::str::FromStr;

/// Error handling
use anyhow::{anyhow, Context, Result};

/// Json Response
use serde_json::json;

/// Parse input
use clap::Parser;

#[derive(Parser, Debug)]
struct Transaction {
    /// Amount to send to another wallet, e.g. 110uosmo
    coin: cosmos::ParsedCoin,
    /// Destination address to receive the funds
    destination: cosmos::Address,
    /// Capture environment variable mnemonic
    #[clap(env = "mnemonic")]
    origin: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Init subscriber to handle traces
    tracing_subscriber::fmt::init();

    tracing::info!("Rust Cli Tool has started");

    // If some wrong format is detected will panic
    let transaction = Transaction::parse();

    // Execute the transaction
    let transaction_details = execute_transaction(&transaction)
        .await
        .context("Error encountered during transaction execution")?;

    // All good
    if transaction_details["code"] == 0 {
        tracing::info!(
            "Transaction completed successfully: {}",
            transaction_details
        );
        Ok(())
    } else {
        // Error
        tracing::error!("Transaction finalized with errors: {}", transaction_details);
        Err(anyhow!(
            "Transaction finalized with errors {}",
            transaction_details
        ))
    }
}

/// Executes a transaction on the Osmosis Testnet.
///
/// This function performs the following steps:
/// 1. Connects to Osmosis Testnet
/// 2. Retrieves the balance for the provided address.
/// 3. Loads a wallet using a mnemonic phrase obtained from the `transaction.origin` field.
/// 4. Sends the specified token amount to the destination address using the provided transaction details.
///
/// ### Arguments
/// * `transaction` - A reference to a [`Transaction`] struct containing the transaction details:
///   - `coin`: The amount to transfer, in the form of a [`ParsedCoin`], e.g., "110uosmo".
///   - `destination`: The wallet address that will receive the funds.
///   - `origin`: The mnemonic phrase for the wallet from which the funds will be sent, captured from the environment variable `mnemonic`.
///
/// ### Returns
/// Returns a `serde_json::Value` containing:
/// * `code` - A `u32` representing the transaction response code (0 indicates success, non-zero indicates failure).
/// * `height` - An `i64` representing the block height where the transaction was included.
/// * `tx_hash` - A `String` representing the transaction hash, useful for tracking the transaction on the blockchain.
///
/// ### Errors
/// This function may return an error in the following cases:
/// - If there is a failure connecting to the Cosmos blockchain
/// - If the balance retrieval for the provided address fails
/// - If the mnemonic phrase provided in the `origin` field is invalid or cannot be parsed
/// - If the transaction execution fails

async fn execute_transaction(transaction: &Transaction) -> Result<serde_json::Value> {
    // Connect to the blockchain
    tracing::info!("Connecting to Osmosis Testnet...");
    let cosmos_addr = cosmos::CosmosNetwork::OsmosisTestnet
        .connect()
        .await
        .context("Error connecting to Osmosis Testnet")?;
    tracing::info!("Connection successful.");

    // Get the address
    let address = transaction.destination;

    // Get balance
    tracing::info!("Getting balance for wallet {}", address);

    // Get all balances
    let balances = cosmos::Cosmos::all_balances(&cosmos_addr, address)
        .await
        .context("Failed to retrieve all balances for the Cosmos address")?;

    // Iterate over all balances for each
    balances.iter().for_each(|balance| {
        // Check if this denom is the one by task
        if balance.denom.contains(&address.to_string()) {
            // Show and record info
            tracing::info!("Balance: {}", balance.amount);
        }
    });

    tracing::info!("Executing transaction....");

    // Vec which contains the Coin to send => 100 uosmo
    // Convert from ParseCoin to cosmos::Coin, ParseCoin fields are private
    let coin: cosmos::Coin = transaction.coin.clone().into();
    let amount: Vec<cosmos::Coin> = vec![coin];

    // Load the wallet
    // Get mnemonic from environment variable
    let mnemonic = transaction.origin.clone();

    // Get Mnemonic
    let seedphrase = cosmos::SeedPhrase::from_str(&mnemonic)
        .context("Failed to retrieve the mnemonic phrase")?;

    // Get wallet from Mnemonic
    let wallet = seedphrase
        .with_hrp(cosmos::AddressHrp::from_static("osmo"))
        .context("Error identifying the wallet")?;

    // Show and record wallet which should match with your
    // Wallet addr in https://testnet-trade.levana.finance/
    tracing::info!("Sender Wallet address: {}", wallet);

    // Destination Wallet
    tracing::info!("Destination Wallet address: {}", address);

    // Execute transaction
    let result = wallet
        .send_coins(&cosmos_addr, address, amount)
        .await
        .context(format!(
            "Error executing the transaction at address {}",
            address
        ))?;

    // Details to json
    let transaction_details = json!({
        "code": result.code,
        "height": result.height,
        "tx_hash": result.txhash,
    });

    Ok(transaction_details)
}
