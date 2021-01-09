use anyhow::Result;
use reqwest::{Client, ClientBuilder};
use serde_json::{json, Value};
use std::time::Duration;

use crate::helpers::CustomerData;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub fn build_http_client() -> Result<Client> {
    let client = ClientBuilder::new()
        .cookie_store(true)
        .user_agent(APP_USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()?;
    Ok(client)
}

pub async fn send_request(client: &Client, address: &str, body: &Value) -> Result<Value> {
    let response = client.post(address).json(body).send().await?;
    let data = response.json::<serde_json::Value>().await?;
    eprintln!("{:#?}", &data);
    Ok(data)
}

pub async fn get_anonymous_token(client: &Client, address: &str) -> Result<()> {
    let graphql_mutatation = r#"
        mutation anonymousLogin {{
            anonymousLogin
        }}
    "#;

    let body = json!({
        "query": graphql_mutatation,
    });

    client.post(address).json(&body).send().await?;
    Ok(())
}

pub async fn sign_user_up_and_get_known_token(
    client: &Client,
    address: &str,
) -> Result<CustomerData> {
    let graphql_mutatation = r#"
        mutation signUp($email: String!, $password: String!, $firstName: String!, $lastName: String!) {{
            signUp(email: $email, password: $password, firstName: $firstName, lastName: $lastName) 
        }}
    "#;

    let email = "superman@test.com";
    let first_name = "Clark";
    let last_name = "Kent";
    let password = "l3xSucks!";

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
            "password": password
        }
    });

    let sign_up_response = send_request(client, address, &body).await?;
    eprintln!("{:#?}", &sign_up_response);

    let customer = CustomerData {
        public_id: None,
        private_id: None,
        cart_id: None,
        email: Some(email.to_owned()),
        password: Some(password.to_owned()),
    };

    Ok(customer)
}
