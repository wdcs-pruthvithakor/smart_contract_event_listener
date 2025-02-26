mod contract;
mod event_listener;
mod web3_client;
use anyhow::{Context, Result};
use dotenv::dotenv;
use event_listener::EventListener;
use std::env;
use std::str::FromStr;
use web3::types::Address;
use web3_client::Web3Client;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get environment variables
    let node_url = env::var("NODE_URL").context("NODE_URL must be set in .env file")?;
    let contract_address =
        env::var("CONTRACT_ADDRESS").context("CONTRACT_ADDRESS must be set in .env file")?;

    // Create Ethereum Client with WebSocket and retry logic
    let mut client = Web3Client::new(&node_url);
    client.connect_with_retry().await?;

    // Convert contract address string to Address
    let contract_address =
        Address::from_str(&contract_address).context("Invalid contract address format")?;

    println!("Monitoring contract at: {:#x}", contract_address);

    // Create EventListener instance
    let mut listener = EventListener::new(client, contract_address);
    listener.listen_for_events().await?;

    Ok(())
}
