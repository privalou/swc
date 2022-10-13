#[derive(Debug, Clone)]
pub struct State {
    pub mongo_client: mongodb::Client,
}

impl State {
    pub async fn new(uri: &str) -> tide::Result<Self> {
        let mongo = mongodb::Client::with_uri_str(uri).await?;
        Ok(Self {
            mongo_client: mongo,
        })
    }

    pub fn with_client(client: mongodb::Client) -> Self {
        Self {
            mongo_client: client,
        }
    }
}
