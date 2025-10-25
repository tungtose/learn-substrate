# Substrate Username Registry

A simple Substrate-based blockchain that demonstrates storing and retrieving Ethereum address → username mappings via custom JSON-RPC methods in a 2-node environment.

## [Demo Link](https://streamable.com/kjynjn)

## Features

- On-chain storage: Ethereum address (H160) → username mapping
- Signed transactions: Only authenticated users can set usernames
- Custom JSON-RPC methods for querying usernames
- Optional signature verification for read operations
- Two-node sync demonstration

## Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    Substrate Network                        │
│                                                             │
│  ┌────────────────────┐      ┌────────────────────┐         │
│  │   Node A (Alice)   │◄────►│   Node B (Bob)     │         │
│  │   Port: 9944       │ P2P  │   Port: 9945       │         │
│  └────────────────────┘      └────────────────────┘         │
│           │                            │                    │
│           │   Shared Blockchain State  │                    │
│           ▼                            ▼                    │
│  ┌──────────────────────────────────────────────┐           │
│  │  Storage: H160 → BoundedVec<u8, 32>          │           │
│  │  0xabc... → "alice"                          │           │
│  │  0xdef... → "bob"                            │           │
│  └──────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────┘
         ▲                           ▲
         │ RPC/Tool                  │ RPC
         │                           │
    ┌────┴─────┐              ┌──────┴──────┐
    │  Client  │              │   Client    │
    │  (Write) │              │   (Read)    │
    └──────────┘              └─────────────┘

```

## Prerequisites

- Rust stable (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- curl or HTTP client
- Essential build tools:
```bash
sudo pacman -S clang git curl
```

## Project Structure
```
substrate-node-template/
├── pallets/template/         # Custom pallet with username storage
│   └── src/lib.rs            # Storage map + extrinsic logic
├── runtime/                  # Runtime configuration
│   ├── src/lib.rs            # Runtime API declaration
│   └── src/apis.rs           # UsernameApi trait
├── node/                     # Node implementation
│   └── src/
│       └── rpc/
│           └── username.rs   # Custom RPC implementation
├── signature                 # Generate ethereum adress and its signature
│
└── summit_account/           # CLI tool to submit accounts using subxt
```

## Setup Instructions

### 1. Build the Project
```bash
# Build in release mode
cargo build --release
```

### 2. Start Node A (Alice)
```bash
./target/release/solochain-template-node \
  --base-path /tmp/alice \
  --chain local \
  --alice \
  --port 30333 \
  --rpc-port 9944 \
  --validator \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001
```

**Expected output:**
```
Local node identity is: 12D3KooW...
💤 Idle (0 peers), best: #0
```

### 3. Start Node B (Bob)

In a new terminal:
```bash
./target/release/solochain-template-node \
  --base-path /tmp/bob \
  --chain local \
  --bob \
  --port 30334 \
  --rpc-port 9945 \
  --validator \
  --node-key 0000000000000000000000000000000000000000000000000000000000000002 \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

**Verify sync:**
Both nodes should show:
```
💤 Idle (1 peers), best: #123
```

The `1 peers` indicates they're connected and syncing!

## Usage Instructions


### 1. Generate an Ethereum address / signature:
```bash
# Generate test wallet and signature
cargo run --bin test_signature
```

Output example:
```
Ethereum Address: 0x5778e653fd3b463e75457d647656f7c18555513a
Message: query_username
Signature: 0x2ee307c1b533...
```
---

### 2. Store a Username (Write Operation)

```bash
# Build the client
cargo build --release

# Submit username
./target/release/submit-username \
  --url ws://127.0.0.1:9944 \
  --eth-address 0x5778e653fd3b463e75457d647656f7c18555513a \
  --username alice
```

---

### 3. Query a Username (Read Operation)

**Method 1: Basic Query (No Authentication)**
```bash
curl -H "Content-Type: application/json" \
  -d '{
    "id":1,
    "jsonrpc":"2.0",
    "method":"username_get",
    "params":["0x5778e653fd3b463e75457d647656f7c18555513a", null]
  }' \
  http://localhost:9944
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": "alice",
  "id": 1
}
```

**Method 2: Secure Query (With Signature Verification)**

Query with signature generated above:
```bash
curl -H "Content-Type: application/json" \
  -d '{
    "id":1,
    "jsonrpc":"2.0",
    "method":"username_get_secure",
    "params":[
      "0x5778e653fd3b463e75457d647656f7c18555513a",
      "0x2ee307c1b533...",
      "query_username",
      null
    ]
  }' \
  http://localhost:9944
```

---

### 4. Verify Two-Node Sync

**Store data on Node A:**
```bash
# Submit via Node A (port 9944)
# UseCLI tool as shown above
```

**Query from Node B:**
```bash
curl -H "Content-Type: application/json" \
  -d '{
    "id":1,
    "jsonrpc":"2.0",
    "method":"username_get",
    "params":["0x742d35cc6634c0532925a3b844bc9e7595f0beb0", null]
  }' \
  http://localhost:9945
```

**Result:** Node B returns the same username stored on Node A! ✅

This demonstrates blockchain consensus and state synchronization.

---

## Custom JSON-RPC Endpoints

### `username_get`

Retrieve username for an Ethereum address (no authentication required).

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "username_get",
  "params": ["0xETH_ADDRESS", null]
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": "username" | null,
  "id": 1
}
```

### `username_get_secure`

Retrieve username with Ethereum signature verification.

**Request:**
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "username_get_secure",
  "params": [
    "0xETH_ADDRESS",
    "0xSIGNATURE_65_BYTES_HEX",
    "MESSAGE_SIGNED",
    null
  ]
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": "username" | null,
  "id": 1
}
```

**Error (invalid signature):**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": 3,
    "message": "Invalid signature"
  },
  "id": 1
}
```

---

## Design Considerations

### 1. **Ethereum Address as Key (H160)**

We use Ethereum addresses (`H160` = 20 bytes) instead of Substrate accounts to demonstrate cross-chain compatibility. This allows:
- Users to prove ownership with Ethereum wallets (MetaMask, etc.)
- Easier integration with Ethereum ecosystem
- Signature verification using secp256k1 (Ethereum's curve)

### 2. **BoundedVec for Storage**
```rust
BoundedVec<u8, ConstU32<32>>
```

Usernames are stored as `BoundedVec` instead of `Vec` to satisfy `MaxEncodedLen` trait bounds required by Substrate storage. This ensures:
- Predictable storage costs
- Compile-time guarantees about maximum storage size

### 3. **Signature Verification Strategy**

**Off-chain (RPC layer):**
- Fast verification before querying storage
- Reduces on-chain computation
- Optional feature for read operations

**On-chain (pallet layer):**
- Only the signed extrinsic requires authentication
- Uses Substrate's built-in `ensure_signed!` macro
- Future work: Could add unsigned extrinsic with Ethereum signature verification on-chain

### 4. **Two RPC Methods for Queries**

- `username_get`: Public, no authentication (gas-free reads)
- `username_get_secure`: Requires Ethereum signature (access control)

This dual approach allows flexibility:
- Public blockchain data remains accessible
- Sensitive queries can require proof of ownership

### 5. **Why Not Use Custom RPC for Writes?**

Custom RPC endpoints are designed for **queries**, not transactions because:
- RPC servers shouldn't hold private keys (security risk)
- Standard `author_submitExtrinsic` is well-supported by all tooling
- Maintains compatibility with wallets, explorers, and clients
- Follows Substrate best practices

### 6. **Metadata Hash Extension**

The runtime includes `CheckMetadataHash` for transaction safety. For development:
- Can be temporarily disabled in `runtime/src/lib.rs`
- Production: Keep enabled and use compatible client versions

---


## Troubleshooting

### Failed to build rockdbs

```bash
# Solution: This is how I solve on arch linux
sudo pacman -S rocksdb
export ROCKSDB_LIB_DIR=/usr/lib
cargo build --release 
```

### Nodes won't sync (0 peers)

1. Check the bootnode address in Node B command matches Node A's peer ID
2. Look for Node A's peer ID in logs: `Local node identity is: 12D3KooW...`
3. Update the `--bootnodes` parameter

### RPC returns `null` for existing username

1. Verify transaction was included in a block (check Polkadot.js)
2. Wait a few seconds for block production
3. Check storage directly via Polkadot.js: Developer → Chain State → template → usernames

### "Priority too low" error

The transaction is already in the mempool. Wait for it to be included in a block (~6 seconds), then try again.

---

## Resources
- [Build custom pallet](https://docs.polkadot.com/tutorials/polkadot-sdk/parachains/zero-to-hero/build-custom-pallet/)
- [Polkadot.js Apps](https://polkadot.js.org/apps/)
- [subxt Documentation](https://docs.rs/subxt/)
- [Ethereum Signature Verification](https://eips.ethereum.org/EIPS/eip-191)
