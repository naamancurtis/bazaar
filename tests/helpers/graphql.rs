use anyhow::Result;
use claim::{assert_none, assert_some};
use reqwest::Response;
use serde_json::{to_string_pretty, Value};

pub async fn parse_graphql_response(response: Response) -> Result<Value> {
    let response = response.json::<Value>().await?;
    if let Some(errors) = response.get("errors") {
        eprintln!("Found Errors: {:?}", to_string_pretty(errors));
        assert_none!(errors.get(0));
    }

    let data = response.get("data");
    assert_some!(data);
    let data = data.unwrap().clone();
    Ok(data)
}
