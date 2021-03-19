use std::{fmt::Display, sync::Arc};

use crate::Configuration;
use chrono::{DateTime, Utc};
use serde::{de::Error, Deserialize, Serialize};

/// The severity of a log message
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub enum Level {
    /// The lowest severity, usually used for very low-level debugging information
    Trace,
    /// Above Trace but below Info in severity, usually used for debugging information only relevant while tracking down an issue
    Debug,
    /// Above Debug but below Warning in severity, usually used for information that may be helpful in identifying important events that are happening
    Info,
    /// Above Info but below Error in severity, usually used for information that generally indicates something might be wrong, but the process is going to continue on
    Warning,
    /// The highest severity level, usually reserved for situations that should almost always be looked at
    Error,
}

/// A log entry
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Log {
    /// The severity of the entry
    pub level: Level,
    /// The process that generated this entry
    pub process: String,
    /// The log message
    pub message: String,
    /// The timestamp when the entry was created
    pub timestamp: DateTime<Utc>,
    /// Structured information that is relevant to the log message
    pub payload: serde_json::Value,
}

impl Log {
    /// Create a new log entry with the `message` and `level` provided
    ///
    /// # Panics
    ///
    /// This must be called when either a global `Configuration` is set or from
    /// within an async task that is executed within `Configuration::run()`
    #[allow(clippy::clippy::needless_pass_by_value)] // This is a choice to make these APIs read cleaner, as Categories are always expected to be an enum constant.
    pub fn new<M: Display>(level: Level, message: M) -> Self {
        let process = Configuration::current()
            .expect("no task or global configuration found")
            .process
            .clone();
        Self {
            level,
            process,
            message: message.to_string(),
            timestamp: Utc::now(),
            payload: serde_json::Value::Null,
        }
    }

    /// Create a new log entry with `Level::Error` and the `message` provided
    ///
    /// # Panics
    ///
    /// This must be called when either a global `Configuration` is set or from
    /// within an async task that is executed within `Configuration::run()`
    pub fn error<M: Display>(message: M) -> Self {
        Self::new(Level::Error, message)
    }

    /// Create a new log entry with `Level::Warning` and the `message` provided
    ///
    /// # Panics
    ///
    /// This must be called when either a global `Configuration` is set or from
    /// within an async task that is executed within `Configuration::run()`
    pub fn warning<M: Display>(message: M) -> Self {
        Self::new(Level::Warning, message)
    }

    /// Create a new log entry with `Level::Info` and the `message` provided
    ///
    /// # Panics
    ///
    /// This must be called when either a global `Configuration` is set or from
    /// within an async task that is executed within `Configuration::run()`
    pub fn info<M: Display>(message: M) -> Self {
        Self::new(Level::Info, message)
    }

    /// Create a new log entry with `Level::Debug` and the `message` provided
    ///
    /// # Panics
    ///
    /// This must be called when either a global `Configuration` is set or from
    /// within an async task that is executed within `Configuration::run()`
    pub fn debug<M: Display>(message: M) -> Self {
        Self::new(Level::Debug, message)
    }

    /// Create a new log entry with `Level::Trace` and the `message` provided
    ///
    /// # Panics
    ///
    /// This must be called when either a global `Configuration` is set or from
    /// within an async task that is executed within `Configuration::run()`
    pub fn trace<M: Display>(message: M) -> Self {
        Self::new(Level::Trace, message)
    }

    /// Add extra information to this log entry, useful for attaching
    /// information that will help understand the entry or the context in which
    /// it was created
    pub fn add<K: Into<String>, V: Serialize>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<&mut Self, serde_json::Error> {
        if matches!(self.payload, serde_json::Value::Null) {
            self.payload = serde_json::Value::Object(serde_json::Map::new());
        }
        let key = key.into();
        if self
            .payload
            .as_object_mut()
            .unwrap()
            .insert(key, serde_json::value::to_value(value)?)
            .is_some()
        {
            return Err(serde_json::Error::custom(
                "attempting to add the same key twice",
            ));
        }

        Ok(self)
    }

    /// The equivalent of `add()`, but exposed in a builder-style pattern
    pub fn with<K: Into<String>, V: Serialize>(
        mut self,
        key: K,
        value: V,
    ) -> Result<Self, serde_json::Error> {
        self.add(key, value)?;

        Ok(self)
    }

    /// Submits this log entry to the current manager
    ///
    /// # Panics
    ///
    /// * If no `Configuration` is available
    /// * If the manager is not able to receive the log message
    pub fn submit(self) {
        let log = Arc::new(self);
        Configuration::current()
            .expect("no task or global configuration found")
            .destination
            .send(log)
            .expect("error sending log to manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Configuration, Manager};

    fn entries_eq_without_timestamps(a: &Log, b: &Log) -> bool {
        a.level == b.level
            && a.process == b.process
            && a.message == b.message
            && a.payload == b.payload
    }

    #[tokio::test]
    async fn entry_building_tests() -> Result<(), serde_json::Error> {
        Configuration::named(
            "entry_building_tests",
            Manager::default().launch(|t| {
                tokio::spawn(t);
            }),
        )
        .run(async {
            assert!(entries_eq_without_timestamps(
                &Log::debug("A"),
                &Log {
                    level: Level::Debug,
                    process: String::from("entry_building_tests"),
                    message: String::from("A"),
                    payload: serde_json::Value::Null,
                    timestamp: Utc::now(),
                }
            ));

            assert!(entries_eq_without_timestamps(
                &Log::info("A"),
                &Log {
                    level: Level::Info,
                    process: String::from("entry_building_tests"),
                    message: String::from("A"),
                    payload: serde_json::Value::Null,
                    timestamp: Utc::now(),
                }
            ));

            assert!(entries_eq_without_timestamps(
                &Log::warning("B"),
                &Log {
                    level: Level::Warning,
                    process: String::from("entry_building_tests"),
                    message: String::from("B"),
                    payload: serde_json::Value::Null,
                    timestamp: Utc::now(),
                }
            ));

            assert!(entries_eq_without_timestamps(
                Log::error("B").add("key", "value")?,
                &Log {
                    level: Level::Error,
                    process: String::from("entry_building_tests"),
                    message: String::from("B"),
                    payload: serde_json::json!({"key": "value"}),
                    timestamp: Utc::now(),
                }
            ));

            assert!(entries_eq_without_timestamps(
                Log::trace("B").add("key", "value")?.add("key2", "value2")?,
                &Log {
                    level: Level::Trace,
                    process: String::from("entry_building_tests"),
                    message: String::from("B"),
                    payload: serde_json::json!({"key": "value", "key2": "value2"}),
                    timestamp: Utc::now(),
                }
            ));

            assert!(Log::trace("B")
                .add("key", "value")?
                .add("key", "value")
                .is_err());

            Ok(())
        })
        .await
    }
}

// #[cfg(feature = "archiver")]
// impl Into<database::schema::Log> for Log {
//     fn into(self) -> database::schema::Log {
//         database::schema::Log {
//             level: self.level.into(),
//             process: self.process.to_string(),
//             message: self.message,
//             payload: match self.payload {
//                 serde_json::Value::Null => None,
//                 other => Some(other),
//             },
//             timestamp: self.timestamp,
//         }
//     }
// }

// #[cfg(feature = "archiver")]
// impl Into<database::schema::Level> for Level {
//     fn into(self) -> database::schema::Level {
//         match self {
//             Self::Error => database::schema::Level::Error,
//             Self::Warning => database::schema::Level::Warning,
//             Self::Info => database::schema::Level::Info,
//             Self::Debug => database::schema::Level::Debug,
//             Self::Trace => database::schema::Level::Trace,
//         }
//     }
// }
