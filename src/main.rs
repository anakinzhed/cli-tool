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
use anyhow::{anyhow, Result};

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
    
    info!("RUST CLI TOOL HAS STARTED");

    let args = Args::parse().params;
    //println!("{:?}", args);
    if args.len() != 2 {
        error!("INVALID INPUT | EXAMPLE FORMAT: 100uosmo osmjiewjfhiewf23uiwesnec");
        return Err(anyhow!("INVALID INPUT | EXAMPLE FORMAT: 100uosmo osmjiewjfhiewf23uiwesnec"))
    }

    //Define a new Transaction
    let mut transaction: Transaction = Transaction::new();

    // Check input -> use log to record each operation
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
            // Json with values
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
        // Error 
        Err(e) => {
            error!("Failed to execute transaction: {}", e);
            return Err(e.context("Failed during transaction execution"));
        }
    }

    //All good
    Ok(())

}

// Setup Logging
fn setup_logging() -> Result<()> {
    // Create a directory named "logs" if it doesn't exist
    std::fs::create_dir_all("logs")?;
    
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
            .open(Path::new("logs").join(log_filename))?) 
        // Set Log level
        .level(log::LevelFilter::Info)
        // Apply the log config
        .apply()?; 
    // All good
    Ok(())
}

// Validate input
fn validate_args(args: &Vec<String>) -> Result<(u32, String, String)> {
    
    // Regular expression in order to extract amount and token () are use to define groups of capture
    let amount_token_regex = Regex::new(r"^(\d+)([a-zA-Z]+[-]*[\d]*)$")?;

    // Regular expression in order to valid token and addr, alpha numeric and '-' '_' also
    let valid_str = Regex::new(r"^[a-zA-Z0-9\-_]+$")?;

    // Try to capture amount and token using the regular expresion
    let captures = amount_token_regex.captures(&args[0]).ok_or(anyhow!("INVALID AMOUNTTOKEN FORMAT. MUST BE AMOUNTTOKEN EX: 100btc"))?;

    // Get Amount
    let amount: u32 = captures[1].parse().map_err(|_| anyhow!("INVALID AMOUNT. THE AMOUNT SPECIFIED ISN'T VALID => MUST BE 0 < AMOUNT"))?;
    
    // Get Token 
    let token = captures[2].to_string();

    // Validate Token vs Regular Expr
    if !valid_str.is_match(&token){
        return Err(anyhow!("INVALID TOKEN FORMAT. ONLY ALLOWED ALPHA NUMERIC VALUES AND/OR '-' '_' SPECIAL CHARACTERS"))
    }
   
    // Validate Addr vs Regular Expr
    if !valid_str.is_match(&args[1]){
        return Err(anyhow!("INVALID ADDR FORMAT. ONLY ALLOWED ALPHA NUMERIC VALUES AND/OR '-' '_' SPECIAL CHARACTERS"))
    }

    Ok((amount, token, args[1].to_string()))
}


async fn execute_transaction(transaction: &Transaction) -> Result<(u32, i64, String)> {
    
    // Connect to the blockchain
    info!("Connecting to Cosmos...");
    let cosmos_addr = cosmos::CosmosNetwork::OsmosisTestnet.connect().await?;
    info!("Connected");
    
    // Get the address
    let address = cosmos::Address::from_str(&transaction.get_addr())?;
    
    // Get balance
    info!("Getting balance for wallet {}", address);

    let balances = cosmos::Cosmos::all_balances(&cosmos_addr, address).await?;

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
        info!("PATH WALLET/WALLET.KEY DOESN'T EXIST");
        info!("Please follow these steps");
        info!("1.- Create wallet dir");
        info!("2.- Inside place 'wallet.key' file which contains your wallet mnemonic phrase");
        info!("3.- Please be sure to read all the required steps at the top of this file");
        return Err(anyhow!("Can not find the 'wallet.key' file in the path: 'wallet/wallet.key'"));
    }

    // Get the wallet
    // Read the wallet key
    let wallet_key = read_to_string("wallet/wallet.key")?;
    
    // Get Mnemonic
    let mnemonic = cosmos::SeedPhrase::from_str(&wallet_key)?;
    
    // Get wallet from Mnemonic
    let wallet = mnemonic.with_hrp(cosmos::AddressHrp::from_string("osmo".to_owned())?)?;
    
    // Show and record wallet which should match with your 
    // Wallet addr in https://testnet-trade.levana.finance/ 
    info!("Sender Wallet address: {}", wallet);

    // Destination Wallet
    info!("Destination Wallet address: {}", address);
    
    // Execute transaction
    let result = wallet.send_coins(&cosmos_addr, address, amount).await?; 

    // All good
    Ok((result.code, result.height, result.txhash))
}

#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn input_valid_one() {
        let input = vec!["1000BTC".to_owned(), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is OK
        assert!(result.is_ok(), "\nOk EXPECTED BUT GOT Err\n");
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
        assert!(result.is_ok(), "\nOk EXPECTED BUT GOT Err\n");
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
        assert!(result.is_err(), "\nErr EXPECTED BUT GOT Ok\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap_err().to_string(), "INVALID AMOUNTTOKEN FORMAT. MUST BE AMOUNTTOKEN EX: 100btc");
    }

    #[test]
    fn input_only_string_1st_slice() {
        let input = vec!["btceth".to_owned(), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is err
        assert!(result.is_err(), "\nErr EXPECTED BUT GOT Ok\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap_err().to_string(), "INVALID AMOUNTTOKEN FORMAT. MUST BE AMOUNTTOKEN EX: 100btc");
    }
    
    #[test]
    fn input_special_char_1st_slice() {
        let input = vec!["239btc!".to_owned(), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is err
        assert!(result.is_err(), "\nErr expected but got Ok\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap_err().to_string(), "INVALID AMOUNTTOKEN FORMAT. MUST BE AMOUNTTOKEN EX: 100btc");
    }

    #[test]
    fn input_special_char_2nd_slice() {
        let input = vec!["239btc".to_owned(), "1A1zP1eP5QGef!i2DMPTfTL5SLmv7DivfNa".to_owned()];
        let result = validate_args(&input);
        // Check is result is err
        assert!(result.is_err(), "\nErr expected but got Ok\n");
        // unwrap Result and compare
        assert_eq!(result.unwrap_err().to_string(), "INVALID ADDR FORMAT. ONLY ALLOWED ALPHA NUMERIC VALUES AND/OR '-' '_' SPECIAL CHARACTERS");
    }

}