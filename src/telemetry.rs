use opentelemetry::trace::Tracer;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_opentelemetry::PreSampledTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Subscriber for the application
///
/// It's filtered based off the standard environment filter
///
/// This version includes
/// - JsonStorageLayer
/// - BunyanFormattingLayer
pub fn generate_subscriber(
    name: &str,
    env_filter: String,
    tracer: impl Tracer + PreSampledTracer + Send + Sync,
) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name.to_string(), std::io::stdout);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
}

/// Initialises the subscriber globally and also sets it to include log statements
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("failed to attach logs to tracing");
    set_global_default(subscriber).expect("failed to set global subscriber");
}
