use anyhow::{Context, Result};
use web3::transports::WebSocket;
use web3::{
    contract::{Contract, Options},
    types::{Address, Log, H160, U256},
    Web3,
};

pub async fn get_contract(
    eth: web3::api::Eth<WebSocket>,
    address: Address,
) -> Result<Contract<WebSocket>> {
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

    let contract = Contract::from_json(eth, address, abi.as_bytes())
        .context("Error creating contract from ABI")?;
    Ok(contract)
}

pub async fn process_event(
    web3: Web3<WebSocket>,
    contract: Contract<WebSocket>,
    log: Log,
) -> Result<()> {
    let tx_hash = log
        .transaction_hash
        .context("Log should have transaction hash")?;
    let block_number = log.block_number.context("Log should have block number")?;

    // Extract sender address from the event data
    let sender_address = if let Some(topics) = &log.topics.get(1) {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&topics.0);
        H160::from_slice(&bytes[12..32]) // Convert the last 20 bytes to an address
    } else {
        let tx = web3
            .eth()
            .transaction(tx_hash.into())
            .await
            .context("Failed to fetch transaction")?
            .context("Transaction should exist")?;
        tx.from.unwrap_or_default()
    };

    // Get current number value using retrieve()
    let number: U256 = contract
        .query("retrieve", (), None, Options::default(), None)
        .await
        .context("Failed to query retrieve function")?;

    // Print event information
    println!("\n===== Event Detected =====");
    println!("Transaction: {:#x}", tx_hash);
    println!("Block: {}", block_number);
    println!("Sender: {:#x}", sender_address);
    println!("New Value: {}", number);
    println!("==========================\n");

    Ok(())
}
