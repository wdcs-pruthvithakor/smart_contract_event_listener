use dotenv::dotenv;
use std::env;
use std::str::FromStr;
use web3::contract::{Contract, Options};
use web3::types::{Address, FilterBuilder, Log, H160, U256};
use web3::Web3;
use web3::transports::WebSocket;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> web3::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Get environment variables
    let node_url = env::var("NODE_URL").expect("NODE_URL must be set in .env file");
    let contract_address = env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set in .env file");
    
    // Connect to Ethereum node
    let ws = WebSocket::new(&node_url).await?;
    let web3 = Web3::new(ws);
    
    println!("Connected to Ethereum node at: {}", node_url);
    
    // Convert contract address string to Address
    let contract_address = Address::from_str(&contract_address)
        .expect("Invalid contract address format");
        
    println!("Monitoring contract at: {:#x}", contract_address);
    
    // ABI for the event we want to listen to
    let event_signature = "NumberUpdatedEvent(address)";
    let event_signature_hash = web3::signing::keccak256(event_signature.as_bytes());
    
    // Create filter for the event
    let filter = FilterBuilder::default()
        .address(vec![contract_address])
        .topics(
            Some(vec![web3::types::H256::from_slice(&event_signature_hash)]),
            None,
            None,
            None,
        )
        .build();
    
    // Get contract instance (to call retrieve() later)
    let contract = get_contract(web3.eth(), contract_address).await?;
    
    // Create event subscription
    let mut sub = web3.eth_subscribe().subscribe_logs(filter).await?;
    println!("Listening for NumberUpdatedEvent...");
    
    // Process events
    while let Some(logs) = sub.next().await {
        match logs {
        Ok(log) => process_event(web3.clone(), contract.clone(), log).await.expect("Error processing event"),
        Err(err) => println!("{err:?}")
        }
    }
    
    Ok(())
}

async fn process_event(web3: Web3<WebSocket>, contract: Contract<WebSocket>, log: Log) -> web3::Result<()> {
    // Get transaction details
    let tx_hash = log.transaction_hash
        .expect("Log should have transaction hash");
    
    let block_number = log.block_number
        .expect("Log should have block number");
        
    // Extract sender address from the event data
    let sender_address = if let Some(topics) = &log.topics.get(1) {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&topics.0);
        H160::from_slice(&bytes[12..32]) // Convert the last 20 bytes to an address
    } else {
        // Alternatively, get it from transaction
        let tx = web3.eth().transaction(tx_hash.into()).await?
            .expect("Transaction should exist");
        tx.from.unwrap_or_default()
    };
    
    // Get current number value using retrieve()
    let number: U256 = contract.query("retrieve", (), None, Options::default(), None)
        .await.unwrap();
    
    // Print event information
    println!("\n===== Event Detected =====");
    println!("Transaction: {:#x}", tx_hash);
    println!("Block: {}", block_number);
    println!("Sender: {:#x}", sender_address);
    println!("New Value: {}", number);
    println!("==========================\n");
    
    Ok(())
}

async fn get_contract(eth: web3::api::Eth<WebSocket>, address: Address) -> web3::Result<Contract<WebSocket>> {
    // Minimal ABI for the retrieve function
    let abi = r#" [
        {
        "anonymous": false,
        "inputs": [
            {
            "indexed": false,
            "internalType": "address",
            "name": "Sender",
            "type": "address"
            }
        ],
        "name": "NumberUpdatedEvent",
        "type": "event"
        },
        {
        "inputs": [],
        "name": "retrieve",
        "outputs": [
            {
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
            }
        ],
        "stateMutability": "view",
        "type": "function"
        },
        {
        "inputs": [
            {
            "internalType": "uint256",
            "name": "num",
            "type": "uint256"
            }
        ],
        "name": "store",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
        }
    ]"#;
    
    let contract = Contract::from_json(eth, address, abi.as_bytes()).expect("Error creating contract");
    Ok(contract)
}