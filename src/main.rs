// Path for Dir log
use std::{path::Path, str::FromStr};

// File I/O
use std::fs::{read_to_string, metadata};

// Logs
use log::{info, error};
use fern::Dispatch;
use std::fs::OpenOptions; 
use chrono::Local;

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
    
    // Initialize Logging 
    setup_logging().context("Failed to init log")?;
    
    info!("Rust Cli Tool has started");

    // If some wrong format is detected will throw an Error
    let transaction = Transaction::parse();

    // Execute the transaction
    let (code, height, txhash) = execute_transaction(&transaction).await
                                                   .context("Error encountered during transaction execution")?;
    // Details to json
    let transaction_details = json!({
        "Code": code,
        "Height": height,
        "TxHash": txhash,
    });

    // All good
    if code == 0 {
        info!("Transaction completed successfully: {}", transaction_details);
    } else { // Error 
        error!("Transaction finalized with errors: {}", transaction_details);
        return Err(anyhow!("Transaction finalized with errors {}", transaction_details));
    }
   
    Ok(())
}

/// Sets up logging for the CLI tool by creating a log file and configuring the logger.
///
/// This function performs the following steps:
/// 1. Creates a directory named `logs` if it doesn't exist.
/// 2. Generates a log file with the format `cli-tool_YYYY-MM-DD_HH-MM-SS.log` based on the current date and time.
/// 3. Configures the logger to output log messages to both the console and the log file.
///
/// The log entries include the timestamp, log level, target (source of the log), and the message.
///
/// ### Returns
/// Returns `Ok(())` if the logging setup was successful, or an error if:
/// - The logs directory cannot be created.
/// - The log file cannot be opened or written to.
/// - The log configuration fails to be applied.
///
/// ### Errors
/// - If unable to create the `logs` directory, an error with the message `"Unable to create the logs directory"` will be returned.
/// - If the log file cannot be opened, an error with the message `"Failed to open log file"` will be returned.
/// - If the log configuration cannot be applied, an error with the message `"Error applying log configuration"` will be returned.

fn setup_logging() -> Result<()> {
    // Create a directory named "logs" if it doesn't exist
    std::fs::create_dir_all("logs").context("Unable to create the logs directory")?;
    
    // Log filename with the format "cli-tool_YYYY-MM-DD_HH-MM-SS.log" based on current time
    let log_filename = format!("cli-tool_{}.log", Local::now().format("%Y-%m-%d_%H-%M-%S")); 

    // Set up the logger
    Dispatch::new()
        .format(|out, message, record| {
            // Log entry with timestamp, level, target and message
            out.finish(format_args!(
                "[{}][{}] {}: {}",
                // Current date and time
                Local::now().format("%Y-%m-%d %H:%M:%S"), 
                // Log level (e.g., info, error)
                record.level(),    
                // Target (where the log came from)                      
                record.target(),      
                // Log message                   
                message                                  
            ))
        })
        // Print log messages to console
        .chain(std::io::stdout()) 
        .chain(OpenOptions::new()
            // Create the log file if it doesn't exist
            .create(true) 
            // Append new logs 
            .append(true)  
            // Open log file in the "logs" directory
            .open(Path::new("logs").join(log_filename)).context("Failed to open log file")?) 
        // Set Log level
        .level(log::LevelFilter::Info)
        // Apply the log config
        .apply().context("Error applying log configuration")?; 
    // All good
    Ok(())
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
async fn execute_transaction(transaction: &Transaction) -> Result<(u32, i64, String)> {
    
    // Connect to the blockchain
    info!("Connecting to Cosmos...");
    let cosmos_addr = cosmos::CosmosNetwork::OsmosisTestnet.connect().await
                              .context("Error connecting to Cosmos")?;
    info!("Connection successful.");
    
    // Get the address
    let address = transaction.destination;
    
    // Get balance
    info!("Getting balance for wallet {}", address);

    let balances = cosmos::Cosmos::all_balances(&cosmos_addr, address).await
                              .context("Failed to retrieve all balances for the Cosmos address")?;

    // Iterate over all balances for each 
    balances.iter().for_each(|balance| {
        // Take the field denom and split it by '/'
        let denom_slice: Vec<&str> = balance.denom.split('/').collect();
        // If was successfull, check if this denom is the one by task
        if denom_slice.len() > 2 && denom_slice[1] == address.to_string(){
            // Show and record info
            info!("Balance: {}", balance.amount);
        }
    });

    info!("Executing transaction....");

    // Vec which contains the Coin to send => 100 uosmo
    // Convert from ParseCoin to cosmos::Coin, ParseCoin fields are private
    let coin: cosmos::Coin = transaction.coin.clone().into();
    let amount: Vec<cosmos::Coin> = vec![coin];

    // Load the wallet
    // Check if wallet exists.
    if metadata("wallet/wallet.key").is_err() {
        info!("The path 'wallet/wallet.key' does not exist");
        info!("Please follow the steps outlined in the readme.md file");
        return Err(anyhow!("Can not find the 'wallet.key' file in the path: 'wallet/wallet.key'"));
    }

    // Get the wallet
    // Read the wallet key
    let wallet_key = read_to_string("wallet/wallet.key")
                             .context("Error reading the wallet.key file")?;
    
    // Get Mnemonic
    let mnemonic = cosmos::SeedPhrase::from_str(&wallet_key)
                               .context("Failed to retrieve the mnemonic phrase")?;
    
    // Get wallet from Mnemonic
    let wallet = mnemonic.with_hrp(cosmos::AddressHrp::from_string("osmo".to_owned())
                                 .context("Error obtaining AddressHrp")?)
                                 .context("Error identifying the wallet")?;
    
    // Show and record wallet which should match with your 
    // Wallet addr in https://testnet-trade.levana.finance/ 
    info!("Sender Wallet address: {}", wallet);

    // Destination Wallet
    info!("Destination Wallet address: {}", address);
    
    // Execute transaction
    let result = wallet.send_coins(&cosmos_addr, address, amount).await
                            .context(format!("Error executing the transaction at address {}", address))?;

    // All good
    Ok((result.code, result.height, result.txhash))
}