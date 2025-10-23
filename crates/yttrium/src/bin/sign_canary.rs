use {
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
        .finish()
        .with(probe_layer.clone())
        .try_init()
        .unwrap();

    // let start = Instant::now();
    let result = test_sign_impl().await;
    tracing::debug!(probe = "e2e");
    // let e2e_latency = start.elapsed();

    if std::env::var("ENABLE_RECORD_CLOUDWATCH_METRICS")
        == Ok("true".to_string())
    {
        let config = aws_config::load_from_env().await;
        let cloudwatch_client = aws_sdk_cloudwatch::Client::new(&config);
        let region = config.region().unwrap().to_string();
        let region_dimension = Dimension::builder()
            .name("Region".to_string())
            .value(region.clone())
            .build();

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
                let probe_dimension = Dimension::builder()
                    .name("Probe".to_string())
                    .value(p.probe.clone())
                    .build();
                let group_dimension = p.group.clone().map(|g| {
                    Dimension::builder()
                        .name("Group".to_string())
                        .value(g)
                        .build()
                });
                let relay_url_dimension = Dimension::builder()
                    .name("RelayUrl".to_string())
                    .value(get_relay_url())
                    .build();
                vec![
                    {
                        let mut metric = MetricDatum::builder()
                            .metric_name("probe_hit")
                            // .metric_name(format!("probe_{}_hit", p.probe.clone()))
                            .dimensions(probe_dimension.clone());
                        if let Some(group_dimension) = group_dimension.clone() {
                            metric = metric.dimensions(group_dimension);
                        }
                        metric
                            .dimensions(region_dimension.clone())
                            .dimensions(relay_url_dimension.clone())
                            .value(1.)
                            .unit(StandardUnit::Count)
                            .timestamp(now)
                            .build()
                    },
                    {
                        let mut metric = MetricDatum::builder()
                            .metric_name("probe_latency_seconds")
                            .dimensions(probe_dimension.clone());
                        if let Some(group_dimension) = group_dimension.clone() {
                            metric = metric.dimensions(group_dimension);
                        }
                        metric
                            .dimensions(region_dimension.clone())
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
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = ProbeVisitor::default();
        event.record(&mut visitor);

        if let Some(probe_name) = visitor.probe {
            let elapsed_s = self.start.elapsed().as_secs_f64();

            if let Ok(mut buf) = self.accumulator.lock() {
                buf.push(ProbePoint {
                    // probe: format!(
                    //     "probe_{}_latency_seconds",
                    //     sanitize_metric(&probe_name)
                    // ),
                    probe: probe_name,
                    group: visitor.group,
                    elapsed_s,
                });
                // buf.push(ProbePoint {
                //     probe: format!(
                //         "probe_{}_hit",
                //         sanitize_metric(&probe_name)
                //     ),
                //     elapsed_s: 1.,
                // });
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
