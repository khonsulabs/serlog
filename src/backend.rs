use std::fmt::Debug;

use crate::Log;
use async_trait::async_trait;

mod memory;
mod os;

pub use self::{memory::*, os::*};

/// A logging backend
#[async_trait]
pub trait Backend: Debug + Send + Sync {
    /// Process the log message `log`
    async fn process_log(&mut self, log: &Log) -> anyhow::Result<()>;
}
