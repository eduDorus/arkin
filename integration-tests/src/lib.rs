pub mod feature_validator;

use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_persistence::{Persistence, PersistenceConfig};
use time::UtcDateTime;

pub use feature_validator::FeatureValidator;

/// Creates a Persistence instance for integration tests by loading configuration
/// from the standard config files.
///
/// # Configuration
/// Reads from config files based on RUN_MODE and CONFIG_DIR environment variables:
/// - RUN_MODE: defaults to "test" (e.g., test.yml, test_secrets.yml)
/// - CONFIG_DIR: defaults to "./configs"
///
pub async fn init_test_persistence() -> Arc<dyn PersistenceReader> {
    // Set defaults for test environment
    // SAFETY: This is safe in test environment as we're setting env vars before any other code runs
    unsafe {
        if std::env::var("RUN_MODE").is_err() {
            std::env::set_var("RUN_MODE", "test");
        }
        if std::env::var("CONFIG_DIR").is_err() {
            std::env::set_var("CONFIG_DIR", "./configs");
        }
    }

    // Load configuration using arkin-core's config loader
    let config: PersistenceConfig = load();

    // Create test instance metadata
    let now: UtcDateTime = time::OffsetDateTime::now_utc().into();
    let instance = Instance::builder()
        .id(uuid::Uuid::new_v4())
        .name("integration-test".to_string())
        .instance_type(InstanceType::Test)
        .created(now)
        .updated(now)
        .build();

    // Initialize persistence in read-only mode for tests
    Persistence::new(
        &config, instance, false, // only_normalized
        false, // only_predictions
        true,  // dry_run (prevents any writes)
    )
}
