pub mod holder;
pub mod issuer;
pub mod sample;
pub mod store;
pub mod verifier;

use std::sync::Once;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

// initalise tracing once for all tests
static INIT: Once = Once::new();

/// Initialise tracing for tests.
///
/// # Panics
///
/// Panics if the tracing subscriber cannot be set.
pub fn init_tracer() {
    INIT.call_once(|| {
        let subscriber = FmtSubscriber::builder().with_max_level(Level::ERROR).finish();
        tracing::subscriber::set_global_default(subscriber).expect("subscriber set");
    });
}
