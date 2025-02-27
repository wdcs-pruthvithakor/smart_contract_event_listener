use crate::contract::{get_contract, process_event};
use crate::web3_client::Web3Client;
use anyhow::Result;
use futures::stream::StreamExt;
use tokio::time::{sleep, timeout, Duration};
use web3::types::Address;

pub struct EventListener {
    contract_address: Address,
    client: Web3Client,
}

impl EventListener {
    pub fn new(client: Web3Client, contract_address: Address) -> Self {
        EventListener {
            contract_address,
            client,
        }
    }

    pub async fn listen_for_events(&mut self) -> Result<()> {
        loop {
            match self.subscribe_and_listen().await {
                Ok(_) => break, // Successfully finished listening
                Err(e) => {
                    eprintln!("⚠️ Event listener error: {}. Retrying in 5 seconds...", e);
                    sleep(Duration::from_secs(5)).await;
                    self.client.reconnect().await?;
                }
            }
        }
        Ok(())
    }

    pub async fn fetch_past_events(&self) -> Result<()> {
        let event_signature = "NumberUpdatedEvent(address)";
        let event_signature_hash = web3::signing::keccak256(event_signature.as_bytes());

        let filter = web3::types::FilterBuilder::default()
            .address(vec![self.contract_address])
            .from_block(web3::types::BlockNumber::Number(0.into())) // Start from the first block
            .to_block(web3::types::BlockNumber::Latest) // Up to the latest block
            .topics(
                Some(vec![web3::types::H256::from_slice(&event_signature_hash)]),
                None,
                None,
                None,
            )
            .build();

        // Fetch logs for past events
        let web3 = self.client.web3();
        let logs = web3.eth().logs(filter).await?;

        println!("Fetched {} past events", logs.len());

        // Process each log
        for log in logs {
            process_event(
                web3.clone(),
                get_contract(web3.eth(), self.contract_address).await?,
                log,
                true,
            )
            .await?;
        }

        Ok(())
    }

    async fn subscribe_and_listen(&mut self) -> Result<()> {
        let event_signature = "NumberUpdatedEvent(address)";
        let event_signature_hash = web3::signing::keccak256(event_signature.as_bytes());

        let filter = web3::types::FilterBuilder::default()
            .address(vec![self.contract_address])
            .topics(
                Some(vec![web3::types::H256::from_slice(&event_signature_hash)]),
                None,
                None,
                None,
            )
            .build();

        // Retry subscription until successful
        loop {
            let web3 = self.client.web3();
            let contract = get_contract(web3.eth(), self.contract_address).await?;

            // Attempt to subscribe to the logs
            match web3.eth_subscribe().subscribe_logs(filter.clone()).await {
                Ok(mut sub) => {
                    println!("📡 Listening for NumberUpdatedEvent...");

                    // Process logs once subscribed
                    loop {
                        // Set a timeout for 30 seconds
                        let event = timeout(Duration::from_secs(300), sub.next()).await;

                        match event {
                            Ok(Some(log)) => match log {
                                Ok(log) => {
                                    process_event(web3.clone(), contract.clone(), log, false)
                                        .await?
                                }
                                Err(err) => {
                                    eprintln!(
                                        "⚠️ Error processing event: {:?}. Reconnecting...",
                                        err
                                    );
                                    return Err(anyhow::anyhow!(
                                        "Subscription failed, reconnecting..."
                                    ));
                                }
                            },
                            Ok(None) => {
                                // This means the stream was closed, break the loop
                                println!("Stream closed");
                                break;
                            }
                            Err(_) => {
                                // Timeout reached, reconnect or retry
                                println!("Timeout waiting for event, trying again...");
                                break;
                            }
                        }
                    }
                    // Reconnect and re-subscribe after failure
                    println!("Reconnecting...");
                    self.client.reconnect().await?;
                }
                Err(e) => {
                    eprintln!(
                        "⚠️ Failed to subscribe to logs: {}. Retrying in 5 seconds...",
                        e
                    );
                    sleep(Duration::from_secs(5)).await;
                    // Reconnect and re-subscribe after failure
                    self.client.reconnect().await?;
                }
            }
        }
    }
}
