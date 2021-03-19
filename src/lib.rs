//! a structured logging framework built with async-io in mind.

#![forbid(unsafe_code)]
#![warn(
    clippy::cargo,
    // clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    future_incompatible,
    rust_2018_idioms,
    missing_docs
)]
#![cfg_attr(doc, warn(rustdoc))]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::items_after_statements,
    clippy::missing_errors_doc,
    clippy::multiple_crate_versions,
    // clippy::missing_panics_doc, // not on stable yet
    clippy::option_if_let_else,
)]

/// logging backends (destinations)
pub mod backend;
mod configuration;
mod log;
mod manager;

pub use self::{configuration::*, log::*, manager::*};

mod macros;
