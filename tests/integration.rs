use anyhow::Result;
use assert_json_diff::assert_json_include;
use chrono::DateTime;
use lazy_static::lazy_static;
use serde_json::json;
use uuid::Uuid;

use bazaar::{
    configuration::DatabaseSettings,
    models::{Currency, Customer},
    telemetry::{generate_subscriber, init_subscriber},
};

const CUSTOMER_GRAPHQL_FIELDS: &str = "#
id,
firstName,
lastName,
email,
createdAt,
lastModified
#";

const SHOPPING_CART_GRAPHQL_FIELDS: &str = "#
id
cartType
items {
   sku 
}
priceBeforeDiscounts
discounts
priceAfterDiscounts
currency
lastModified
createdAt
#";

lazy_static! {
    /// To ensure logs are only outputted in tests when requred, by default
    /// tests run with no logs being captured
    ///
    /// In order to set logs to be captured during tests run them with:
    /// `TEST_LOG=true cargo test health_check_works | bunyan`
    static ref TRACING: () = {
        let filter = if std::env::var("TEST_LOG").is_ok() {
            "debug"
        } else {
            ""
        };
        let subscriber = generate_subscriber("test".to_string(), filter.into());
        init_subscriber(subscriber);
    };

    static ref DEFAULT_CUSTOMER: serde_json::Value = {
        json!({
            "email": format!("{}@test.com", Uuid::nil()),
            "firstName": Uuid::nil(),
            "lastName": Uuid::nil()
        })
    };
}

#[actix_rt::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn mutation_create_customer_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let graphql_mutatation = format!(
        r#"
        mutation createCustomer($email: String!, $firstName: String!, $lastName: String!) {{
            createCustomer(email: $email, firstName: $firstName, lastName: $lastName) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let email = format!("{}@test.com", Uuid::new_v4());
    let first_name = Uuid::new_v4();
    let last_name = Uuid::new_v4();

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": email,
            "firstName": first_name,
            "lastName": last_name
        }
    });
    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["createCustomer"].clone();

    assert_json_include!(
        actual: data,
        expected: json!({
            "email": email,
            "firstName": first_name,
            "lastName": last_name,
        })
    );
    Ok(())
}

#[actix_rt::test]
async fn query_customer_by_id_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let customer_id = insert_default_customer(&app.db_pool)
        .await?
        .customer
        .unwrap();

    let graphql_mutatation = format!(
        r#"
        query customerById($id: UUID!) {{
            customerById(id: $id) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "id": customer_id,
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["customerById"].clone();
    assert_json_include!(actual: data, expected: DEFAULT_CUSTOMER.clone());
    Ok(())
}

#[actix_rt::test]
async fn query_customer_by_email_works() -> Result<(), Box<dyn std::error::Error>> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    insert_default_customer(&app.db_pool)
        .await
        .expect("default customer failed to be created");

    let graphql_mutatation = format!(
        r#"
        query customerByEmail($email: String!) {{
            customerByEmail(email: $email) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "email": format!("{}@test.com", Uuid::nil()),
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["customerByEmail"].clone();
    assert_json_include!(actual: data, expected: DEFAULT_CUSTOMER.clone());
    Ok(())
}

#[actix_rt::test]
async fn mutation_update_customer_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let customer_id = insert_default_customer(&app.db_pool)
        .await?
        .customer
        .unwrap();

    let graphql_mutatation = format!(
        r#"
        mutation updateCustomer($id: UUID!, $update: CustomerUpdate) {{
            updateCustomer(id: $id, update: $update) {{
                {}
            }}
        }}
    "#,
        CUSTOMER_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "id": customer_id,
            "update": {
                "email": "updated@test.com",
                "firstName": "updated",
                "lastName": "updated"
            }
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["updateCustomer"].clone();
    dbg!(&data);

    assert_json_include!(
        actual: &data,
        expected: json!({
            "firstName": "updated",
            "lastName": "updated",
            "email": "updated@test.com"
        })
    );

    let last_modified = DateTime::parse_from_rfc3339(&data["lastModified"].as_str().unwrap())
        .expect("date should parse correctly with rfc3339");
    let created_at = DateTime::parse_from_rfc3339(&data["createdAt"].as_str().unwrap())
        .expect("date should parse correctly with rfc3339");

    assert!(last_modified > created_at);
    Ok(())
}

#[actix_rt::test]
async fn mutation_create_anonymous_cart_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let graphql_mutatation = format!(
        r#"
        mutation createAnonymousCart($currency: Currency!) {{
            createAnonymousCart(currency: $currency) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "currency": "GBP",
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;
    let data = response.json::<serde_json::Value>().await?["data"]["createAnonymousCart"].clone();
    assert_json_include!(
        actual: data,
        expected: json!({
            "currency": "GBP",
            "cartType": "ANONYMOUS",
            "items": []
        })
    );

    Ok(())
}

#[actix_rt::test]
async fn mutation_create_known_cart_works() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let customer = insert_default_customer(&app.db_pool)
        .await?
        .customer
        .unwrap();

    let graphql_mutatation = format!(
        r#"
        mutation createKnownCart($id: UUID!, $currency: Currency!) {{
            createKnownCart(id: $id, currency: $currency) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "currency": "GBP",
            "id": customer
        }
    });

    dbg!(&body);
    let response = client.post(&app.address).json(&body).send().await?;
    dbg!(&response);
    let data = response.json::<serde_json::Value>().await?["data"]["createKnownCart"].clone();
    assert_json_include!(
        actual: &data,
        expected: json!({
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": []
        })
    );

    let customer = Customer::find_by_id(customer, &app.db_pool)
        .await
        .expect("failed to query customer from the database")
        .expect("failed to find customer");
    assert_eq!(
        customer.cart_id.expect("customer should have cart id"),
        Uuid::parse_str(&data["id"].as_str().expect("Cart should always have an ID"))
            .expect("cart id should be valid UUID")
    );
    Ok(())
}

#[actix_rt::test]
async fn mutation_create_known_cart_doesnt_recreate_existing_cart() -> Result<()> {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let ids = insert_default_customer_with_cart(&app.db_pool).await?;

    let graphql_mutatation = format!(
        r#"
        mutation createKnownCart($id: UUID!, $currency: Currency!) {{
            createKnownCart(id: $id, currency: $currency) {{
                {}
            }}
        }}
    "#,
        SHOPPING_CART_GRAPHQL_FIELDS
    );

    let body = json!({
        "query": graphql_mutatation,
        "variables": {
            "currency": "GBP",
            "id": ids.customer.unwrap()
        }
    });

    dbg!(&body);

    let response = client.post(&app.address).json(&body).send().await?;

    dbg!(&response);
    let data = response.json::<serde_json::Value>().await?["data"]["createKnownCart"].clone();
    assert_json_include!(
        actual: &data,
        expected: json!({
            "id": ids.cart,
            "currency": "GBP",
            "cartType": "KNOWN",
            "items": []
        })
    );

    let customer = Customer::find_by_id(ids.customer.unwrap(), &app.db_pool)
        .await
        .expect("failed to query customer from the database")
        .expect("failed to find customer");

    assert_eq!(
        customer.cart_id.expect("customer should have cart id"),
        Uuid::parse_str(&data["id"].as_str().expect("Cart should always have an ID"))
            .expect("cart id should be valid UUID")
    );
    Ok(())
}

use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub struct IdHolder {
    customer: Option<Uuid>,
    cart: Option<Uuid>,
}

pub async fn spawn_app() -> TestApp {
    lazy_static::initialize(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = bazaar::get_configuration().expect("failed to read configuration");

    let database_name = Uuid::new_v4().to_string();
    configuration.set_database_name(database_name);

    let pool = configure_database(&configuration.database).await;

    let server = bazaar::build_app(listener, pool.clone()).expect("failed to bind address");

    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("failed to connect to database");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("failed to create database");

    let pool = PgPool::connect_with(config.with_db())
        .await
        .expect("failed to connect to database");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run database migrations");
    pool
}

pub async fn insert_default_customer(pool: &PgPool) -> Result<IdHolder> {
    let email = format!("{}@test.com", Uuid::nil());
    let first_name = Uuid::nil().to_string();
    let last_name = Uuid::nil().to_string();

    let customer = Customer::new(email, first_name, last_name, pool).await;
    dbg!(&customer);
    let customer = customer.expect("failed to insert default customer");
    Ok(IdHolder {
        customer: Some(customer.id),
        cart: None,
    })
}

pub async fn insert_default_customer_with_cart(pool: &PgPool) -> Result<IdHolder> {
    let email = format!("{}@test.com", Uuid::nil());
    let first_name = Uuid::nil().to_string();
    let last_name = Uuid::nil().to_string();

    let customer = Customer::new(email, first_name, last_name, pool)
        .await
        .expect("failed to insert default customer");
    dbg!(&customer);
    let cart = Customer::add_new_cart(customer.id, Currency::GBP, pool)
        .await
        .expect("failed to attach cart to default customer");
    dbg!(&cart);
    Ok(IdHolder {
        customer: Some(customer.id),
        cart: Some(cart.id),
    })
}
