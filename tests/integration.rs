#![feature(try_trait)]
use anyhow::Result;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

use bazaar::{
    configuration::DatabaseSettings,
    models::{shopping_cart::CartType, Currency, Customer, ShoppingCart},
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
id,
cartType,
items,
priceBeforeDiscounts,
discounts,
priceAfterDiscounts,
currency
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

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CreateCustomerResponse {
        create_customer: Customer,
    }

    let response = response.json::<Response<CreateCustomerResponse>>().await?;

    let customer = response
        .data
        .expect("successful response should contain data")
        .create_customer;

    assert_eq!(customer.email, email);
    assert_eq!(customer.first_name, first_name.to_string());
    assert_eq!(customer.last_name, last_name.to_string());
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

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CustomerQueryResponse {
        customer_by_id: Option<Customer>,
    }

    let response = response.json::<Response<CustomerQueryResponse>>().await?;
    dbg!(&response);

    let customer = response
        .data
        .expect("successful response should contain data")
        .customer_by_id;
    assert!(customer.is_some());
    let customer = customer.unwrap();

    assert_eq!(customer.email, format!("{}@test.com", Uuid::nil()));
    assert_eq!(customer.first_name, Uuid::nil().to_string());
    assert_eq!(customer.last_name, Uuid::nil().to_string());
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

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CustomerQueryResponse {
        customer_by_email: Option<Customer>,
    }

    let response = response.json::<Response<CustomerQueryResponse>>().await?;
    dbg!(&response);

    let customer = response
        .data
        .expect("successful response should contain data")
        .customer_by_email;

    assert!(customer.is_some());
    let customer = customer.unwrap();

    assert_eq!(customer.email, format!("{}@test.com", Uuid::nil()));
    assert_eq!(customer.first_name, Uuid::nil().to_string());
    assert_eq!(customer.last_name, Uuid::nil().to_string());
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

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CustomerUpdateResponse {
        update_customer: Customer,
    }

    let response = response.json::<Response<CustomerUpdateResponse>>().await?;
    dbg!(&response);

    let customer = response
        .data
        .expect("successful response should contain data")
        .update_customer;

    assert_eq!(customer.email, format!("{}@test.com", "updated"));
    assert_eq!(customer.first_name, "updated".to_string());
    assert_eq!(customer.last_name, "updated".to_string());
    assert!(customer.last_modified > customer.created_at);
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

    dbg!(&response);

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AnonymousCartResponse {
        create_anonymous_cart: Option<ShoppingCart>,
    }

    let response = response.json::<Response<AnonymousCartResponse>>().await?;
    dbg!(&response);

    let cart = response
        .data
        .expect("successful response should contain data")
        .create_anonymous_cart;
    assert!(cart.is_some());
    let cart = cart.unwrap();

    // By Rusts strong type system, all the other necessary fields by default must be
    // present
    assert_eq!(cart.currency, Currency::GBP);
    assert_eq!(cart.cart_type, CartType::Anonymous);
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

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct KnownCartResponse {
        create_known_cart: Option<ShoppingCart>,
    }

    let response = response.json::<Response<KnownCartResponse>>().await?;
    dbg!(&response);

    let cart = response
        .data
        .expect("successful response should contain data")
        .create_known_cart;
    assert!(cart.is_some());
    let cart = cart.unwrap();

    // By Rusts strong type system, all the other necessary fields by default must be
    // present
    assert_eq!(cart.currency, Currency::GBP);
    assert_eq!(cart.cart_type, CartType::Known);

    let customer = Customer::find_by_id(customer, &app.db_pool)
        .await
        .expect("failed to query customer from the database")
        .expect("failed to find customer");
    assert_eq!(
        customer.cart_id.expect("customer should have cart id"),
        cart.id
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

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct KnownCartResponse {
        create_known_cart: Option<ShoppingCart>,
    }

    let response = response.json::<Response<KnownCartResponse>>().await?;
    dbg!(&response);

    let cart = response
        .data
        .expect("successful response should contain data")
        .create_known_cart;
    assert!(cart.is_some());
    let cart = cart.unwrap();

    assert_eq!(cart.id, ids.cart.unwrap());
    assert_eq!(cart.currency, Currency::GBP);
    assert_eq!(cart.cart_type, CartType::Known);

    let customer = Customer::find_by_id(ids.customer.unwrap(), &app.db_pool)
        .await
        .expect("failed to query customer from the database")
        .expect("failed to find customer");

    assert_eq!(
        customer.cart_id.expect("customer should have cart id"),
        cart.id
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

// Taken from https://github.com/graphql-rust/graphql-client

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Response<Data> {
    /// The absent, partial or complete response data.
    pub data: Option<Data>,
    /// The top-level errors returned by the server.
    pub errors: Option<Vec<Error>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Error {
    /// The human-readable error message. This is the only required field.
    pub message: String,
    /// Which locations in the query the error applies to.
    pub locations: Option<Vec<Location>>,
    /// Which path in the query the error applies to, e.g. `["users", 0, "email"]`.
    pub path: Option<Vec<PathFragment>>,
    /// Additional errors. Their exact format is defined by the server.
    pub extensions: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Location {
    /// The line number in the query string where the error originated (starting from 1).
    pub line: i32,
    /// The column number in the query string where the error originated (starting from 1).
    pub column: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PathFragment {
    /// A key inside an object
    Key(String),
    /// An index inside an array
    Index(i32),
}

impl fmt::Display for PathFragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PathFragment::Key(ref key) => write!(f, "{}", key),
            PathFragment::Index(ref idx) => write!(f, "{}", idx),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use `/` as a separator like JSON Pointer.
        let path = self
            .path
            .as_ref()
            .map(|fragments| {
                fragments
                    .iter()
                    .fold(String::new(), |mut acc, item| {
                        acc.push_str(&format!("{}/", item));
                        acc
                    })
                    .trim_end_matches('/')
                    .to_string()
            })
            .unwrap_or_else(|| "<query>".to_string());

        // Get the location of the error. We'll use just the first location for this.
        let loc = self
            .locations
            .as_ref()
            .and_then(|locations| locations.iter().next())
            .cloned()
            .unwrap_or_else(Location::default);

        write!(f, "{}:{}:{}: {}", path, loc.line, loc.column, self.message)
    }
}
