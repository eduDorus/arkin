use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use tokio_rustls::rustls::crypto::{aws_lc_rs, CryptoProvider};
use tracing::{error, info};

use arkin_binance::{prelude::*, AggTradeStream, PartialDepthStream};
use arkin_core::prelude::*;
use trade::CancelOpenOrdersRequest;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    init_tracing();
    info!("Starting Arkin Binance 🚀");

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let (mut conn, _) = BinanceWebSocketClient::connect_default().await?;
    let agg_trade_stream: Vec<Stream> = vec![
        AggTradeStream::new("btcusdt").into(),
        // AggTradeStream::new("ethusdt").into(),
        // DiffDepthStream::from_100ms("btcusdt").into(),
        // BookTickerStream::from_symbol("btcusdt").into(),
        PartialDepthStream::from_100ms("btcusdt", 20).into(),
    ];
    conn.subscribe(agg_trade_stream.iter()).await;

    // Start a timer for 10 seconds
    let timer = tokio::time::Instant::now();
    let duration = Duration::new(10, 0);
    // Read messages
    while let Some(message) = conn.as_mut().next().await {
        if timer.elapsed() >= duration {
            info!("10 seconds elapsed, exiting loop.");
            break; // Exit the loop after 10 seconds
        }
        match message {
            Ok(message) => {
                let data = message.into_data();
                let string_data = String::from_utf8(data).expect("Found invalid UTF-8 chars");
                info!("{}", &string_data);
            }
            Err(_) => break,
        }
    }
    // Disconnect
    conn.close().await.expect("Failed to disconnect");

    let api_key = "ppCYOYKlKLRVwGCzmcbXNf2Qn34aeDEN36A4I0Fwdj8WmpvfkxO9cmNIx5PwhmOd";
    let api_secret = "cs4wa0w860lgkViblUzua4ThRXpfD22ruG8d0GytU7fIrJCvz8jvCAzKpaKPwTl0";
    let url = "https://fapi.binance.com";
    let credentials = Credentials::from_hmac(api_key, api_secret);
    let client = BinanceHttpClient::with_url(url).credentials(credentials);

    // Send order
    // let req: Request = NewOrder::new("SOLUSDT", Side::Buy, "LIMIT")
    //     .time_in_force(TimeInForce::Gtc)
    //     .quantity(Decimal::new(1, 0))
    //     .price(Decimal::new(2150000, 4))
    //     .into();
    // let res = client.send(req).await;
    // match res {
    //     Ok(res) => {
    //         info!("Response: {:?}", res);
    //     }
    //     Err(e) => {
    //         error!("Error: {:?}", e);
    //     }
    // }
    // sleep for 30 seconds
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Cancel order
    let req: Request = CancelOpenOrdersRequest::new("SOLUSDT").into();
    let res = client.send(req).await;
    match res {
        Ok(res) => {
            info!("Response: {:?}", res);
        }
        Err(e) => {
            error!("Error: {:?}", e);
        }
    }
    Ok(())
}
