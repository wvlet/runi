use runi_log::{info, warn, debug, error, span, Level};

fn main() {
    runi_log::init();

    info!("Runi started");
    debug!(version = "0.1.0", "loading config");
    warn!(retries = 3, "connection slow");

    let span = span!(Level::INFO, "request", method = "GET", path = "/api");
    let _guard = span.enter();
    info!("handling request");
    error!("something went wrong");
}
