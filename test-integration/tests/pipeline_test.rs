use test_log::test;
use tracing::info;

/// We want to test the pipeline that we have created. The pipeline is a simple
#[test(tokio::test)]
async fn test_pipeline() {
    info!("Hello World!")
}
