use std::{io::{self, Write}, str::FromStr};

// File I/O for better error handling
use fs_err;

// Error handling
use anyhow::{anyhow, Context, Result};

// Json Response
use serde_json::json;

// Parse input
use clap::Parser;
#[derive(Parser, Debug)]
struct Transaction {
    /// Amount to send to another wallet, e.g. 110uosmo
    coin: cosmos::ParsedCoin,
    /// Destination address to receive the funds
    destination: cosmos::Address,
}

#[tokio::main]
async fn main() -> Result<()> {

    println!("Rust Cli Tool has started");

    // If some wrong format is detected will panic
    let transaction = Transaction::parse();

    // Execute the transaction
    let transaction_details = execute_transaction(&transaction)
        .await
        .context("Error encountered during transaction execution")?;

    // All good
    if transaction_details["code"] == 0 {
        println!(
            "Transaction completed successfully: {}",
            transaction_details
        );
        Ok(())
    } else {
        // Error
        let error_message = format!("Transaction finalized with errors: {}", transaction_details);
        writeln!(io::stderr(), "{}", error_message).unwrap();
        Err(anyhow!(
            "Transaction finalized with errors {}",
            transaction_details
        ))
    }
}

/// Executes a transaction on the Cosmos blockchain.
///
/// This function performs the following steps:
/// 1. Connects to the Cosmos blockchain.
/// 2. Retrieves the balance for the given Cosmos wallet address.
/// 3. Loads the wallet from the file located at `wallet/wallet.key`.
/// 4. Executes the specified transaction by sending the specified token amount to the destination address.
///
/// ### Arguments
/// * `transaction` - A reference to a [`Transaction`] object containing the transaction details,
///   including the recipient address, token, and amount to be transferred.
///
/// ### Returns
/// Returns a tuple containing:
/// * `TxResponse::code` - A `u32` representing the transaction response code (0 for success, non-zero for failure).
/// * `TxResponse::height` - An `i64` representing the block height at which the transaction was included.
/// * `TxResponse::tx_hash` - A `String` representing the transaction hash, useful for tracking the transaction.
///
/// ### Errors
/// This function may return an error in the following cases:
/// - If it fails to connect to the Cosmos blockchain.
/// - If the `wallet.key` file does not exist or cannot be read.
/// - If the mnemonic phrase in the `wallet.key` file is invalid or cannot be loaded.
/// - If the transaction fails during execution.
///
async fn execute_transaction(transaction: &Transaction) -> Result<serde_json::Value> {
    // Connect to the blockchain
    println!("Connecting to Cosmos...");
    let cosmos_addr = cosmos::CosmosNetwork::OsmosisTestnet
        .connect()
        .await
        .context("Error connecting to Cosmos")?;
    println!("Connection successful.");

    // Get the address
    let address = transaction.destination;

    // Get balance
    println!("Getting balance for wallet {}", address);

    let balances = cosmos::Cosmos::all_balances(&cosmos_addr, address)
        .await
        .context("Failed to retrieve all balances for the Cosmos address")?;

    // Iterate over all balances for each
    balances.iter().for_each(|balance| {
        // Check if this denom is the one by task
        if balance.denom.contains(&address.to_string()){
            // Show and record info
            println!("Balance: {}", balance.amount);
        }
    });

    println!("Executing transaction....");

    // Vec which contains the Coin to send => 100 uosmo
    // Convert from ParseCoin to cosmos::Coin, ParseCoin fields are private
    let coin: cosmos::Coin = transaction.coin.clone().into();
    let amount: Vec<cosmos::Coin> = vec![coin];

    // Load the wallet
    // Check if wallet exists.
    if fs_err::metadata("wallet/wallet.key").is_err() {
        println!("The path 'wallet/wallet.key' does not exist");
        println!("Please follow the steps outlined in the readme.md file");
        return Err(anyhow!(
            "Can not find the 'wallet.key' file in the path: 'wallet/wallet.key'"
        ));
    }

    // Get the wallet
    // Read the wallet key
    let wallet_key =
        fs_err::read_to_string("wallet/wallet.key").context("Error reading the wallet.key file")?;

    // Get Mnemonic
    let mnemonic = cosmos::SeedPhrase::from_str(&wallet_key)
        .context("Failed to retrieve the mnemonic phrase")?;

    // Get wallet from Mnemonic
    let wallet = mnemonic
        .with_hrp(cosmos::AddressHrp::from_static("osmo"))
        .context("Error identifying the wallet")?;

    // Show and record wallet which should match with your
    // Wallet addr in https://testnet-trade.levana.finance/
    println!("Sender Wallet address: {}", wallet);

    // Destination Wallet
    println!("Destination Wallet address: {}", address);

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
