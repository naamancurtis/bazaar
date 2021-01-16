use opentelemetry::{
    global,
    sdk::{propagation::TraceContextPropagator, trace, Resource},
};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_NAMESPACE,
};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use std::sync::Arc;

use bazaar::{
    build_app, get_configuration,
    telemetry::{generate_subscriber, init_subscriber},
};

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let app_name = concat!(env!("CARGO_PKG_NAME"), "::", env!("CARGO_PKG_VERSION"),);
    let configuration = Arc::new(get_configuration()?);

    global::set_text_map_propagator(TraceContextPropagator::new());
    let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline()
        .with_endpoint(configuration.get_telemetry_agent_endpoint())
        .with_trace_config(trace::config().with_resource(Resource::new(vec![
            SERVICE_NAME.string(app_name),
            SERVICE_NAMESPACE.string("bazaar"),
            DEPLOYMENT_ENVIRONMENT.string(configuration.env.to_string()),
        ])))
        .install()?;
    let subscriber = generate_subscriber(app_name, String::from("info"), tracer);
    init_subscriber(subscriber);

    let connection = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(configuration.database.with_db())
        .await
        .expect("failed to connect to database");

    let listener = TcpListener::bind(configuration.get_addr())?;

    build_app(listener, connection, configuration)?.await?;
    Ok(())
}
