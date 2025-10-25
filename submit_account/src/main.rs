use clap::Parser;
use sp_core::H160;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;

#[subxt::subxt(runtime_metadata_path = "../artifacts/metadata.scale")]
pub mod polkadot {}

#[derive(Parser, Debug)]
#[command(name = "submit-username")]
#[command(about = "Submit username to Substrate blockchain", long_about = None)]
struct Args {
    /// Node RPC URL
    #[arg(short, long, default_value = "ws://127.0.0.1:9944")]
    url: String,

    /// Ethereum address
    #[arg(short, long)]
    eth_address: String,

    /// Username to set
    #[arg(short = 'n', long)]
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("Connecting to node: {}", args.url);
    let api = OnlineClient::<PolkadotConfig>::from_url(&args.url).await?;
    println!("Connected");

    let eth_addr_hex = args.eth_address.trim_start_matches("0x");
    let eth_addr_bytes = hex::decode(eth_addr_hex).map_err(|_| "Invalid Ethereum address hex")?;

    if eth_addr_bytes.len() != 20 {
        return Err("Ethereum address must be 20 bytes".into());
    }

    let mut eth_address = [0u8; 20];
    eth_address.copy_from_slice(&eth_addr_bytes);
    let eth_address = H160(eth_address);

    let username = args.username.as_bytes().to_vec();

    println!("Submitting ...");
    println!("Eth Address: 0x{}", hex::encode(eth_addr_bytes));
    println!("Username: {}", String::from_utf8_lossy(&username));
    println!();

    let tx = polkadot::tx()
        .template()
        .set_username(eth_address, username);

    let from = dev::alice();

    api.tx()
        .sign_and_submit_then_watch_default(&tx, &from)
        .await?
        .wait_for_finalized_success()
        .await?;

    println!("=== Transaction finalized in block! ===");
    println!("Query with:");
    println!(
        r#"curl -H "Content-Type: application/json" -d '{{"id":1,"jsonrpc":"2.0","method":"username_get","params":["0x{}", null]}}' {}"#,
        hex::encode(eth_address.as_bytes()),
        args.url.replace("ws://", "http://").replace("9944", "9944")
    );

    Ok(())
}
