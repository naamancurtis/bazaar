use anyhow::Result;
use assert_json_diff::assert_json_include;
use serde_json::json;

mod helpers;
use helpers::*;

#[actix_rt::test]
async fn query_customer_fails_for_anonymous_user() -> Result<()> {
    let app = spawn_app().await;
    let tokens = get_anonymous_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        query customer {{
            customer {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let data = send_request(&app.address, Some(&tokens.tokens.access_token), body).await?;

    let errors = data["errors"].clone();
    assert_json_include!(
        actual: errors,
        expected: json!([{
            "message": "Anonymous users do not have access to this resource",
            "extensions": {
                "status": 401,
                "statusText": "UNAUTHORIZED"
            }
        }])
    );
    Ok(())
}

#[actix_rt::test]
async fn query_customer_works_for_known_user() -> Result<()> {
    let app = spawn_app().await;
    let tokens = get_known_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        query customer {{
            customer {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let data = send_request(&app.address, Some(&tokens.tokens.access_token), body).await?;
    let data = data["data"]["customer"].clone();
    assert_json_include!(
        actual: data,
        expected: json!({
            "id": tokens.customer.id.unwrap(),
            "firstName": "Bruce",
            "lastName": "Wayne",
            "email": "imbatman@test.com",
        })
    );
    Ok(())
}

#[actix_rt::test]
async fn query_customer_with_known_user_includes_cart() -> Result<()> {
    let app = spawn_app().await;
    let tokens = get_known_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        query customer {{
            customer {{
                {}
                cart {{
                    {}
                }}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS, SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let data = send_request(&app.address, Some(&tokens.tokens.access_token), body).await?;
    let data = data["data"]["customer"].clone();
    assert_json_include!(
        actual: data,
        expected: json!({
            "id": tokens.customer.id.unwrap(),
            "firstName": "Bruce",
            "lastName": "Wayne",
            "email": "imbatman@test.com",
            "cart": {
                "id": tokens.customer.cart_id,
                "currency": "GBP",
                "cartType": "KNOWN",
                "priceBeforeDiscounts": 0.0,
                "priceAfterDiscounts": 0.0,
                "items": [],
            }
        })
    );
    Ok(())
}

#[actix_rt::test]
async fn query_cart_works_for_anonymous_user() -> Result<()> {
    let app = spawn_app().await;
    let tokens = get_anonymous_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        query cart {{
            cart {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let data = send_request(&app.address, Some(&tokens.tokens.access_token), body).await?;
    let data = data["data"]["cart"].clone();

    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": tokens.cart_id,
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0,
            "items": [],
        })
    );

    Ok(())
}

#[actix_rt::test]
async fn query_cart_works_for_known_user() -> Result<()> {
    let app = spawn_app().await;
    let tokens = get_known_token(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        query cart {{
            cart {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let data = send_request(&app.address, Some(&tokens.tokens.access_token), body).await?;
    let data = data["data"]["cart"].clone();

    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": tokens.customer.cart_id.unwrap(),
            "currency": "GBP",
            "cartType": "KNOWN",
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0,
            "items": [],
        })
    );

    Ok(())
}
