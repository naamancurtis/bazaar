use anyhow::Result;
// use assert_json_diff::assert_json_include;
use reqwest::{redirect::Policy, ClientBuilder, StatusCode};
use serde_json::json;
use std::time::Duration;

mod helpers;
use helpers::*;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

// @TODO - Add this back in once login is correctly setting the refresh cookie
// #[actix_rt::test]
// async fn refresh_anonymous_works() -> Result<()> {
//     let app = spawn_app().await;

//     let client = ClientBuilder::new()
//         .user_agent(APP_USER_AGENT)
//         .timeout(Duration::from_secs(5))
//         .cookie_store(true)
//         .redirect(Policy::none())
//         .build()?;

//     let graphql_mutatation = format!(
//         r#"
//         mutation anonymousLogin {{
//             anonymousLogin{{
//                {}
//             }}
//         }}
//     "#,
//         TOKEN_GRAPHQL_FIELDS,
//     );

//     let body = json!({
//         "query": graphql_mutatation,
//     });

//     let response = client.post(&app.address).json(&body).send().await?;
//     dbg!(&response.cookies().collect::<Vec<Cookie>>());
//     let response = response.json::<Value>().await?;
//     // eprintln!("GraphQL Response {:#?}", &response);
//     let returned_tokens = response["data"]["anonymousLogin"].clone();

//     let body = json!({
//         "loginRedirectUrl": "/login"
//     });

//     let response = client
//         .post(&format!("{}/refresh", &app.address))
//         .json(&body);
//     // dbg!(&response);
//     let response = response.send().await?;
//     // eprintln!("Refresh Response: {:#?}", &response);

//     assert!(response.status().is_success());
//     let refreshed_tokens = response.json::<Value>().await?;
//     assert_json_include!(actual: refreshed_tokens, expected: returned_tokens);

//     Ok(())
// }

#[actix_rt::test]
async fn refresh_anonymous_redirects_if_no_token_is_found() -> Result<()> {
    let app = spawn_app().await;

    let client = ClientBuilder::new()
        .user_agent(APP_USER_AGENT)
        .timeout(Duration::from_secs(5))
        .cookie_store(true)
        .redirect(Policy::none())
        .build()?;

    let body = json!({
        "loginRedirectUrl": "/login"
    });

    let response = client
        .post(&format!("{}/refresh", &app.address))
        .json(&body);
    dbg!(&response);
    let response = response.send().await?;
    // eprintln!("Refresh Response: {:#?}", &response);
    let location = response
        .headers()
        .get("Location")
        .expect("Location header should be set");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        location,
        &body["loginRedirectUrl"].as_str().expect("should exist")
    );

    Ok(())
}
