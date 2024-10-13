/// Error handling
use anyhow::{anyhow, Context, Result};

/// Parse input
use clap::Parser;

/// Transaction to execute
#[derive(Parser)]
struct Transaction {
    /// Amount to send to another wallet, e.g. 110uosmo
    coin: cosmos::ParsedCoin,
    /// Destination address to receive the funds
    destination: cosmos::Address,
    /// Capture environment variable mnemonic
    #[clap(env = "COSMOS_WALLET")]
    origin: cosmos::SeedPhrase,
}

/// Transaction Response
struct TResponse {
    /// Transaction responde code
    code: u32,
    /// Node where transaction occurs
    height: i64,
    /// Transaction txhash
    txhash: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Init subscriber to handle traces
    tracing_subscriber::fmt::init();

    tracing::info!("Rust Cli Tool has started");

    // If some wrong format is detected will panic
    let transaction = Transaction::parse();

    // Execute the transaction
    let tresponse = execute_transaction(&transaction)
        .await
        .context("Error encountered during transaction execution")?;

    // Tresponse to String
    let transaction_details = format!(
        "code {} heigth {} txhash {}",
        tresponse.code, tresponse.height, tresponse.txhash
    );

    // All good
    if tresponse.code == 0 {
        tracing::info!(
            "Transaction completed successfully: {}",
            transaction_details
        );
        Ok(())
    } else {
        tracing::error!("Transaction failed: {}", transaction_details);
        Err(anyhow!(
            "Failed to execute transaction: {}",
            transaction_details
        ))
    }
}

/// Executes a transaction on the Osmosis Testnet.
///
/// This function performs the following steps:
/// 1. Connects to Osmosis Testnet
/// 2. Retrieves the balances from the given address.
/// 3. Loads a wallet using a SeedPhrase obtained from the `transaction.origin` field.
/// 4. Sends the specified token amount to the destination address using the provided transaction details.
///
/// ### Arguments
/// * `transaction` - A reference to a [`Transaction`] struct containing the transaction details:
///   - `coin`: The amount to transfer, in the form of a [`ParsedCoin`], e.g., "110uosmo".
///   - `destination`: The wallet address that will receive the funds.
///   - `origin`: The SeedPhrase of the wallet from which the funds will be sent, captured from the environment variable `COSMOS_WALLET`.
///
/// ### Returns
/// Returns a [`TResponse`] struct containing:
/// * `code` - A `u32` representing the transaction response code (0 indicates success, non-zero indicates failure).
/// * `height` - An `i64` representing the block height where the transaction was included.
/// * `tx_hash` - A `String` representing the transaction hash, useful for tracking the transaction on the blockchain.
///
/// ### Errors
/// This function may return an error in the following cases:
/// - If there is a failure connecting to the Cosmos blockchain
/// - If the balance retrieval for the provided address fails
/// - If there is an error identifying the wallet
/// - If the transaction execution fails

async fn execute_transaction(transaction: &Transaction) -> Result<TResponse> {
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
    tracing::info!("Getting balances for address {}", address);

    // Get all balances
    let balances = cosmos::Cosmos::all_balances(&cosmos_addr, address)
        .await
        .context("Failed to retrieve all balances for the Cosmos address")?;

    // Iterate over all Coins and for each one get the balance
    // A Cosmos Address can contains several Coins
    let mut addr_balances = String::new();

    balances.iter().for_each(|balance| {
        addr_balances += &format!("\nDenom: {}, Balance: {}", balance.denom, balance.amount);
    });

    tracing::info!("Balances: {}", addr_balances);

    tracing::info!("Executing transaction...");

    // Vec which contains the Coin to send => 100 uosmo
    // Convert from ParseCoin to cosmos::Coin, ParseCoin fields are private
    let coin: cosmos::Coin = transaction.coin.clone().into();
    let amount: Vec<cosmos::Coin> = vec![coin];

    // Load the wallet
    // Get wallet from SeedPhrase::Mnemonic
    let wallet = transaction
        .origin
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

    // Send Response
    Ok(TResponse {
        code: result.code,
        height: result.height,
        txhash: result.txhash,
    })
}
