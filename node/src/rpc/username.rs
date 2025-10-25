use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::ErrorObjectOwned,
};

use solochain_template_runtime::apis::UsernameApi as UsernameRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::H160;
use sp_io::hashing::keccak_256;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[rpc(client, server)]
pub trait UsernameApi<BlockHash> {
    #[method(name = "username_get")]
    fn get_username(&self, eth_address: H160, at: Option<BlockHash>) -> RpcResult<Option<String>>;

    #[method(name = "username_get_secure")]
    fn get_username_secure(
        &self,
        eth_address: H160,
        signature: String,
        message: String,
        at: Option<BlockHash>,
    ) -> RpcResult<Option<String>>;
}

pub struct UsernameRpc<C, Block> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> UsernameRpc<C, Block> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

#[async_trait]
impl<C, Block> UsernameApiServer<<Block as BlockT>::Hash> for UsernameRpc<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: UsernameRuntimeApi<Block>,
{
    fn get_username(
        &self,
        eth_address: H160,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Option<String>> {
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        let api = self.client.runtime_api();

        api.get_username(at_hash, eth_address)
            .map(|opt| opt.map(|bytes| String::from_utf8_lossy(&bytes[..]).to_string()))
            .map_err(|e| {
                ErrorObjectOwned::owned(1, "Unable to query username", Some(format!("{:?}", e)))
            })
    }

    fn get_username_secure(
        &self,
        eth_address: H160,
        signature: String,
        message: String,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Option<String>> {
        let sig_bytes = hex::decode(signature.trim_start_matches("0x"))
            .map_err(|_| ErrorObjectOwned::owned(2, "Invalid signature format", None::<()>))?;

        if sig_bytes.len() != 65 {
            return Err(ErrorObjectOwned::owned(
                2,
                "Signature must be 65 bytes",
                None::<()>,
            ));
        }

        let message_bytes = message.as_bytes();

        if !verify_ethereum_signature(&eth_address, message_bytes, &sig_bytes) {
            return Err(ErrorObjectOwned::owned(3, "Invalid signature", None::<()>));
        }

        self.get_username(eth_address, at)
    }
}

fn verify_ethereum_signature(eth_address: &H160, message: &[u8], signature: &[u8]) -> bool {
    log::info!("=== Debug Signature Verification ===");
    log::info!("Expected address: {:?}", eth_address);
    log::info!("Message: {:?}", String::from_utf8_lossy(message));
    log::info!("Signature length: {}", signature.len());

    // Ethereum signed message format
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut eth_message = prefix.as_bytes().to_vec();
    eth_message.extend_from_slice(message);

    let hash = keccak_256(&eth_message);
    log::info!("Message hash: 0x{}", hex::encode(&hash));

    // Convert recovery ID
    let mut sig_array = [0u8; 65];
    sig_array.copy_from_slice(&signature[..65]);
    log::info!("Recovery ID (before): {}", sig_array[64]);

    if sig_array[64] >= 27 {
        sig_array[64] -= 27;
    }
    log::info!("Recovery ID (after): {}", sig_array[64]);

    // Try to recover public key
    match sp_io::crypto::secp256k1_ecdsa_recover(&sig_array, &hash) {
        Ok(pubkey) => {
            log::info!("Public key recovered: 0x{}", hex::encode(&pubkey));

            let recovered_addr_hash = keccak_256(&pubkey[1..]);
            let recovered_eth_addr = H160::from_slice(&recovered_addr_hash[12..32]);

            log::info!("Recovered address: {:?}", recovered_eth_addr);
            log::info!("Match: {}", &recovered_eth_addr == eth_address);

            &recovered_eth_addr == eth_address
        }
        Err(_e) => {
            log::error!("Recovery failed");
            false
        }
    }
}
