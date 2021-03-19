# SirLog

[![Live Build Status](https://img.shields.io/github/workflow/status/khonsulabs/sirlog/Tests/main)](https://github.com/khonsulabs/sirlog/actions?query=workflow:Tests) [![codecov](https://codecov.io/gh/khonsulabs/sirlog/branch/main/graph/badge.svg)](https://codecov.io/gh/khonsulabs/sirlog)

This project is extremely early in development, and is being developed as part of a custom application stack for [Cosmic Verge](https://github.com/khonsulabs/cosmicverge).

The goals of this logging framework are:

* Keep it simple: With modern async tooling, many logging frameworks are more complicated than they need to be.
* Take advantage of Rust: While the examples are currently using Strings for
  messages, SirLog is designed to allow interoperability with enums for
  messages/processes as well as any `serde`-serializable object for structured
  data.
* Provide a rust-only stack for centralized log collection and archiving. (Not available yet)

More information coming soon.

Licensed under the [MIT License](./LICENSE-MIT) and the [Apache License 2.0](./LICENSE-APACHE).