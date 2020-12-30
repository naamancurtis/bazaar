use anyhow::Result;
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client,
};
use serde_json::Value;

pub async fn send_request(address: &str, token: Option<&str>, body: Value) -> Result<Value> {
    let client = Client::new();
    let access_token = if let Some(token) = token {
        format!("Bearer {}", token)
    } else {
        String::default()
    };
    let mut request = client.post(address);
    if !access_token.is_empty() {
        request = request.header(AUTHORIZATION, HeaderValue::from_str(&access_token)?);
    }
    let response = request.json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?;
    eprintln!("{:#?}", &data);
    Ok(data)
}
