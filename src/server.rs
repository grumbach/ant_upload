use autonomi::{Bytes, Client, Wallet, client::payment::PaymentOption};

pub const ENVIRONMENTS: [&str; 3] = ["local", "autonomi", "alpha"];
pub const DEFAULT_ENVIRONMENT: &str = "alpha";
pub const DEFAULT_LOCAL_SECRET_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

#[derive(Clone)]
pub struct Server {
    wallet: Wallet,
    client: Client,
}

impl Server {
    pub async fn new(mut secret_key: &str, environment: &str) -> Result<Self, String> {
        println!("Initializing client with environment: {environment:?}");

        let client = init_client(environment).await?;
        println!("Client initialized");

        let evm_network = client.evm_network();
        println!("EVM network: {evm_network:?}");

        if environment == "local" && secret_key.is_empty() {
            secret_key = DEFAULT_LOCAL_SECRET_KEY;
        }
        let wallet =
            Wallet::new_from_private_key(evm_network.clone(), secret_key).map_err(|e| {
                println!("Error loading wallet: {e}");
                format!("Error loading wallet: {e}")
            })?;
        println!("Wallet loaded");

        Ok(Self { wallet, client })
    }

    pub async fn put_data(&self, bytes: &[u8], filename: &str) -> Result<(String, String), String> {
        println!("Uploading {} bytes...", bytes.len());

        // use existing payment if available (from previous failed attempt)
        let wallet = self.wallet.clone();
        let payment =
            if let Ok(Some(receipt)) = crate::cached_payments::load_payment_for_file(filename) {
                println!("Using cached payment: no need to re-pay");
                PaymentOption::Receipt(receipt)
            } else {
                PaymentOption::Wallet(wallet)
            };

        // upload data
        let bytes = Bytes::from(bytes.to_vec());
        let (price, addr) = match self.client.data_put_public(bytes, payment).await {
            Ok((price, addr)) => (price, addr),
            // save payment to local disk for re-use if upload failed
            Err(autonomi::client::PutError::Batch(upload_state)) => {
                let res = crate::cached_payments::save_payment(filename, &upload_state);
                println!("Error uploading data: {upload_state}");
                println!("Cached payment to local disk for retry: {filename}: {res:?}");
                return Err(format!("Error uploading data: {upload_state}"));
            }
            Err(e) => {
                println!("Error uploading data: {e}");
                return Err(format!("Error uploading data: {e}"));
            }
        };

        println!("Upload complete with price: {price:?} at: {addr:?}");
        Ok((addr.to_hex(), price.to_string()))
    }
}

async fn init_client(environment: &str) -> Result<Client, String> {
    let res = match environment {
        "local" => Client::init_local().await,
        "alpha" => Client::init_alpha().await,
        _ => Client::init().await, // "autonomi"
    };
    res.map_err(|e| {
        println!("Error initializing client: {e}");
        format!("Error initializing client: {e}")
    })
}
