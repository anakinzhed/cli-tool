// using our cosmos-rs library
use cosmos;

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

// Regular expresions 
use regex::*;

// Json Response
use serde_json::json;

// Parse input
use clap::Parser;
#[derive(Parser, Debug)]
struct Args {
    // param[0] => Example: 110uosmo
    // param[1] => Example: "osmoihjwfeiuehwifewofnbifwwef"
    params: Vec<String>,
}

//Transaction todo
#[derive(Debug)]
struct Transaction{
    amount: u32,
    token: String,
    addr: String,
}

//Transcation methods
impl Transaction{
    
    fn new() -> Self{
        Transaction{
            amount : 0,
            token : "".to_owned(),
            addr : "".to_owned(),
        }
    }

    fn get_amount(&self) -> u32 {
        self.amount
    }

    fn get_token(&self) -> String {
        self.token.clone()
    }
    
    fn get_addr(&self) -> String {
        self.addr.clone()
    }
    
    fn set_amount(&mut self, amount: u32) {
        self.amount = amount;
    }

    fn set_token(&mut self, token: String) {
        self.token = token;
    }

    fn set_addr(&mut self, addr: String) {
        self.addr = addr;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    
    // Initialize Logging 
    setup_logging()?;
    
    info!("Rust Cli Tool has started");

    let args = Args::parse().params;

    if args.len() != 2 {
        error!("Invalid input, example format: 100uosmo osmjiewjfhiewf23uiwesnec");
        return Err(anyhow!("Invalid input, example format: 100uosmo osmjiewjfhiewf23uiwesnec"))
    }

    //Define a new Transaction
    let mut transaction: Transaction = Transaction::new();

    // Check input
    match validate_args(&args){
        // All good
        Ok((amount, token , addr)) => {
            
            info!("Parameters details:");
            info!("Amount: {} Token: {} Addr: {}", amount, token, addr);
            
            // Set values for transaction
            transaction.set_amount(amount);
            transaction.set_token(token);
            transaction.set_addr(addr);

            info!("Transaction parameters have been set");
            // args no longer required
            std::mem::drop(args);
        }
        // There were an error
        Err(e) =>{
            error!("{e}");
            return Err(e);
        }
    } 

    // Execute the transaction
    match execute_transaction(&transaction).await{
        // Transaction completed
        Ok((code, height, txhash)) => {

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
        }
        Err(e) => {
            error!("Failed to execute transaction: {}", e);
            return Err(e.context("Error encountered during transaction execution"));
        }
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
            // Write mode 
            .write(true)  
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

/// Validates the provided arguments for a Cosmos transaction.
///
/// This function validates two key arguments: 
/// 1. The combination of an amount and a token (in the format `AMOUNTTOKEN`).
/// 2. A recipient address.
///
/// The function uses regular expressions to ensure that both the token and address are valid.
/// 
/// ### Arguments
/// * `args` - A reference to a `Vec<String>` containing the arguments to be validated. The expected format is:
///   - `args[0]`: A string containing the amount and token (e.g., "100btc").
///   - `args[1]`: A string representing the recipient address.
///
/// ### Returns
/// Returns a tuple containing:
/// * `amount` - A `u32` representing the parsed amount from the `AMOUNTTOKEN` argument.
/// * `token` - A `String` containing the extracted token symbol from the `AMOUNTTOKEN`.
/// * `address` - A `String` representing the validated recipient address.
///
/// ### Errors
/// This function may return an error in the following cases:
/// - If the `AMOUNTTOKEN` format is invalid (e.g., "100btc" is the expected format).
/// - If the token contains invalid characters (only alphanumeric characters, `-`, and `_` are allowed).
/// - If the address contains invalid characters (only alphanumeric characters, `-`, and `_` are allowed).
/// - If the amount is not a valid positive integer.

fn validate_args(args: &Vec<String>) -> Result<(u32, String, String)> {
    
    // Regular expression in order to extract amount and token () are use to define groups of capture
    let amount_token_regex = Regex::new(r"^(\d+)([a-zA-Z]+[-]*[\d]*)$")
                                    .context("Failed to create regular expression for [amount][token]")?;

    // Regular expression in order to valid token and addr, alpha numeric and '-' '_' also
    let valid_str = Regex::new(r"^[a-zA-Z0-9\-_]+$")
                           .context("Error constructing regular expression for validating token and address")?;

    // Try to capture amount and token using the regular expresion
    let captures = amount_token_regex.captures(&args[0])
                                .context("Invalid AMOUNTTOKEN format. Expected format: 'AMOUNTTOKEN' (ex. '100btc')")?;

    // Get Amount
    let amount: u32 = captures[1].parse().context("Invalid amount. The specified amount must be greater than 0")?;
    
    // Get Token 
    let token = captures[2].to_string();

    // Validate Token vs Regular Expr
    if !valid_str.is_match(&token){
        return Err(anyhow!("Invalid token format. Only alphanumeric values and/or '-' and '_' special characters are allowed"))
    }
   
    // Validate Addr vs Regular Expr
    if !valid_str.is_match(&args[1]){
        return Err(anyhow!("Invalid address format. Only alphanumeric values and/or '-' and '_' special characters are allowed"))
    }

    Ok((amount, token, args[1].to_string()))
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
async fn execute_transaction(transaction: &Transaction) -> Result<(u32, i64, String)> {
    
    // Connect to the blockchain
    info!("Connecting to Cosmos...");
    let cosmos_addr = cosmos::CosmosNetwork::OsmosisTestnet.connect().await
                              .context("Error connecting to Cosmos")?;
    info!("Connection successful.");
    
    // Get the address
    let address = cosmos::Address::from_str(&transaction.get_addr())
                           .context(format!("Failed to convert {} to Address", &transaction.get_addr()))?;
    
    // Get balance
    info!("Getting balance for wallet {}", address);

    let balances = cosmos::Cosmos::all_balances(&cosmos_addr, address).await
                              .context("Failed to retrieve all balances for the Cosmos address")?;

    // Iterate over all balances for each 
    balances.iter().for_each(|balance| {
        // Take the field denom and split it by '/'
        let denom_slice: Vec<&str> = balance.denom.split('/').collect();
        // If was successfull, check if this denom is the one by task
        if denom_slice.len() > 2 && denom_slice[1] == transaction.get_addr(){
            // Show and record info
            info!("Balance: {}", balance.amount);
        }
    });

    info!("Executing transaction....");

    // Vec which contains the Coin to send => 100 uosmo
    let amount: Vec<cosmos::Coin> = vec![cosmos::Coin{denom: transaction.get_token() , amount: transaction.get_amount().to_string()}];
    
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


//
// UNIT TEST
//
#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn input_valid_one() {
        let input = vec!["1000BTC".to_owned(), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is OK
        assert!(result.is_ok(), "\nOk expected but got Err\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap(), (
            1000 as u32,
            "BTC".to_owned(),
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()
        ));
    }

    #[test]
    fn input_valid_two() {
        let input = vec!["1000ERC-20".to_owned(),"ethA1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is OK
        assert!(result.is_ok(), "\nOk expected but got Err\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap(), (
            1000 as u32,
            "ERC-20".to_owned(),
            "ethA1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()
        ));
    }

    #[test]
    fn input_only_number_1st_slice() {
        let input = vec!["1000234".to_owned(), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is err
        assert!(result.is_err(), "\nErr expected but got Ok\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap_err().to_string(), "Invalid AMOUNTTOKEN format. Expected format: 'AMOUNTTOKEN' (ex. '100btc')");
    }

    #[test]
    fn input_only_string_1st_slice() {
        let input = vec!["btceth".to_owned(), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is err
        assert!(result.is_err(), "\nErr expected but got Ok\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap_err().to_string(), "Invalid AMOUNTTOKEN format. Expected format: 'AMOUNTTOKEN' (ex. '100btc')");
    }
    
    #[test]
    fn input_special_char_1st_slice() {
        let input = vec!["239btc!".to_owned(), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is err
        assert!(result.is_err(), "\nErr expected but got Ok\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap_err().to_string(), "Invalid AMOUNTTOKEN format. Expected format: 'AMOUNTTOKEN' (ex. '100btc')");
    }

    #[test]
    fn input_special_char_2nd_slice() {
        let input = vec!["239btc".to_owned(), "1A1zP1eP5QGef!i2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is err
        assert!(result.is_err(), "\nErr expected but got Ok\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap_err().to_string(), "Invalid address format. Only alphanumeric values and/or '-' and '_' special characters are allowed");
    }

}