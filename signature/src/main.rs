use clap::Parser;
use sp_core::{ecdsa, Pair, H160};
use sp_io::hashing::keccak_256;

#[derive(Parser, Debug)]
#[command(name = "signature")]
struct Args {
    /// Username to register
    #[arg(short, long)]
    username: String,

    /// Nonce for replay protection
    #[arg(short, long)]
    nonce: u64,

    #[arg(short, long)]
    private_key: Option<String>,
}

fn main() {
    let args = Args::parse();

    let pair = if let Some(pk_hex) = args.private_key {
        let pk_bytes =
            hex::decode(pk_hex.trim_start_matches("0x")).expect("Invalid private key hex");

        if pk_bytes.len() != 32 {
            eprintln!("Private key must be 32 bytes");
            return;
        }

        let seed: [u8; 32] = pk_bytes.try_into().unwrap();
        ecdsa::Pair::from_seed(&seed)
    } else {
        let (pair, seed) = ecdsa::Pair::generate();
        println!("=== Generated Private Key ===");
        println!("Private Key: 0x{}", hex::encode(&seed));
        println!();
        pair
    };

    let username = args.username.as_bytes();

    // Message format: "set_username:{username}:{nonce}"
    let mut message = b"set_username:".to_vec();
    message.extend_from_slice(username);
    message.push(b':');
    message.extend_from_slice(args.nonce.to_string().as_bytes());

    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut eth_message = prefix.as_bytes().to_vec();
    eth_message.extend_from_slice(&message);

    let message_hash = keccak_256(&eth_message);
    let signature = pair.sign_prehashed(&message_hash);

    // Convert Substrate format (0-3) to Ethereum format (27-30)
    let mut sig_bytes = signature.0.to_vec();
    if sig_bytes[64] < 27 {
        sig_bytes[64] += 27;
    }

    let mut recover_sig = [0u8; 65];
    recover_sig.copy_from_slice(&sig_bytes);
    if recover_sig[64] >= 27 {
        recover_sig[64] -= 27;
    }

    let recovered_pubkey = match sp_io::crypto::secp256k1_ecdsa_recover(&recover_sig, &message_hash)
    {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Failed to recover public key");
            return;
        }
    };

    // Derive Ethereum address from RECOVERED public key (skip first byte)
    let addr_hash = keccak_256(&recovered_pubkey[1..]);
    let eth_address = H160::from_slice(&addr_hash[12..32]);
    let sig_hex = hex::encode(&sig_bytes);

    println!("=== Test Wallet Generated ===");
    println!(
        "Ethereum Address: 0x{}",
        hex::encode(eth_address.as_bytes())
    );
    println!("Message: {}", String::from_utf8_lossy(message.as_slice()));
    println!("Signature: 0x{}", sig_hex);
    println!();
    println!("=== First store username using submit_account binary ===");
    println!("Address: 0x{}", hex::encode(eth_address.as_bytes()));

    println!();

    println!("=== Then test with curl ===");
    println!(r#"curl -H "Content-Type: application/json" \"#);
    println!(
        r#"  -d '{{"id":1,"jsonrpc":"2.0","method":"username_get","params":["0x{}", null]}}' \"#,
        hex::encode(eth_address.as_bytes())
    );
    println!(r#"  http://localhost:9944"#);
}
