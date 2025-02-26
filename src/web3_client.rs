use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use web3::{transports::WebSocket, Web3};

#[derive(Clone)]
pub struct Web3Client {
    node_url: String,
    web3: Option<Web3<WebSocket>>,
}

impl Web3Client {
    pub fn new(node_url: &str) -> Self {
        Web3Client {
            node_url: node_url.to_string(),
            web3: None,
        }
    }

    pub async fn connect_with_retry(&mut self) -> Result<()> {
        let mut attempts = 0;
        loop {
            match WebSocket::new(&self.node_url).await {
                Ok(ws) => {
                    self.web3 = Some(Web3::new(ws));
                    println!("Connected to Ethereum node at: {}", self.node_url);
                    break;
                }
                Err(e) => {
                    attempts += 1;
                    eprintln!(
                        "Failed to connect to WebSocket: {}. Retrying {}/5...",
                        e, attempts
                    );
                    if attempts >= 5 {
                        return Err(anyhow::anyhow!("Failed to connect after 5 attempts").into());
                    }
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
        Ok(())
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        println!("🔄 Attempting to reconnect...");
        self.connect_with_retry().await
    }

    // pub async fn ping(&self) -> Result<()> {
    //     let web3 = self.web3().clone();
    //     // Send a lightweight eth_block_number call to check the connection
    //     match web3.eth().block_number().await {
    //         Ok(_) => Ok(()), // Connection is alive
    //         Err(e) => Err(anyhow::anyhow!("Ping failed: {}", e)),
    //     }
    // }

    pub fn web3(&self) -> Web3<WebSocket> {
        self.web3.clone().expect("Failed to get web3 context")
    }
}
