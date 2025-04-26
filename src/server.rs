use autonomi::{Bytes, Client, Wallet};

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

    pub async fn put_data(&self, bytes: &[u8]) -> Result<(String, String), String> {
        println!("Uploading {} bytes...", bytes.len());

        let payment = self.wallet.clone().into();
        let bytes = Bytes::from(bytes.to_vec());
        let (price, addr) = self
            .client
            .data_put_public(bytes, payment)
            .await
            .map_err(|e| {
                println!("Error uploading data: {e}");
                format!("Error uploading data: {e}")
            })?;

        println!("Upload complete with price: {price:?} at: {addr:?}");
        Ok((addr.to_hex(), price.to_string()))
    }
}

async fn init_client(environment: &str) -> Result<Client, String> {
    match environment {
        "local" => Client::init_local().await.map_err(|e| {
            println!("Error initializing client: {e}");
            format!("Error initializing client: {e}")
        }),
        "alpha" => {
            Client::init_alpha()
                .await
                .map_err(|e| {
                    println!("Error initializing client: {e}");
                    format!("Error initializing client: {e}")
                })
        }
        // "autonomi"
        _ => {
            Client::init()
            .await
            .map_err(|e| {
                println!("Error initializing client: {e}");
                format!("Error initializing client: {e}")
            })
        }
    }
}
