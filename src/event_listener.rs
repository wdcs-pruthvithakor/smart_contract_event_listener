use crate::contract::{get_contract, process_event};
use anyhow::{Context, Result};
use futures::stream::StreamExt;
use web3::Web3;
use web3::{transports::WebSocket, types::Address};

pub struct EventListener {
    web3: Web3<WebSocket>,
    contract_address: Address,
}

impl EventListener {
    pub fn new(web3: Web3<WebSocket>, contract_address: Address) -> Self {
        EventListener {
            web3,
            contract_address,
        }
    }

    pub async fn listen_for_events(&self) -> Result<()> {
        // ABI for the event we want to listen to
        let event_signature = "NumberUpdatedEvent(address)";
        let event_signature_hash = web3::signing::keccak256(event_signature.as_bytes());

        // Create filter for the event
        let filter = web3::types::FilterBuilder::default()
            .address(vec![self.contract_address])
            .topics(
                Some(vec![web3::types::H256::from_slice(&event_signature_hash)]),
                None,
                None,
                None,
            )
            .build();

        // Get contract instance (to call retrieve() later)
        let contract = get_contract(self.web3.eth(), self.contract_address).await?;

        // Create event subscription
        let mut sub = self
            .web3
            .eth_subscribe()
            .subscribe_logs(filter)
            .await
            .context("Failed to subscribe to logs")?;
        println!("Listening for NumberUpdatedEvent...");

        // Process events
        while let Some(logs) = sub.next().await {
            match logs {
                Ok(log) => process_event(self.web3.clone(), contract.clone(), log).await?,
                Err(err) => eprintln!("Error processing event: {:?}", err),
            }
        }

        Ok(())
    }
}
