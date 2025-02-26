use std::future::Future;
use std::time::Duration;
use tracing::warn;

pub async fn retry<F, Fut, T, E>(mut f: F, max_retries: usize) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                warn!("Retrying after error: {}", e);
                retries += 1;
                let delay = Duration::from_secs(2u64.pow(retries as u32));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
