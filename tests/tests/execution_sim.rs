use test_log::test;
use tracing::info;

#[test(tokio::test)]
async fn test_pipeline() {
    info!("Hello World!")
}
