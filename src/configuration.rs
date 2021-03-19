use std::{borrow::Cow, sync::Arc};

use flume::Sender;
use futures::Future;
use once_cell::sync::OnceCell;

use crate::Log;

pub static GLOBAL_CONFIG: OnceCell<Arc<Configuration>> = OnceCell::new();

tokio::task_local! {
    pub static TASK_CONFIG: Option<Arc<Configuration>>;
}

#[derive(Debug)]
pub struct Configuration {
    pub sender: Sender<Arc<Log>>,
    pub process: Cow<'static, str>,
}

impl Configuration {
    pub fn named<S: Into<Cow<'static, str>>>(process_name: S, sender: Sender<Arc<Log>>) -> Self {
        Self {
            sender,
            process: process_name.into(),
        }
    }

    pub fn set_global(config: Self) {
        GLOBAL_CONFIG.set(Arc::new(config)).unwrap();
    }

    pub async fn run<F: Future<Output = R> + Send, R: Send>(self, future: F) -> R {
        TASK_CONFIG.scope(Some(Arc::new(self)), future).await
    }

    pub(crate) fn current() -> Option<Arc<Self>> {
        TASK_CONFIG
            .try_with(Clone::clone)
            .unwrap_or_else(|_| GLOBAL_CONFIG.get().cloned())
    }
}
