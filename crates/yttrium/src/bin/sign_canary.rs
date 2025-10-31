use {
    aws_config::Region,
    aws_sdk_cloudwatch::{
        primitives::DateTime,
        types::{Dimension, MetricDatum, StandardUnit},
    },
    std::{
        sync::{Arc, Mutex},
        time::{Instant, SystemTime},
    },
    tracing::Event,
    tracing_subscriber::{
        layer::{Context, SubscriberExt},
        registry::LookupSpan,
        util::SubscriberInitExt,
        Layer,
    },
    yttrium::sign::{client::get_relay_url, test_helpers::test_sign_impl},
};

#[tokio::main]
pub async fn main() {
    let probe_layer = ProbeLayer::new();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .finish()
        .with(probe_layer.clone())
        .try_init()
        .unwrap();

    let result = test_sign_impl().await;
    if result.is_ok() {
        tracing::debug!(probe = "e2e");
    } else {
        tracing::debug!(probe = "e2e_fail");
    }

    if std::env::var("ENABLE_RECORD_CLOUDWATCH_METRICS")
        == Ok("true".to_string())
    {
        // Load the config twice
        let region = {
            // Once to get the region
            let config = aws_config::load_from_env().await;
            config.region().unwrap().to_string()
        };

        // Second time to actually create the actual client we'll use
        const RECORD_REGION: &str = "eu-central-1";
        let config = aws_config::from_env()
            .region(Region::new(RECORD_REGION))
            .load()
            .await;
        let cloudwatch_client = aws_sdk_cloudwatch::Client::new(&config);
        let region_dimension = Dimension::builder()
            .name("Region".to_string())
            .value(region.clone())
            .build();
        let relay_url_dimension = Dimension::builder()
            .name("RelayUrl".to_string())
            .value(get_relay_url())
            .build();
        let dimensions = vec![region_dimension, relay_url_dimension];

        // TODO measure crypto operation latency
        // TODO measure storage operation latency
        // TODO measure client request latency

        println!("probe_layer: {:?}", probe_layer.accumulator());
        // panic!();

        let now = DateTime::from(SystemTime::now());
        let metrics = probe_layer
            .accumulator()
            .iter()
            .flat_map(|p| {
                let mut dimensions = dimensions.clone();
                dimensions.push(
                    Dimension::builder()
                        .name("Probe".to_string())
                        .value(p.probe.clone())
                        .build(),
                );
                if let Some(group) = p.group.clone() {
                    dimensions.push(
                        Dimension::builder()
                            .name("Group".to_string())
                            .value(group)
                            .build(),
                    );
                }
                vec![
                    {
                        MetricDatum::builder()
                            .metric_name("probe_hit")
                            .set_dimensions(Some(dimensions.clone()))
                            .value(1.)
                            .unit(StandardUnit::Count)
                            .timestamp(now)
                            .build()
                    },
                    {
                        MetricDatum::builder()
                            .metric_name("probe_latency_seconds")
                            .set_dimensions(Some(dimensions.clone()))
                            .value(p.elapsed_s)
                            .unit(StandardUnit::Seconds)
                            .timestamp(now)
                            .build()
                    },
                ]
            })
            .collect::<Vec<_>>();

        cloudwatch_client
            .put_metric_data()
            .namespace("RustSignClientCanary")
            .set_metric_data(Some(metrics))
            // .metric_data(
            //     MetricDatum::builder()
            //         .metric_name("e2e.success".to_string())
            //         .dimensions(dimensions.clone())
            //         .value(if result.is_ok() { 1. } else { 0. })
            //         .unit(StandardUnit::Count)
            //         .timestamp(DateTime::from(SystemTime::now()))
            //         .build(),
            // )
            // .metric_data(
            //     MetricDatum::builder()
            //         .metric_name("e2e.latency".to_string())
            //         .dimensions(dimensions.clone())
            //         .value(e2e_latency.as_millis() as f64)
            //         .unit(StandardUnit::Milliseconds)
            //         .timestamp(DateTime::from(SystemTime::now()))
            //         .build(),
            // )
            .send()
            .await
            .unwrap();
    }

    if let Err(e) = result {
        panic!("Test failed: {e}");
    }
}

const PROBE_KEY: &str = "probe";
const GROUP_KEY: &str = "group";

#[derive(Clone, Debug)]
pub struct ProbePoint {
    pub probe: String,
    pub group: Option<String>,
    pub elapsed_s: f64,
}

// Extension type to store group in span extensions
#[derive(Clone, Debug)]
struct GroupExtension(String);

#[derive(Clone)]
pub struct ProbeLayer {
    start: Instant,
    accumulator: Arc<Mutex<Vec<ProbePoint>>>,
}

impl ProbeLayer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            accumulator: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn accumulator(&self) -> Vec<ProbePoint> {
        self.accumulator.lock().unwrap().clone()
    }
}

impl<S> Layer<S> for ProbeLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut visitor = ProbeVisitor::default();
        event.record(&mut visitor);

        if let Some(probe_name) = visitor.probe {
            let elapsed_s = self.start.elapsed().as_secs_f64();

            // If group wasn't set on the event, walk up the span hierarchy to find it
            let group = visitor.group.or_else(|| {
                let mut current_span = ctx.event_span(event);
                while let Some(span) = current_span {
                    if let Some(ext) = span.extensions().get::<GroupExtension>()
                    {
                        return Some(ext.0.clone());
                    }
                    current_span = span.parent();
                }
                None
            });

            if let Ok(mut buf) = self.accumulator.lock() {
                buf.push(ProbePoint { probe: probe_name, group, elapsed_s });
            }
        }
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        let mut visitor = ProbeVisitor::default();
        attrs.record(&mut visitor);

        if let Some(group) = visitor.group {
            if let Some(span) = ctx.span(id) {
                span.extensions_mut().insert(GroupExtension(group));
            } else {
                tracing::trace!(
                    "Failed to lookup span {:?} for group extension insertion",
                    id
                );
            }
        }
    }
}

// Extract the `probe = "<name>"` field from the event
#[derive(Default)]
struct ProbeVisitor {
    probe: Option<String>,
    group: Option<String>,
}

impl tracing::field::Visit for ProbeVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == PROBE_KEY {
            self.probe = Some(value.to_string());
        } else if field.name() == GROUP_KEY {
            self.group = Some(value.to_string());
        }
    }

    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn std::fmt::Debug,
    ) {
        if field.name() == PROBE_KEY {
            self.probe = Some(format!("{value:?}"));
        } else if field.name() == GROUP_KEY {
            self.group = Some(format!("{value:?}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt},
    };

    #[test]
    fn test_probe_with_event_group() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        tracing::debug!(probe = "test_probe", group = "event_group");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].probe, "test_probe");
        assert_eq!(points[0].group, Some("event_group".to_string()));
    }

    #[test]
    fn test_probe_with_span_group() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        let span = tracing::debug_span!("test_span", group = "span_group");
        let _enter = span.enter();
        tracing::debug!(probe = "test_probe");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].probe, "test_probe");
        assert_eq!(points[0].group, Some("span_group".to_string()));
    }

    #[test]
    fn test_probe_with_nested_spans_inner_group() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        let outer = tracing::debug_span!("outer_span");
        let _outer_enter = outer.enter();
        let inner = tracing::debug_span!("inner_span", group = "inner_group");
        let _inner_enter = inner.enter();
        tracing::debug!(probe = "test_probe");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].probe, "test_probe");
        assert_eq!(points[0].group, Some("inner_group".to_string()));
    }

    #[test]
    fn test_probe_with_nested_spans_outer_group() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        let outer = tracing::debug_span!("outer_span", group = "outer_group");
        let _outer_enter = outer.enter();
        let inner = tracing::debug_span!("inner_span");
        let _inner_enter = inner.enter();
        tracing::debug!(probe = "test_probe");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].probe, "test_probe");
        // Should find the group from the outer span
        assert_eq!(points[0].group, Some("outer_group".to_string()));
    }

    #[test]
    fn test_probe_with_nested_spans_both_groups() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        let outer = tracing::debug_span!("outer_span", group = "outer_group");
        let _outer_enter = outer.enter();
        let inner = tracing::debug_span!("inner_span", group = "inner_group");
        let _inner_enter = inner.enter();
        tracing::debug!(probe = "test_probe");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].probe, "test_probe");
        // Inner span group should take precedence
        assert_eq!(points[0].group, Some("inner_group".to_string()));
    }

    #[test]
    fn test_probe_event_group_overrides_span() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        let span = tracing::debug_span!("test_span", group = "span_group");
        let _enter = span.enter();
        tracing::debug!(probe = "test_probe", group = "event_group");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].probe, "test_probe");
        // Event group should override span group
        assert_eq!(points[0].group, Some("event_group".to_string()));
    }

    #[test]
    fn test_probe_without_group() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        tracing::debug!(probe = "test_probe");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].probe, "test_probe");
        assert_eq!(points[0].group, None);
    }

    #[test]
    fn test_multiple_probes_same_span() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        let span = tracing::debug_span!("test_span", group = "shared_group");
        let _enter = span.enter();
        tracing::debug!(probe = "probe1");
        tracing::debug!(probe = "probe2");
        tracing::debug!(probe = "probe3");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 3);
        assert_eq!(points[0].probe, "probe1");
        assert_eq!(points[0].group, Some("shared_group".to_string()));
        assert_eq!(points[1].probe, "probe2");
        assert_eq!(points[1].group, Some("shared_group".to_string()));
        assert_eq!(points[2].probe, "probe3");
        assert_eq!(points[2].group, Some("shared_group".to_string()));
    }

    #[test]
    fn test_deeply_nested_spans() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        let level1 = tracing::debug_span!("level1", group = "top_level");
        let _enter1 = level1.enter();
        let level2 = tracing::debug_span!("level2");
        let _enter2 = level2.enter();
        let level3 = tracing::debug_span!("level3");
        let _enter3 = level3.enter();
        tracing::debug!(probe = "deep_probe");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].probe, "deep_probe");
        // Should find the group from the top-level span
        assert_eq!(points[0].group, Some("top_level".to_string()));
    }

    #[test]
    fn test_non_probe_events_ignored() {
        let probe_layer = ProbeLayer::new();
        let _guard = tracing_subscriber::registry()
            .with(probe_layer.clone())
            .set_default();

        tracing::debug!("regular debug message");
        tracing::info!(group = "some_group", "message with group but no probe");
        tracing::debug!(probe = "actual_probe", group = "probe_group");

        let points = probe_layer.accumulator();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].probe, "actual_probe");
        assert_eq!(points[0].group, Some("probe_group".to_string()));
    }
}
