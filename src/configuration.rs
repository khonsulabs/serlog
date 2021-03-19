use std::{fmt::Display, sync::Arc};

use flume::Sender;
use futures::Future;
use once_cell::sync::OnceCell;

use crate::Log;

/// The global logging configuration
static GLOBAL_CONFIG: OnceCell<Arc<Configuration>> = OnceCell::new();

tokio::task_local! {
    /// The async-task-local logging configuration
    static TASK_CONFIG: Option<Arc<Configuration>>;
}

/// A logging configuration
#[derive(Debug)]
pub struct Configuration {
    /// the destination for log messages to be delivered
    pub destination: Sender<Arc<Log>>,
    /// the name of the process that generates the logs being sent
    pub process: String,
}

impl Configuration {
    /// Create a new configuration using the
    pub fn named<S: Display>(process: S, sender: Sender<Arc<Log>>) -> Self {
        Self {
            destination: sender,
            process: process.to_string(),
        }
    }

    /// Set the global logging configuration. If no other configuration is
    /// found, this one is used.
    ///
    /// # Panics
    ///
    /// Panics if another global logging configuration has already been set
    pub fn set_global(config: Self) {
        GLOBAL_CONFIG.set(Arc::new(config)).unwrap();
    }

    /// Executes a `Future` with the configuration. Log messages from within
    /// code executed by the future will be submitted through this configuration
    pub async fn run<F: Future<Output = R> + Send, R: Send>(self, future: F) -> R {
        TASK_CONFIG.scope(Some(Arc::new(self)), future).await
    }

    pub(crate) fn current() -> Option<Arc<Self>> {
        TASK_CONFIG
            .try_with(Clone::clone)
            .unwrap_or_else(|_| GLOBAL_CONFIG.get().cloned())
    }
}
