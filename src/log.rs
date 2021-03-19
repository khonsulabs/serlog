use std::{borrow::Cow, fmt::Display, sync::Arc};

use crate::Configuration;
use chrono::{DateTime, Utc};
use serde::{de::Error, Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Log {
    pub level: Level,
    pub process: Cow<'static, str>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
}

impl Log {
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

    pub fn error<M: Display>(message: M) -> Self {
        Self::new(Level::Error, message)
    }

    pub fn warning<M: Display>(message: M) -> Self {
        Self::new(Level::Warning, message)
    }

    pub fn info<M: Display>(message: M) -> Self {
        Self::new(Level::Info, message)
    }

    pub fn debug<M: Display>(message: M) -> Self {
        Self::new(Level::Debug, message)
    }

    pub fn trace<M: Display>(message: M) -> Self {
        Self::new(Level::Trace, message)
    }

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

    pub fn with<K: Into<String>, V: Serialize>(
        mut self,
        key: K,
        value: V,
    ) -> Result<Self, serde_json::Error> {
        self.add(key, value)?;

        Ok(self)
    }

    pub fn submit(self) {
        let log = Arc::new(self);
        Configuration::current()
            .expect("no task or global configuration found")
            .sender
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
                    process: Cow::from("entry_building_tests"),
                    message: String::from("A"),
                    payload: serde_json::Value::Null,
                    timestamp: Utc::now(),
                }
            ));

            assert!(entries_eq_without_timestamps(
                &Log::info("A"),
                &Log {
                    level: Level::Info,
                    process: Cow::from("entry_building_tests"),
                    message: String::from("A"),
                    payload: serde_json::Value::Null,
                    timestamp: Utc::now(),
                }
            ));

            assert!(entries_eq_without_timestamps(
                &Log::warning("B"),
                &Log {
                    level: Level::Warning,
                    process: Cow::from("entry_building_tests"),
                    message: String::from("B"),
                    payload: serde_json::Value::Null,
                    timestamp: Utc::now(),
                }
            ));

            assert!(entries_eq_without_timestamps(
                Log::error("B").add("key", "value")?,
                &Log {
                    level: Level::Error,
                    process: Cow::from("entry_building_tests"),
                    message: String::from("B"),
                    payload: serde_json::json!({"key": "value"}),
                    timestamp: Utc::now(),
                }
            ));

            assert!(entries_eq_without_timestamps(
                Log::trace("B").add("key", "value")?.add("key2", "value2")?,
                &Log {
                    level: Level::Trace,
                    process: Cow::from("entry_building_tests"),
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
