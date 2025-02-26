# Ethereum Event Listener

A Rust application that monitors a specific Ethereum smart contract for events and displays real-time updates when the contract state changes.

## Overview

This application connects to an Ethereum node via WebSocket and listens for `NumberUpdatedEvent` events emitted by a simple Storage smart contract. When an event is detected, it displays transaction details and queries the contract for the updated value.

## Features

- WebSocket connection to Ethereum nodes with automatic reconnection
- Event monitoring for specific contract events
- Robust error handling and retry logic
- Real-time display of contract state changes

## Prerequisites

- Rust and Cargo installed ([rustup.rs](https://rustup.rs/))
- Access to an Ethereum node with WebSocket support (local or remote)
- A deployed instance of the Storage contract

## Project Structure

```
├── src/
│   ├── main.rs          # Application entry point
│   ├── contract.rs      # Contract ABI and event processing
│   ├── event_listener.rs # Event subscription and handling
│   ├── web3_client.rs   # WebSocket client with retry logic
├── .env                 # Environment configuration
├── Cargo.toml           # Project dependencies
```

## Smart Contract

The application monitors the following Solidity contract:

```solidity
// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.2 <0.9.0;

contract Storage {
    uint256 number;
    event NumberUpdatedEvent(address Sender);

    function store(uint256 num) public {
        number = num;
        emit NumberUpdatedEvent(msg.sender);
    }

    function retrieve() public view returns (uint256) {
        return number;
    }
}
```

## Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/wdcs-pruthvithakor/smart_contract_event_listener.git
   cd smart_contract_event_listener
   ```

2. Create a `.env` file in the project root with the following variables:
   ```
   NODE_URL=ws://localhost:8545
   CONTRACT_ADDRESS=0xYourContractAddress
   ```

3. Build the application:
   ```bash
   cargo build --release
   ```

## Usage

1. Run the application:
   ```bash
   cargo run --release
   ```

2. The application will connect to the Ethereum node, subscribe to events, and display:
   ```
   Connected to Ethereum node at: ws://localhost:8545
   Monitoring contract at: 0xYourContractAddress
   📡 Listening for NumberUpdatedEvent...
   ```

3. When an event is detected, you'll see output like:
   ```
   ===== Event Detected =====
   Transaction: 0x1234...
   Block: 123456
   Sender: 0xabcd...
   New Value: 42
   ==========================
   ```

## Testing with Hardhat

For local testing, you can use Hardhat to deploy the contract and trigger events:

1. Deploy the contract using Hardhat Ignition:
   - Create a `StorageModule.js` file:

     ```javascript
     const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");

     module.exports = buildModule("StorageModule", (m) => {
         const store = m.contract("Storage");

         return { store };
     });
     ```

   - Run the deployment:
     ```bash
     npx hardhat ignition deploy ./ignition/modules/StorageModule.js --network localhost
     ```

2. Set up your `.env` file in the root directory of the project with the following required environment variables:

   ```
   NODE_URL=ws://localhost:8545
   CONTRACT_ADDRESS=0x5fbdb2315678afecb367f032d93f642f64180aa3  # Replace with your deployed contract address
   PRIVATE_KEY=your_private_key_here  # Replace with your Ethereum private key
   ```

   The `NODE_URL` should point to a running Ethereum node (like a local Hardhat node or Infura).
3. Use the provided JavaScript script to trigger events:
   ```javascript
   require("dotenv").config();
   const { ethers } = require("ethers");
   
   // Load environment variables
   const RPC_URL = process.env.NODE_URL;
   const PRIVATE_KEY = process.env.PRIVATE_KEY;
   const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS;
   
   async function main() {
       const provider = new ethers.JsonRpcProvider(RPC_URL);
       const wallet = new ethers.Wallet(PRIVATE_KEY, provider);
   
       const contractABI = [
           "function store(uint256 num) public",
           "function retrieve() public view returns (uint256)"
       ];
   
       const contract = new ethers.Contract(CONTRACT_ADDRESS, contractABI, wallet);
   
       console.log("Storing number...");
       const tx = await contract.store(8);
       await tx.wait();
       console.log(`Transaction hash: ${tx.hash}`);
   
       const storedValue = await contract.retrieve();
       console.log(`Stored value: ${storedValue.toString()}`);
   }
   
   main().catch(console.error);
   ```

## How It Works

1. The application establishes a WebSocket connection to the Ethereum node
2. It creates a filter to capture specific events from the target contract 
3. When an event is detected, it:
   - Extracts transaction details and sender address
   - Queries the contract for the updated value
   - Displays all information in a formatted output
4. If connection issues occur, it automatically retries with backoff

### Setting Up the Ethereum Event Listener

1. **Configure the Rust Event Listener**: Ensure that your `Cargo.toml` has all dependencies properly configured, and the `.env` file has the correct values for `NODE_URL` and `CONTRACT_ADDRESS`.

2. **Start the Event Listener**: Run the Rust project using the following command:

   ```bash
   cargo run
   ```

   This will connect to the Ethereum node (e.g., Hardhat running locally), subscribe to the `NumberUpdatedEvent`, and print event details (transaction hash, sender address, block number, and stored value) whenever the event is triggered.

### Triggering the Event

1. **Trigger an Event**: Use the provided `trigger_event.js` file to call the `store` function of the smart contract and emit the `NumberUpdatedEvent`.

   - Ensure the environment variables `NODE_URL`, `PRIVATE_KEY`, and `CONTRACT_ADDRESS` are set in a `.env` file.
   - Run the script:
     ```bash
     node trigger_event.js
     ```

   This will call the `store(8)` function, emit the event, and the Rust program will log the details.

### Sample Output from the Rust Event Listener

Once the `NumberUpdatedEvent` is triggered, the Rust event listener will print information such as:

```
Connected to Ethereum node at: ws://localhost:8545/
Monitoring contract at: 0x5fbdb2315678afecb367f032d93f642f64180aa3
📡 Listening for NumberUpdatedEvent...

===== Event Detected =====
Transaction: 0x27bdd78e18d4bfd05c7499d6d6a651ed63b017eb0cf9d0b6370185a20db5917d
Block: 6
Sender: 0xbda5747bfd65f08deb54cb465eb87d40e51b197e
New Value: 8
==========================
```

This output contains:
- **Transaction Hash**: The hash of the transaction that triggered the event.
- **Block Number**: The block number where the transaction was mined.
- **Sender**: The address that initiated the `store` function call.
- **New Value**: The new value stored in the smart contract (after calling `store`).

### Debugging and Reconnection

If there are any issues with the WebSocket connection or event subscription, the system will automatically attempt to reconnect every 5 seconds and continue listening for events.

You can also adjust the reconnection logic or timeouts in the `EventListener` struct as per your requirements.

---

## Dependencies

- `web3`: Ethereum interaction
- `tokio`: Asynchronous runtime
- `anyhow`: Error handling
- `dotenv`: Environment variable management
- `futures`: Stream processing

## Troubleshooting

- **Connection Issues**: Ensure your Ethereum node has WebSocket support enabled
- **No Events**: Verify the contract address is correct and events are being emitted
- **Parsing Errors**: Confirm the contract ABI matches the deployed contract

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.