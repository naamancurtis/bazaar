use anyhow::Result;
use jsonwebtoken::{dangerous_insecure_decode, TokenData};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{header::HeaderMap, Client, ClientBuilder};
use serde_json::{json, Value};
use std::time::Duration;

use bazaar::models::Claims;

use crate::helpers::{CustomerData, TOKEN_GRAPHQL_FIELDS};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub struct Cookies {
    pub access: Option<TokenData<Claims>>,
    pub refresh: Option<TokenData<Claims>>,
}

pub struct Response {
    pub data: Value,
    pub cookies: Cookies,
}

lazy_static! {
    static ref COOKIE_TOKEN: Regex =
        Regex::new("^(?P<token_type>[A-Z]+)=(?P<token>[a-zA-z0-9/.-]+);")
            .expect("Regex should be valid");
}

pub fn build_http_client() -> Result<Client> {
    let client = ClientBuilder::new()
        .cookie_store(true)
        .user_agent(APP_USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()?;
    Ok(client)
}

pub async fn send_request(client: &Client, address: &str, body: &Value) -> Result<Response> {
    let response = client.post(address).json(body).send().await?;

    let headers = response.headers();
    let cookies = parse_cookies(&headers);
    let data = response.json::<serde_json::Value>().await?;

    eprintln!("{:#?}", &data);
    Ok(Response { data, cookies })
}

pub async fn get_anonymous_token(client: &Client, address: &str) -> Result<CustomerData> {
    let graphql_mutatation = format!(
        r#"
        mutation anonymousLogin {{
            anonymousLogin {{
                {}
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let response = client.post(address).json(&body).send().await?;

    let headers = response.headers();
    let cookies = parse_cookies(&headers);
    let (public_id, cart_id) = if let Some(access) = cookies.access {
        (access.claims.sub, Some(access.claims.cart_id))
    } else {
        (None, None)
    };

    let data = response.json::<serde_json::Value>().await?;
    eprintln!("{:#?}", &data);
    let customer = CustomerData {
        public_id,
        private_id: None,
        cart_id,
        first_name: None,
        last_name: None,
        email: None,
        password: None,
    };
    Ok(customer)
}

pub async fn sign_user_up_and_get_known_token(
    client: &Client,
    address: &str,
) -> Result<CustomerData> {
    let graphql_mutatation = format!(
        r#"
        mutation signUp($email: String!, $password: String!, $firstName: String!, $lastName: String!) {{
            signUp(email: $email, password: $password, firstName: $firstName, lastName: $lastName) {{
                {}
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS
    );

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

    let response = client.post(address).json(&body).send().await?;

    let headers = response.headers();
    let cookies = parse_cookies(&headers);
    let (public_id, cart_id) = if let Some(access) = cookies.access {
        (access.claims.sub, Some(access.claims.cart_id))
    } else {
        (None, None)
    };

    let data = response.json::<serde_json::Value>().await?;
    eprintln!("{:#?}", &data);

    let customer = CustomerData {
        public_id,
        private_id: None,
        cart_id,
        first_name: Some(first_name.to_owned()),
        last_name: Some(last_name.to_owned()),
        email: Some(email.to_owned()),
        password: Some(password.to_owned()),
    };

    Ok(customer)
}

fn parse_cookies(headers: &HeaderMap) -> Cookies {
    let cookies = headers.get_all("set-cookie");
    let mut access_token = String::default();
    let mut refresh_token = String::default();
    for c in cookies {
        let cookie = COOKIE_TOKEN.captures(c.to_str().unwrap()).unwrap();
        if cookie.name("token_type").unwrap().as_str() == "ACCESS" {
            access_token = cookie.name("token").unwrap().as_str().to_string();
        }
        if cookie.name("token_type").unwrap().as_str() == "REFRESH" {
            refresh_token = cookie.name("token").unwrap().as_str().to_string();
        }
    }
    if access_token != String::default() {
        assert_ne!(access_token, refresh_token);
    }
    let access_token: Option<TokenData<Claims>> = dangerous_insecure_decode(&access_token).ok();
    let refresh_token: Option<TokenData<Claims>> = dangerous_insecure_decode(&refresh_token).ok();
    Cookies {
        access: access_token,
        refresh: refresh_token,
    }
}
