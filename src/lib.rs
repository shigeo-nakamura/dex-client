use reqwest::header::HeaderMap;
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct TickerResponse {
    pub symbol: String,
    pub price: String,
}

#[derive(Deserialize, Debug)]
pub struct PnlResponse {
    pub data: String,
}

#[derive(Serialize)]
struct CreateOrderPayload {
    symbol: String,
    size: String,
    side: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateOrderResponse {
    pub result: String,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DexClient {
    client: Client,
    base_url: String,
}

impl DexClient {
    pub async fn new(api_key: String, base_url: String) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .default_headers(Self::headers_with_api_key(api_key))
            .build()?;

        Ok(DexClient { client, base_url })
    }

    fn headers_with_api_key(api_key: String) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", api_key.parse().unwrap());
        headers
    }

    async fn handle_request<T: serde::de::DeserializeOwned>(
        &self,
        result: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<T, reqwest::Error> {
        let response = result?;
        response.json().await
    }

    pub async fn get_ticker(&self, symbol: &str) -> Result<TickerResponse, reqwest::Error> {
        let url = format!("{}/ticker?dex=apex&symbol={}", self.base_url, symbol);
        log::debug!("{:?}", url);
        self.handle_request(self.client.get(&url).send().await)
            .await
    }

    pub async fn get_yesterday_pnl(&self) -> Result<PnlResponse, reqwest::Error> {
        let url = format!("{}/yesterday-pnl?dex=apex", self.base_url);
        log::debug!("{:?}", url);
        self.handle_request(self.client.get(&url).send().await)
            .await
    }

    pub async fn create_order(
        &self,
        symbol: &str,
        size: &str,
        side: &str,
    ) -> Result<CreateOrderResponse, reqwest::Error> {
        let url = format!("{}/create-order?dex=apex", self.base_url);
        log::debug!("{:?}", url);
        let payload = CreateOrderPayload {
            symbol: symbol.to_string(),
            size: size.to_string(),
            side: side.to_string(),
        };
        self.handle_request(self.client.post(&url).json(&payload).send().await)
            .await
    }
}
