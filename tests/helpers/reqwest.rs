use anyhow::Result;
use reqwest::ClientBuilder;
use serde_json::Value;
use std::time::Duration;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub async fn send_request(address: &str, token: Option<&str>, body: Value) -> Result<Value> {
    let client = ClientBuilder::new()
        .user_agent(APP_USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()?;
    let mut request = client.post(address);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?;
    eprintln!("{:#?}", &data);
    Ok(data)
}
