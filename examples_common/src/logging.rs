//! Logging utilities.

use std::collections::HashMap;
use std::io;
use std::iter::successors;
use std::sync::Arc;

use parking_lot::Once;
use tracing::{Level, Metadata};
use tracing_subscriber::{filter, Layer, Registry};
use tracing_subscriber::layer::SubscriberExt;

const DEFAULT_LEVEL: Level = Level::WARN;
const DEFAULT_CRATE_LEVEL: Level = Level::DEBUG;

static LOGGING_INIT: Once = Once::new();

/// Emit logs at all levels to test logging.
#[macro_export]
macro_rules! log_level_test {
    () => {{
        tracing::trace!("TRACE");
        tracing::debug!("DEBUG");
        tracing::info!("INFO");
        tracing::warn!("WARN");
        tracing::error!("ERROR");
    }};
}

pub use log_level_test;

/// Initialize logging idempotently.
///
/// Calling this more than once will have no effect.
pub fn init_logging(filter: LogLevelFilter) {
    let filter = Arc::new(filter);

    LOGGING_INIT.call_once(move || init_logging_actual(filter));
}

/// Test logging in this crate by emitting events at all log levels.
#[allow(unused)]
pub fn self_log_test() {
    log_level_test!();
}

fn init_logging_actual(filter: Arc<LogLevelFilter>) {
    tracing::subscriber::set_global_default(
        Registry::default().with(
            tracing_subscriber::fmt::layer()
                .with_writer(io::stderr)
                .pretty()
                .with_filter(filter::filter_fn(move |meta| filter.allow(meta))),
        ),
    )
        .unwrap();
}

pub struct LogLevelFilter {
    global: Level,
    modules: HashMap<String, Level>,
}

impl Default for LogLevelFilter {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl LogLevelFilter {
    pub fn builder() -> LogLevelFilterBuilder {
        let mut b = LogLevelFilterBuilder {
            ..Default::default()
        };

        b.global = Some(DEFAULT_LEVEL);
        b.modules
            .insert(env!("CARGO_CRATE_NAME").to_string(), DEFAULT_CRATE_LEVEL);

        b
    }

    pub fn set_global(&mut self, level: Level) {
        self.global = level;
    }

    pub fn filter<S>(&mut self, logger: S, level: Level)
        where
            S: Into<String>,
    {
        self.modules.insert(logger.into(), level);
    }

    pub fn allow(&self, meta: &Metadata) -> bool {
        // NOTE on levels: trace has the _lowest_ possible value in sorting (i.e. 0), while error
        //      has the *highest* possible value in sorting (i.e. 4). thus, in order for us to
        //      determine whether a given level is allowed, we must check `log.level` is greater
        //      than or equal to `log_rule.level`.
        if let Some(module) = meta.module_path() {
            let level = successors(Some(module), |m| {
                m.rsplit_once("::").map(|(head, _tail)| head)
            })
                .find_map(|m| self.modules.get(m))
                .unwrap_or(&self.global);

            return meta.level() <= level;
        }

        true
    }
}

#[derive(Default)]
pub struct LogLevelFilterBuilder {
    global: Option<Level>,
    #[allow(unused)]
    modules: HashMap<String, Level>,
}

impl LogLevelFilterBuilder {
    #[allow(unused)]
    pub fn global(mut self, level: Level) -> Self {
        self.global = level.into();
        self
    }

    pub fn level<S>(mut self, logger: S, level: Level) -> Self
        where
            S: Into<String>,
    {
        self.modules.insert(logger.into(), level);
        self
    }

    pub fn build(self) -> LogLevelFilter {
        LogLevelFilter {
            global: self.global.unwrap_or(DEFAULT_LEVEL),
            modules: self.modules,
        }
    }
}
