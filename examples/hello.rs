use std::time::Duration;

use sirlog::{backend, info, Configuration, Manager};

#[tokio::main]
async fn main() {
    Configuration::set_global(Configuration::named(
        "hello",
        Manager::default()
            .with_backend(backend::Os::std())
            .spawn_tokio(),
    ));

    info!("Sir Log says, 'Hello, World!'");

    // sirlog is asynchronous, so without yielding the main task the log message will not appear
    tokio::time::sleep(Duration::from_millis(1)).await;
}
