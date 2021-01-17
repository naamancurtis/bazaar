use opentelemetry::{
    global,
    sdk::{propagation::TraceContextPropagator, trace, Resource},
};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_NAMESPACE,
};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use std::sync::Arc;

use bazaar::{build_app, get_configuration};

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let app_name = concat!(env!("CARGO_PKG_NAME"), "::", env!("CARGO_PKG_VERSION"),);
    let configuration = Arc::new(get_configuration()?);

    // @TODO Work out how to get OTEL metrics working
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(String::from("info")));
    let formatting_layer = BunyanFormattingLayer::new(app_name.to_string(), std::io::stdout);
    LogTracer::init().expect("failed to attach logs to tracing");

    global::set_text_map_propagator(TraceContextPropagator::new());

    let (tracer, _uninstall) = opentelemetry_otlp::new_pipeline()
        .with_endpoint(configuration.get_telemetry_agent_endpoint())
        .with_trace_config(trace::config().with_resource(Resource::new(vec![
            SERVICE_NAME.string(app_name),
            SERVICE_NAMESPACE.string("bazaar"),
            DEPLOYMENT_ENVIRONMENT.string(configuration.env.to_string()),
        ])))
        .install()?;

    let otel_layer = OpenTelemetryLayer::new(tracer);
    let registry = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(otel_layer);
    set_global_default(registry)?;

    let connection = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(configuration.database.with_db())
        .await
        .expect("failed to connect to database");

    let listener = TcpListener::bind(configuration.get_addr())?;

    build_app(listener, connection, configuration)?.await?;
    Ok(())
}
