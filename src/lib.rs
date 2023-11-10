use reqwest::header::HeaderMap;
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt::{self, Display};

#[derive(Deserialize, Debug)]
pub struct TickerResponse {
    pub result: String,
    pub symbol: String,
    pub price: String,
}

#[derive(Deserialize, Debug)]
pub struct PnlResponse {
    pub result: String,
    pub data: String,
}

#[derive(Serialize)]
struct CreateOrderPayload {
    symbol: String,
    size: String,
    side: String,
}

#[derive(Serialize)]
struct CloseAllPositionsPayload {
    symbol: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateOrderResponse {
    pub result: String,
    pub price: String,
    pub size: String,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CloseAllPositionsResponse {
    pub result: String,
}

#[derive(Clone, Debug)]
pub struct DexClient {
    client: Client,
    base_url: String,
}

#[derive(Debug)]
pub enum DexError {
    Serde(serde_json::Error),
    Reqwest(reqwest::Error),
    ServerResponse(String),
}

impl Display for DexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DexError::Serde(ref e) => write!(f, "Serde JSON error: {}", e),
            DexError::Reqwest(ref e) => write!(f, "Reqwest error: {}", e),
            DexError::ServerResponse(ref e) => write!(f, "Server response error: {}", e),
        }
    }
}

impl StdError for DexError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            DexError::Serde(ref e) => Some(e),
            DexError::Reqwest(ref e) => Some(e),
            DexError::ServerResponse(_) => None,
        }
    }
}

impl From<serde_json::Error> for DexError {
    fn from(err: serde_json::Error) -> DexError {
        DexError::Serde(err)
    }
}

impl From<reqwest::Error> for DexError {
    fn from(err: reqwest::Error) -> DexError {
        DexError::Reqwest(err)
    }
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
        url: &str,
    ) -> Result<T, DexError> {
        let response = result.map_err(DexError::from)?;

        if response.status().is_success() {
            let headers = response.headers().clone();
            let body = response.text().await.map_err(DexError::from)?;
            log::debug!("Response body: {}", body);

            serde_json::from_str(&body).map_err(|e| {
                log::warn!("Response header: {:?}", headers);
                log::error!("Failed to deserialize response: {}", e);
                DexError::Serde(e)
            })
        } else {
            let error_message = format!(
                "Server returned error: {}. Requested url: {}",
                response.status(),
                url
            );
            log::error!("{}", &error_message);
            Err(DexError::ServerResponse(error_message))
        }
    }

    pub async fn get_ticker(&self, symbol: &str) -> Result<TickerResponse, DexError> {
        let url = format!("{}/ticker?dex=apex&symbol={}", self.base_url, symbol);
        log::trace!("{:?}", url);
        self.handle_request(self.client.get(&url).send().await, &url)
            .await
    }

    pub async fn get_yesterday_pnl(&self) -> Result<PnlResponse, DexError> {
        let url = format!("{}/yesterday-pnl?dex=apex", self.base_url);
        log::trace!("{:?}", url);
        self.handle_request(self.client.get(&url).send().await, &url)
            .await
    }

    pub async fn create_order(
        &self,
        symbol: &str,
        size: &str,
        side: &str,
    ) -> Result<CreateOrderResponse, DexError> {
        let url = format!("{}/create-order?dex=apex", self.base_url);
        log::trace!("{:?}", url);
        let payload = CreateOrderPayload {
            symbol: symbol.to_string(),
            size: size.to_string(),
            side: side.to_string(),
        };
        self.handle_request(self.client.post(&url).json(&payload).send().await, &url)
            .await
    }

    pub async fn close_all_positions(
        &self,
        symbol: Option<String>,
    ) -> Result<CloseAllPositionsResponse, DexError> {
        let url = format!("{}/close_all_positions?dex=apex", self.base_url);
        log::trace!("{:?}", url);
        let payload = match symbol {
            Some(symbol) => CloseAllPositionsPayload { symbol },
            None => CloseAllPositionsPayload {
                symbol: String::new(),
            },
        };
        self.handle_request(self.client.post(&url).json(&payload).send().await, &url)
            .await
    }
}
