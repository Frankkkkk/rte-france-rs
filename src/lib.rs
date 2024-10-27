use oauth2::reqwest::http_client;
use oauth2::AccessToken;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenResponse, TokenUrl};

pub mod api;
//use api::generation::GenerationForecast;

const PRODUCTION_BASE_URL: &str = "https://digital.iservices.rte-france.com/";

#[derive(Debug)]
enum ApiException {
    /// Invalid cliend id or secret
    InvalidToken,
    /// Too many requests
    TooManyRequests,
    /// The application (api endpoint) has not be registered with the oauth application
    ApplicationNotRegistered,
    UnknownError,
}

pub trait ApiClient {
    fn http_get(
        &self,
        path: &str,
        query_string: &[(String, String)],
        //) -> Result<reqwest::blocking::Response, reqwest::Error>;
    ) -> Result<String, anyhow::Error>;
}

#[derive(Debug)]
pub struct RteApi {
    client_id: ClientId,
    client_secret: ClientSecret,
    base_url: String,

    token: Option<AccessToken>,
}

impl RteApi {
    pub fn new(client_id: String, client_secret: String) -> Self {
        RteApi {
            client_id: ClientId::new(client_id),
            client_secret: ClientSecret::new(client_secret),
            base_url: PRODUCTION_BASE_URL.to_string(),
            token: None,
        }
    }

    pub fn from_env_values() -> Self {
        let client_id = std::env::var("CLIENT_ID").expect("CLIENT_ID must be set");
        let client_secret = std::env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");

        RteApi::new(client_id, client_secret)
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn authenticate(&mut self) -> anyhow::Result<()> {
        let auth_url = format!("{}/oauth/authorize", self.base_url);
        let token_url = format!("{}/oauth/token", self.base_url);
        let client = BasicClient::new(
            self.client_id.clone(),
            Some(self.client_secret.clone()),
            AuthUrl::new(auth_url)?,
            Some(TokenUrl::new(token_url)?),
        );

        let token_result = client.exchange_client_credentials().request(http_client)?;

        self.token = Some(token_result.access_token().clone());

        Ok(())
    }

    pub fn get_token(&self) -> &String {
        self.token.as_ref().unwrap().secret()
    }
}

impl ApiClient for RteApi {
    fn http_get(
        &self,
        path: &str,
        query_string: &[(String, String)],
    ) -> Result<String, anyhow::Error> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.token.as_ref().unwrap().secret();

        let http_client = reqwest::blocking::Client::new();

        println!("url: {:?}", url);
        let response = http_client
            .get(&url)
            .query(&query_string)
            .bearer_auth(token)
            .send()?;

        let status_code = response.status();

        let body = response.text()?;
        println!("response: {:?}", body);
        if !status_code.is_success() {
            let status = match status_code.as_u16() {
                401 => ApiException::InvalidToken,
                429 => ApiException::TooManyRequests,
                403 => ApiException::ApplicationNotRegistered,
                _ => ApiException::UnknownError,
            };
            eprintln!(
                "Error HTTP {} ({:?}): {}",
                status_code.as_str(),
                status,
                body
            );
            return Err(anyhow::Error::msg(format!("Request failed: {}", 42)));
        }

        Ok(body)
    }
}
