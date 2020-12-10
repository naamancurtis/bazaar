use anyhow::Result;
use reqwest::Response;
use serde_json::{to_string_pretty, Value};

pub async fn parse_graphql_response(response: Response) -> Result<Value> {
    let response = response.json::<Value>().await?;
    if let Some(errors) = response.get("errors") {
        eprintln!("Found Errors: {:?}", to_string_pretty(errors));
        assert!(errors.get(0).is_none());
    }

    let data = response.get("data");
    assert!(data.is_some());
    let data = data.unwrap().clone();
    Ok(data)
}
