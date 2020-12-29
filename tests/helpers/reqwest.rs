use anyhow::Result;
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client,
};
use serde_json::Value;

pub async fn send_request(address: &str, token: &str, body: Value) -> Result<Value> {
    let client = Client::new();
    let access_token = format!("Bearer {}", token);
    let response = client
        .post(address)
        .header(AUTHORIZATION, HeaderValue::from_str(&access_token)?)
        .json(&body)
        .send()
        .await?;
    let data = response.json::<serde_json::Value>().await?;
    eprintln!("{:#?}", &data);
    Ok(data)
}
