use anyhow::Result;
use assert_json_diff::assert_json_include;
use chrono::DateTime;
use claim::assert_some;
use serde_json::json;
use uuid::Uuid;

use bazaar::{
    database::{CartItemDatabase, CustomerDatabase, ShoppingCartDatabase},
    models::{cart_item::InternalCartItem, Customer, ShoppingCart},
};

mod helpers;
use helpers::*;

// @TODO Add in tests for Refresh

#[actix_rt::test]
async fn mutation_sign_up_without_token_works() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;

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

    let email = "007@test.com";
    let first_name = "James";
    let last_name = "Bond";

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
            "password": Uuid::nil()
        }
    });

    let response = send_request(&client, &app.address, &body).await?;
    let tokens = response.data["data"]["signUp"].clone();

    let issued_at = &tokens["issuedAt"];
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);

    let new_customer =
        Customer::find_by_email::<CustomerDatabase>(email.to_string(), &app.db_pool).await?;
    assert_eq!(&new_customer.first_name, first_name);
    assert_eq!(&new_customer.last_name, last_name);
    // The private ID should not be exposed publically
    assert_ne!(
        response
            .cookies
            .access
            .expect("after signing up there should be a valid token")
            .claims
            .sub,
        Some(new_customer.id)
    );

    Ok(())
}

#[actix_rt::test]
async fn mutation_sign_up_with_anonymous_token_works() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let anon_customer = get_anonymous_token(&client, &app.address).await?;

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

    let email = "007@test.com";
    let first_name = "James";
    let last_name = "Bond";

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
            "password": Uuid::nil()
        }
    });

    let response = send_request(&client, &app.address, &body).await?;
    let returned_tokens = response.data["data"]["signUp"].clone();

    let issued_at = &returned_tokens["issuedAt"];
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);

    let new_customer =
        Customer::find_by_email::<CustomerDatabase>(email.to_string(), &app.db_pool).await?;

    let access_claims = response
        .cookies
        .access
        .expect("after signing up there should be a valid token")
        .claims;
    assert_eq!(&new_customer.first_name, first_name);
    assert_eq!(&new_customer.last_name, last_name);
    // The private ID should not be exposed publically
    assert_ne!(access_claims.sub, Some(new_customer.id));
    // The cart should have been promoted, so they both should be the same
    assert_eq!(access_claims.cart_id, anon_customer.cart_id.unwrap());

    Ok(())
}

#[actix_rt::test]
async fn mutation_sign_up_with_known_tokens_should_error() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let _customer = sign_user_up_and_get_known_token(&client, &app.address).await?;

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

    let email = "007@test.com";
    let first_name = "James";
    let last_name = "Bond";

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
            "password": Uuid::nil()
        }
    });

    let response = send_request(&client, &app.address, &body).await?;
    let errors = response.data["errors"].clone();
    assert_json_include!(
        actual: errors,
        expected: json!([{
            "message": "Bad Request: Customer already exists",
            "extensions": {
                "status": 400,
                "statusText": "BAD_REQUEST"
            }
        }])
    );

    Ok(())
}

#[actix_rt::test]
async fn mutation_login_with_valid_credentials_and_no_tokens_works() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let customer_details = insert_default_customer(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": customer_details.email.clone().unwrap(),
            "password": customer_details.password.unwrap()
        }
    });

    let response = send_request(&client, &app.address, &body).await?;
    let returned_tokens = response.data["data"]["login"].clone();

    let issued_at = &returned_tokens["issuedAt"];
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);
    assert_some!(response.cookies.access);
    assert_some!(response.cookies.refresh);

    Ok(())
}

// @TODO need to verify that the carts are merged correctly
#[actix_rt::test]
async fn mutation_login_with_valid_credentials_and_anonymous_tokens_works() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let _anon_customer = get_anonymous_token(&client, &app.address).await?;
    let customer_details = insert_default_customer(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": customer_details.email.clone().unwrap(),
            "password": customer_details.password.unwrap()
        }
    });

    let response = send_request(&client, &app.address, &body).await?;
    let returned_tokens = response.data["data"]["login"].clone();

    let issued_at = &returned_tokens["issuedAt"];
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);
    assert_some!(response.cookies.access);
    assert_some!(response.cookies.refresh);

    Ok(())
}

#[actix_rt::test]
async fn mutation_login_with_already_logged_in_customer_errors() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let _customer = sign_user_up_and_get_known_token(&client, &app.address).await?;

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": "random@email.com",
            "password": "randomPassw0rd!"
        }
    });

    let response = send_request(&client, &app.address, &body).await?;
    let errors = response.data["errors"].clone();

    assert_json_include!(
        actual: errors,
        expected: json!([{
            "message": "Bad Request: Customer already has valid tokens",
            "extensions": {
                "status": 400,
                "statusText": "BAD_REQUEST"
            }
        }])
    );
    Ok(())
}

#[actix_rt::test]
async fn mutation_login_with_non_existent_customer_errors() -> Result<()> {
    let app = spawn_app().await;
    let _customer_details = insert_default_customer(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": "random@email.com",
            "password": "randomPassw0rd!"
        }
    });

    let client = build_http_client()?;
    get_anonymous_token(&client, &app.address).await?;

    // [ No tokens | Anonymous Tokens ]
    // One client holds no tokens in cookies, the other holds anonymous tokens
    let test_cases = vec![build_http_client()?, client];

    for case in test_cases.into_iter() {
        let response = send_request(&case, &app.address, &body).await?;
        let errors = response.data["errors"].clone();

        assert_json_include!(
            actual: errors,
            expected: json!([{
                "message": "Could not find resource",
                "extensions": {
                    "status": 404,
                    "statusText": "NOT_FOUND"
                }
            }])
        );
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_login_with_invalid_credentials_errors() -> Result<()> {
    let app = spawn_app().await;
    let _customer_details = insert_default_customer(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation login($email: String!, $password: String!) {{
            login(email: $email, password: $password) {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": "imbatman@test.com",
            "password": "randomPassw0rd!"
        }
    });

    let client = build_http_client()?;
    get_anonymous_token(&client, &app.address).await?;

    // [ No tokens | Anonymous Tokens ]
    // One client holds no tokens in cookies, the other holds anonymous tokens
    let test_cases = vec![build_http_client()?, client];

    for case in test_cases.into_iter() {
        let response = send_request(&case, &app.address, &body).await?;
        let errors = response.data["errors"].clone();

        assert_json_include!(
            actual: errors,
            expected: json!([{
                "message": "Incorrect credentials provided",
                "extensions": {
                    "status": 401,
                    "statusText": "UNAUTHORIZED"
                }
            }])
        );
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_anonymous_login_works() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;

    let graphql_mutatation = format!(
        r#"
        mutation anonymousLogin {{
            anonymousLogin{{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let response = send_request(&client, &app.address, &body).await?;
    let returned_tokens = response.data["data"]["anonymousLogin"].clone();

    let issued_at = &returned_tokens["issuedAt"];
    assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);
    assert_some!(response.cookies.access);
    assert_some!(response.cookies.refresh);

    Ok(())
}

#[actix_rt::test]
async fn mutation_update_customer_works() -> Result<()> {
    let app = spawn_app().await;
    let client = build_http_client()?;
    let _customer = sign_user_up_and_get_known_token(&client, &app.address).await?;

    let graphql_mutatation = format!(
        r#"
        mutation updateCustomer($update: [CustomerUpdate!]!) {{
            updateCustomer(update: $update) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let generate_json = |key: &str, value: &str| -> serde_json::Value {
        json!({
            "key": key,
            "value": value
        })
    };

    let test_cases = vec![
        json!([generate_json("email", "updated@test.com")]),
        json!([generate_json("firstName", "Updated")]),
        json!([generate_json("lastName", "Updated")]),
        json!([
            generate_json("email", "deadpool@troll.com"),
            generate_json("firstName", "Mr"),
            generate_json("lastName", "Pool")
        ]),
    ];
    let expected = vec![
        json!({
            "firstName": "Clark",
            "lastName": "Kent",
            "email": "updated@test.com"
        }),
        json!({
            "firstName": "Updated",
            "lastName": "Kent",
            "email": "updated@test.com"
        }),
        json!({
            "firstName": "Updated",
            "lastName": "Updated",
            "email": "updated@test.com"
        }),
        json!({
            "firstName": "Mr",
            "lastName": "Pool",
            "email": "deadpool@troll.com"
        }),
    ];

    for (case, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let body = json!({
            "query": graphql_mutatation,
            "variables": {
                "update": case
            }
        });
        let response = send_request(&client, &app.address, &body).await?;
        let data = response.data["data"]["updateCustomer"].clone();

        assert_json_include!(actual: &data, expected: expected);

        let last_modified = DateTime::parse_from_rfc3339(&data["lastModified"].as_str().unwrap())
            .expect("date should parse correctly with rfc3339");
        let created_at = DateTime::parse_from_rfc3339(&data["createdAt"].as_str().unwrap())
            .expect("date should parse correctly with rfc3339");

        assert!(last_modified > created_at);
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_update_customer_without_known_token_errors() -> Result<()> {
    let app = spawn_app().await;

    let graphql_mutatation = format!(
        r#"
        mutation updateCustomer($update: [CustomerUpdate!]!) {{
            updateCustomer(update: $update) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let update = json!([{
        "key": "firstName",
        "value": "Clark"
    }]);

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "update": update
            }
    });

    let client = build_http_client()?;
    get_anonymous_token(&client, &app.address).await?;

    // [ No tokens | Anonymous Tokens ]
    // One client holds no tokens in cookies, the other holds anonymous tokens
    let test_cases = vec![build_http_client()?, client];
    let expected = vec![
        json!([{
            "message": "Invalid token provided",
            "extensions": {
                "status": 401,
                "statusText": "INVALID_TOKEN",
                "details": "No token was found"
            }
        }]),
        json!([{
            "message": "Anonymous users do not have access to this resource",
            "extensions": {
                "status": 401,
                "statusText": "UNAUTHORIZED"
            }
        }]),
    ];

    for (client, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let response = send_request(&client, &app.address, &body).await?;
        let errors = response.data["errors"].clone();
        assert_json_include!(actual: errors, expected: expected);
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_add_item_to_cart_works() -> Result<()> {
    let app = spawn_app().await;
    let anon_client = build_http_client()?;
    let anon_customer = get_anonymous_token(&anon_client, &app.address).await?;
    let known_client = build_http_client()?;
    let known_customer = sign_user_up_and_get_known_token(&known_client, &app.address).await?;

    let graphql_mutatation = format!(
        r#"
        mutation addItemsToCart($newItems: [UpdateCartItem!]!) {{
            addItemsToCart(newItems: $newItems) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "newItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    let test_cases = vec![anon_client, known_client];

    let expected = vec![
        json!({
            "id": anon_customer.cart_id.unwrap(),
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "items": [{
                "sku": "12345678",
                "quantity": 3,
                "name": "Item 1",
                "tags": []
            }],
        }),
        json!({
            "id": known_customer.cart_id.unwrap(),
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": [{
                "sku": "12345678",
                "quantity": 3,
                "name": "Item 1",
                "tags": []
            }],
        }),
    ];

    for (client, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let response = send_request(&client, &app.address, &body).await?;
        let cart = response.data["data"]["addItemsToCart"].clone();

        assert_json_include!(actual: &cart, expected: &expected);
        assert_on_decimal(cart["priceBeforeDiscounts"].as_f64().unwrap(), 2.97);
        assert_on_decimal(cart["priceAfterDiscounts"].as_f64().unwrap(), 2.97);
        assert_on_decimal(cart["items"][0]["pricePerUnit"].as_f64().unwrap(), 0.99);

        let cart = ShoppingCart::find_by_id::<ShoppingCartDatabase>(
            Uuid::parse_str(expected["id"].as_str().expect("should have valid UUID"))
                .expect("should be valid UUID"),
            &app.db_pool,
        )
        .await
        .expect("should be able to fetch cart");
        assert_eq!(cart.items.len(), 1);
        assert_on_decimal(cart.price_before_discounts, 2.97);
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_remove_item_from_cart_completely_removes_negative_quantities() -> Result<()> {
    let app = spawn_app().await;

    let anon_client = build_http_client()?;
    let anon_customer = get_anonymous_token(&anon_client, &app.address).await?;
    let anon_cart_id = anon_customer.cart_id.clone().unwrap();

    let known_client = build_http_client()?;
    let known_customer = sign_user_up_and_get_known_token(&known_client, &app.address).await?;
    let known_cart_id = known_customer.cart_id.clone().unwrap();
    assert_ne!(anon_cart_id, known_cart_id);

    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        anon_cart_id,
        vec![InternalCartItem {
            sku: "12345678".to_string(),
            quantity: 1,
        }],
        &app.db_pool,
    )
    .await?;

    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        known_cart_id,
        vec![InternalCartItem {
            sku: "12345678".to_string(),
            quantity: 1,
        }],
        &app.db_pool,
    )
    .await?;

    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let graphql_mutatation = format!(
        r#"
        mutation removeItemsFromCart($removedItems: [UpdateCartItem!]!) {{
            removeItemsFromCart(removedItems: $removedItems) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    // This update would actually set the quantity to -2
    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "removedItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    let test_cases = vec![anon_client, known_client];

    let expected = vec![
        json!({
            "id": anon_cart_id,
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "items": [],
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0
        }),
        json!({
            "id": known_cart_id,
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": [],
            "priceBeforeDiscounts": 0.0,
            "priceAfterDiscounts": 0.0
        }),
    ];

    for (client, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let response = send_request(&client, &app.address, &body).await?;
        let cart = response.data["data"]["removeItemsFromCart"].clone();

        assert_json_include!(actual: &cart, expected: &expected);

        let cart = ShoppingCart::find_by_id::<ShoppingCartDatabase>(
            Uuid::parse_str(expected["id"].as_str().expect("should have valid UUID"))
                .expect("should be valid UUID"),
            &app.db_pool,
        )
        .await
        .expect("should be able to fetch cart");
        assert!(cart.items.is_empty());
        assert!(cart.price_after_discounts == 0f64);
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_remove_items_from_cart_correctly_handles_leftover_items() -> Result<()> {
    let app = spawn_app().await;

    let anon_client = build_http_client()?;
    let anon_customer = get_anonymous_token(&anon_client, &app.address).await?;
    let anon_cart_id = anon_customer.cart_id.clone().unwrap();

    let known_client = build_http_client()?;
    let known_customer = sign_user_up_and_get_known_token(&known_client, &app.address).await?;
    let known_cart_id = known_customer.cart_id.clone().unwrap();
    assert_ne!(anon_cart_id, known_cart_id);

    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        anon_cart_id,
        vec![
            InternalCartItem {
                sku: "12345678".to_string(),
                quantity: 5,
            },
            InternalCartItem {
                sku: "22345678".to_string(),
                quantity: 2,
            },
        ],
        &app.db_pool,
    )
    .await?;

    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let cart = ShoppingCart::edit_cart_items::<ShoppingCartDatabase, CartItemDatabase>(
        known_cart_id,
        vec![
            InternalCartItem {
                sku: "12345678".to_string(),
                quantity: 5,
            },
            InternalCartItem {
                sku: "22345678".to_string(),
                quantity: 2,
            },
        ],
        &app.db_pool,
    )
    .await?;

    assert!(!cart.items.is_empty());
    assert!(cart.price_before_discounts > 0f64);

    let graphql_mutatation = format!(
        r#"
        mutation removeItemsFromCart($removedItems: [UpdateCartItem!]!) {{
            removeItemsFromCart(removedItems: $removedItems) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "removedItems": [{
                "sku": "12345678",
                "quantity": 3
            }]
        }
    });

    let test_cases = vec![anon_client, known_client];

    let expected = vec![
        json!({
            "id": anon_cart_id,
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "items": [
                {
                    "sku": "12345678",
                    "quantity": 2,
                    "name": "Item 1",
                    "tags": []
                },
                {
                    "sku": "22345678",
                    "quantity": 2,
                    "name": "Item 2",
                    "tags": []
                }
            ],
        }),
        json!({
            "id": known_cart_id,
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": [
                {
                    "sku": "12345678",
                    "quantity": 2,
                    "name": "Item 1",
                    "tags": []
                },
                {
                    "sku": "22345678",
                    "quantity": 2,
                    "name": "Item 2",
                    "tags": []
                }
            ],
        }),
    ];

    for (client, expected) in test_cases.into_iter().zip(expected.into_iter()) {
        let response = send_request(&client, &app.address, &body).await?;
        let cart = response.data["data"]["removeItemsFromCart"].clone();

        assert_json_include!(actual: &cart, expected: &expected);
        assert_on_decimal(cart["priceBeforeDiscounts"].as_f64().unwrap(), 22.98);
        assert_on_decimal(cart["priceAfterDiscounts"].as_f64().unwrap(), 22.98);
        assert_on_decimal(cart["items"][0]["pricePerUnit"].as_f64().unwrap(), 0.99);
        assert_on_decimal(cart["items"][1]["pricePerUnit"].as_f64().unwrap(), 10.50);

        let cart = ShoppingCart::find_by_id::<ShoppingCartDatabase>(
            Uuid::parse_str(expected["id"].as_str().expect("should have valid UUID"))
                .expect("should be valid UUID"),
            &app.db_pool,
        )
        .await
        .expect("should be able to fetch cart");
        assert_eq!(cart.items.len(), 2);
        assert!(cart.price_before_discounts < 23.0);
    }

    Ok(())
}

#[actix_rt::test]
async fn mutation_refresh_works() -> Result<()> {
    let app = spawn_app().await;
    let anon_client = build_http_client()?;
    let anon_customer = get_anonymous_token(&anon_client, &app.address).await?;
    let known_client = build_http_client()?;
    let known_customer = sign_user_up_and_get_known_token(&known_client, &app.address).await?;

    let graphql_mutatation = format!(
        r#"
        mutation refresh {{
            refresh {{
               {} 
            }}
        }}
    "#,
        TOKEN_GRAPHQL_FIELDS,
    );

    let body = json!({
        "query": graphql_mutatation,
    });

    let cases = vec![anon_client, known_client];
    let cmp_tokens = vec![
        (
            anon_customer.raw_access_token,
            anon_customer.raw_refresh_token,
        ),
        (
            known_customer.raw_access_token,
            known_customer.raw_refresh_token,
        ),
    ];

    for (client, (access, refresh)) in cases.into_iter().zip(cmp_tokens.into_iter()) {
        let response = send_request(&client, &app.address, &body).await?;
        let returned_tokens = response.data["data"]["refresh"].clone();

        let issued_at = &returned_tokens["issuedAt"];
        assert!(issued_at.as_u64().expect("should have valid number") > 1_000_000);
        assert_some!(response.cookies.access);
        assert_some!(response.cookies.refresh);

        // Due the timer on refresh tokens, the access token should be refreshed
        // but the refresh token should not have been
        assert_ne!(response.cookies.raw_access, access);
        assert_eq!(response.cookies.raw_refresh, refresh);
    }

    Ok(())
}
