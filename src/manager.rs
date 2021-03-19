use std::sync::Arc;

use flume::{Receiver, Sender};
use futures::{future::BoxFuture, FutureExt};

use crate::{backend::Backend, Log};

/// A manager of log messages that runs asynchronously and forwards received log
/// messages onto one or more backends
#[derive(Default, Debug)]
pub struct Manager {
    backends: Vec<Box<dyn Backend>>,
}

impl Manager {
    /// Attach a backend
    pub fn with_backend<B: Backend + 'static>(mut self, backend: B) -> Self {
        self.backends.push(Box::new(backend));
        self
    }

    /// If you are using a custom async executor, this function allows you to
    /// pass in a closure that is responsible for spawning the future into your
    /// async executor.
    ///
    /// # Returns
    ///
    /// The destination for the Manager. This is passed in during creation of a `Configuration`
    #[must_use]
    pub fn launch<F: FnOnce(BoxFuture<'static, ()>)>(self, spawner: F) -> Sender<Arc<Log>> {
        let (sender, receiver) = flume::unbounded();

        spawner(self.run(receiver).boxed());

        sender
    }

    /// Spawns this manager within the global tokio runtime.
    ///
    /// # Returns
    ///
    /// The destination for the Manager. This is passed in during creation of a `Configuration`
    #[must_use]
    pub fn spawn_tokio(self) -> Sender<Arc<Log>> {
        self.launch(|task| {
            tokio::spawn(task);
        })
    }

    async fn run(mut self, receiver: Receiver<Arc<Log>>) {
        while let Ok(log) = receiver.recv_async().await {
            futures::future::join_all(
                self.backends
                    .iter_mut()
                    .map(|backend| backend.process_log(&log)),
            )
            .await
            .into_iter()
            .collect::<Result<Vec<_>, anyhow::Error>>()
            .expect("Error communicating with logging backends");
        }
    }
}
