use std::fmt::Debug;

use async_trait::async_trait;
use tokio::io::{stderr, stdout, AsyncWrite, AsyncWriteExt};

use crate::Level;

use super::Backend;

trait AsyncWriter: AsyncWrite + Send + Sync + Debug + Unpin + 'static {}

impl<T> AsyncWriter for T where T: AsyncWrite + Send + Sync + Debug + Unpin + 'static {}

#[derive(Debug)]
pub struct Os {
    err: Box<dyn AsyncWriter>,
    default: Box<dyn AsyncWriter>,
}

impl Os {
    #[must_use]
    pub fn std() -> Self {
        Self {
            err: Box::new(stderr()),
            default: Box::new(stdout()),
        }
    }
}

#[async_trait]
impl Backend for Os {
    async fn process_log(&mut self, log: &crate::Log) -> anyhow::Result<()> {
        let pipe = if log.level >= Level::Warning {
            &mut self.err
        } else {
            &mut self.default
        };

        let message = format_args!(
            "{} [{}] [{}]: {}\n",
            fixed_width_level(log.level),
            log.timestamp.to_rfc3339(),
            log.process,
            log.message.to_string(),
        )
        .to_string();

        pipe.write_all(message.as_bytes()).await?;
        pipe.flush().await?;

        Ok(())
    }
}

const fn fixed_width_level(level: Level) -> &'static str {
    match level {
        Level::Trace => "TRACE",
        Level::Debug => "DEBUG",
        Level::Info => "INFO ",
        Level::Warning => "WARN ",
        Level::Error => "ERROR",
    }
}
