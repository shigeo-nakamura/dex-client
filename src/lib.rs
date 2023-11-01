use reqwest;
use reqwest::header::HeaderMap;
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
    client: reqwest::blocking::Client,
    base_url: String,
}

impl DexClient {
    pub fn new(api_key: String, base_url: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .default_headers(Self::headers_with_api_key(api_key))
            .build()
            .unwrap();

        DexClient { client, base_url }
    }

    fn headers_with_api_key(api_key: String) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", api_key.parse().unwrap());
        headers
    }

    pub fn get_ticker(&self, symbol: &str) -> Result<TickerResponse, reqwest::Error> {
        let url = format!("{}/ticker?dex=apex&symbol={}", self.base_url, symbol);
        log::info!("{:?}", url);
        self.client.get(&url).send()?.json()
    }

    pub fn get_yesterday_pnl(&self) -> Result<PnlResponse, reqwest::Error> {
        let url = format!("{}/yesterday-pnl?dex=apex", self.base_url);
        log::info!("{:?}", url);
        self.client.get(&url).send()?.json()
    }

    pub fn create_order(
        &self,
        symbol: &str,
        size: &str,
        side: &str,
    ) -> Result<CreateOrderResponse, reqwest::Error> {
        let url = format!("{}/create-order?dex=apex", self.base_url);
        log::info!("{:?}", url);

        let payload = CreateOrderPayload {
            symbol: symbol.to_string(),
            size: size.to_string(),
            side: side.to_string(),
        };

        self.client.post(&url).json(&payload).send()?.json()
    }
}
