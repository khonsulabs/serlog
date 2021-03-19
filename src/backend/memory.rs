use std::{collections::VecDeque, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{backend::Backend, Log};

/// A memory-based log message backend. This is for use cases where you want to review the last X log messages.
#[derive(Debug)]
pub struct Memory {
    /// The maximum number of entries to keep in memory
    pub max_entries: usize,
    /// The storage for the backend. Locking this will block log entries from arriving, so you should only acquire the mutex lock for short operations.
    pub entries: Arc<Mutex<VecDeque<Log>>>,
}

impl Memory {
    /// Create a new instance
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: Arc::default(),
        }
    }
}

#[async_trait]
impl Backend for Memory {
    async fn process_log(&mut self, log: &Log) -> anyhow::Result<()> {
        let mut entries = self.entries.lock().await;

        entries.push_front(log.clone());

        while entries.len() > self.max_entries {
            entries.pop_back();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{Configuration, Manager};

    use super::*;

    #[tokio::test]
    async fn send_test() -> anyhow::Result<()> {
        let test_backend = Memory::new(2);
        let entries = test_backend.entries.clone();
        let sender = Manager::default()
            .with_backend(test_backend)
            .launch(|task| {
                tokio::spawn(task);
            });

        Configuration::named("send_test", sender)
            .run(async {
                Log::info("A").submit();
                Log::info("B").submit();
                Log::info("A").submit();
            })
            .await;

        tokio::time::sleep(Duration::from_millis(1)).await;
        {
            let entries = entries.lock().await;
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].message, "A");
            assert_eq!(entries[1].message, "B");
        }

        Ok(())
    }
}
