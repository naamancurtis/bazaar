/// This is virtually a straight copy and paste from the ApolloTracing & Tracing Extensions from
/// the core library, just modified slightly
use std::collections::BTreeMap;
use std::ops::Deref;

use chrono::{DateTime, Utc};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, ResolveInfo};
use async_graphql::{
    value, QueryPathNode, Request, ServerError, ServerResult, ValidationResult, Value, Variables,
};
use async_graphql_parser::types::ExecutableDocument;
use tracing::{span, Level, Span};

macro_rules! prefix_context {
    ($context:literal) => {
        concat!("graphql::", $context)
    };
}

const TARGET: &str = "async_graphql::graphql";

/// Tracing extension configuration for each request.
pub struct OpenTelemetryConfig {
    /// Use a span as the parent node of the entire query.
    parent: Option<Span>,
    return_tracing_data_to_client: bool,
}

impl Default for OpenTelemetryConfig {
    fn default() -> Self {
        Self {
            parent: None,
            return_tracing_data_to_client: true,
        }
    }
}

impl OpenTelemetryConfig {
    /// Use the provided span as the parent node of the enire query.
    pub fn parent_span(mut self, span: Span) -> Self {
        self.parent = Some(span);
        self
    }
}

#[derive(Debug)]
struct PendingResolve {
    path: Vec<String>,
    field_name: String,
    parent_type: String,
    return_type: String,
    start_time: DateTime<Utc>,
}

#[derive(Debug)]
struct ResolveStat {
    pending_resolve: PendingResolve,
    end_time: DateTime<Utc>,
    start_offset: i64,
}

impl Deref for ResolveStat {
    type Target = PendingResolve;

    fn deref(&self) -> &Self::Target {
        &self.pending_resolve
    }
}

impl Serialize for ResolveStat {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("path", &self.path)?;
        map.serialize_entry("fieldName", &self.field_name)?;
        map.serialize_entry("parentType", &self.parent_type)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.serialize_entry("startOffset", &self.start_offset)?;
        map.serialize_entry(
            "duration",
            &(self.end_time - self.start_time).num_nanoseconds(),
        )?;
        map.end()
    }
}

pub struct OpenTelemetryExtension;

impl ExtensionFactory for OpenTelemetryExtension {
    fn create(&self) -> Box<dyn Extension> {
        Box::new(OpenTelemetry {
            metrics: Metrics {
                start_time: Utc::now(),
                end_time: Utc::now(),
                resolves: Default::default(),
            },
            traces: Default::default(),
            fields: Default::default(),
            query_name: None,
        })
    }
}

struct OpenTelemetry {
    metrics: Metrics,
    traces: Traces,
    fields: BTreeMap<usize, TelemetryData>,
    query_name: Option<String>,
}

struct Metrics {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    resolves: Vec<ResolveStat>,
}

#[derive(Default)]
struct Traces {
    root: Option<Span>,
    parse: Option<Span>,
    validation: Option<Span>,
    execute: Option<Span>,
}

struct TelemetryData {
    span: Span,
    metrics: PendingResolve,
}

impl TelemetryData {
    pub fn new<'a>(
        span: Span,
        path_node: &'a QueryPathNode<'a>,
        parent_type: String,
        return_type: String,
    ) -> Self {
        Self {
            metrics: PendingResolve {
                path: path_node.to_string_vec(),
                field_name: path_node.field_name().to_string(),
                parent_type,
                return_type,
                start_time: Utc::now(),
            },
            span,
        }
    }
}

#[async_trait::async_trait]
impl Extension for OpenTelemetry {
    fn name(&self) -> Option<&'static str> {
        Some("tracing")
    }

    async fn prepare_request(
        &mut self,
        ctx: &ExtensionContext<'_>,
        request: Request,
    ) -> ServerResult<Request> {
        let parent_span = ctx
            .data_opt::<OpenTelemetryConfig>()
            .and_then(|cfg| cfg.parent.as_ref());

        let root_span = match parent_span {
            Some(parent) => span!(
                target: TARGET,
                parent: parent,
                Level::INFO,
                prefix_context!("request")
            ),
            None => span!(
                target: TARGET,
                parent: None,
                Level::INFO,
                prefix_context!("request")
            ),
        };

        root_span.with_subscriber(|(id, d)| d.enter(id));
        self.traces.root.replace(root_span);
        Ok(request)
    }

    fn parse_start(
        &mut self,
        _ctx: &ExtensionContext<'_>,
        _query_source: &str,
        _variables: &Variables,
    ) {
        if let Some(ref root) = self.traces.root {
            let parse_span = span!(
                target: TARGET,
                parent: root,
                Level::INFO,
                prefix_context!("parse")
            );

            parse_span.with_subscriber(|(id, d)| d.enter(id));
            self.traces.parse.replace(parse_span);
            self.metrics.start_time = Utc::now();
        }
    }

    fn validation_start(&mut self, _ctx: &ExtensionContext<'_>) {
        if let Some(parent) = &self.traces.root {
            let validation_span = span!(
                target: TARGET,
                parent: parent,
                Level::INFO,
                prefix_context!("validation")
            );
            validation_span.with_subscriber(|(id, d)| d.enter(id));
            self.traces.validation.replace(validation_span);
        }
    }

    fn parse_end(&mut self, _ctx: &ExtensionContext<'_>, _document: &ExecutableDocument) {
        self.traces
            .parse
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
    }

    fn validation_end(&mut self, _ctx: &ExtensionContext<'_>, _result: &ValidationResult) {
        self.traces
            .validation
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
    }

    fn execution_start(&mut self, _ctx: &ExtensionContext<'_>) {
        let execute_span = if let Some(parent) = &self.traces.root {
            span!(
                target: TARGET,
                parent: parent,
                Level::INFO,
                prefix_context!("execute")
            )
        } else {
            // For every step of the subscription stream.
            tracing::warn!("SETTING NONE FOR PARENT");
            span!(
                target: TARGET,
                parent: None,
                Level::INFO,
                prefix_context!("execute")
            )
        };

        execute_span.with_subscriber(|(id, d)| d.enter(id));
        self.traces.execute.replace(execute_span);
    }

    fn execution_end(&mut self, ctx: &ExtensionContext<'_>) {
        self.traces
            .root
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
        self.metrics.end_time = Utc::now();

        if let Some(parent_span) = ctx
            .data_opt::<OpenTelemetryConfig>()
            .and_then(|cfg| cfg.parent.as_ref())
        {
            parent_span.with_subscriber(|(id, d)| d.exit(id));
        }
    }

    fn resolve_start(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        let parent_span = match info.resolve_id.parent {
            Some(parent_id) if parent_id > 0 => self
                .fields
                .get(&parent_id)
                .map(|telemetry_data| &telemetry_data.span),
            _ => self.traces.execute.as_ref(),
        };

        if let Some(parent_span) = parent_span {
            if self.query_name.is_none() {
                self.query_name = Some(info.path_node.to_string());
            }

            let span = span!(
                target: TARGET,
                parent: parent_span,
                Level::DEBUG,
                prefix_context!("field_resolver"),
                graphql_field_id = %info.resolve_id.current,
                graphql_path = %info.path_node,
                graphql_parent_type = %info.parent_type,
                graphql_return_type = %info.return_type,
            );

            span.with_subscriber(|(id, d)| d.enter(id));

            let telemetry_data = TelemetryData::new(
                span,
                info.path_node,
                info.parent_type.to_string(),
                info.return_type.to_string(),
            );
            self.fields.insert(info.resolve_id.current, telemetry_data);
        }
    }

    fn resolve_end(&mut self, _ctx: &ExtensionContext<'_>, info: &ResolveInfo<'_>) {
        if let Some(telemetry_data) = self.fields.remove(&info.resolve_id.current) {
            telemetry_data.span.with_subscriber(|(id, d)| d.exit(id));
            let pending_resolve = telemetry_data.metrics;
            let start_offset = (pending_resolve.start_time - self.metrics.start_time)
                .num_nanoseconds()
                .unwrap();
            self.metrics.resolves.push(ResolveStat {
                pending_resolve,
                start_offset,
                end_time: Utc::now(),
            });
        }
    }

    fn error(&mut self, _ctx: &ExtensionContext<'_>, err: &ServerError) {
        let resolved_values = self.metrics.resolves.len();
        let pending_values = self.fields.len();
        let time_to_error_ms = (Utc::now() - self.metrics.start_time).num_milliseconds();
        tracing::error!(target: TARGET, error = %err.message, error.extensions = ?err.extensions, resolved_values, pending_values, %time_to_error_ms);

        for (_, TelemetryData { span, .. }) in self.fields.iter() {
            span.with_subscriber(|(id, d)| d.exit(id));
        }
        self.fields.clear();

        self.traces
            .execute
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
        self.traces
            .validation
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
        self.traces
            .parse
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
        self.traces
            .root
            .take()
            .and_then(|span| span.with_subscriber(|(id, d)| d.exit(id)));
    }

    fn result(&mut self, ctx: &ExtensionContext<'_>) -> Option<Value> {
        if let Some(cfg) = ctx.data_opt::<OpenTelemetryConfig>() {
            if !cfg.return_tracing_data_to_client {
                return None;
            }
        }
        self.metrics
            .resolves
            .sort_by(|a, b| a.start_offset.cmp(&b.start_offset));

        let result = value!({
            "version": 1,
            "startTime": self.metrics.start_time.to_rfc3339(),
            "endTime": self.metrics.end_time.to_rfc3339(),
            "duration": (self.metrics.end_time - self.metrics.start_time).num_nanoseconds(),
            "execution": {
                "resolvers": self.metrics.resolves
            }
        });
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::extensions::TracingConfig;
    use async_graphql::*;
    use tracing::{span, Level};

    struct QueryRoot;

    #[Object]
    impl QueryRoot {
        pub async fn get_jane(&self) -> Query {
            Query {
                id: 100,
                details: SubQuery {
                    name: "Jane".to_owned(),
                },
            }
        }
    }

    #[derive(SimpleObject)]
    struct Query {
        id: i32,
        details: SubQuery,
    }

    #[derive(SimpleObject)]
    struct SubQuery {
        name: String,
    }

    #[tokio::test]
    async fn basic_test() {
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
            .extension(OpenTelemetryExtension)
            .finish();

        let root_span = span!(parent: None, Level::INFO, "span root");
        let query = r#"
                query {
                    getJane {
                        id
                        details {
                            name
                        }
                    }
                }
            "#;

        let request = Request::new(query).data(TracingConfig::default().parent_span(root_span));
        schema.execute(request).await;
    }
}
