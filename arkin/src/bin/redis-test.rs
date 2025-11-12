use anyhow::Result;
use arkin_core::utils::custom_serde;
use core::str;
use redis::{AsyncTypedCommands, PushInfo, PushKind, Value};
use rmp_serde::{Deserializer, Serializer};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use strum::Display;
use time::OffsetDateTime;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let client = redis::Client::open("redis://192.168.100.100/?protocol=resp3")?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let config = redis::AsyncConnectionConfig::new().set_push_sender(tx);
    let mut sub_con = client.get_multiplexed_async_connection_with_config(&config).await?;
    sub_con
        .subscribe(&[Channel::Channel1.to_string(), Channel::Channel2.to_string()])
        .await?;

    // Spawn receiver task
    let receiver_handle = tokio::spawn(async move {
        loop {
            if let Some(push) = rx.recv().await {
                match push {
                    PushInfo {
                        kind: PushKind::Message,
                        data,
                    } => {
                        // RESP3 format: [channel_name, message_data]
                        if data.len() >= 2 {
                            if let Value::BulkString(bytes) = &data[1] {
                                let mut de = Deserializer::new(&bytes[..]);
                                match Deserialize::deserialize(&mut de) {
                                    Ok(msg) => {
                                        let msg: Message = msg;
                                        let now_nanos =
                                            OffsetDateTime::now_utc().time() - OffsetDateTime::UNIX_EPOCH.time();
                                        let latency = now_nanos - msg.timestamp();
                                        let latency_us = latency.whole_microseconds();
                                        match msg {
                                            Message::Tick {
                                                instrument,
                                                bid,
                                                ask,
                                                ..
                                            } => {
                                                println!(
                                                    "Received Tick: instrument={}, bid={}, ask={}, latency={}µs",
                                                    instrument, bid, ask, latency_us,
                                                );
                                            }
                                            Message::Trade {
                                                instrument,
                                                price,
                                                quantity,
                                                ..
                                            } => {
                                                println!(
                                                    "Received Trade: instrument={}, price={}, quantity={}, latency={}µs",
                                                    instrument, price, quantity, latency_us,
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => eprintln!("Deserialize error: {}", e),
                                }
                            } else {
                                eprintln!("Unexpected message format");
                            }
                        } else {
                            eprintln!("Unexpected data length");
                        }
                    }
                    PushInfo {
                        kind: PushKind::Subscribe,
                        data,
                    } => {
                        println!("Subscribed: {:?}", data);
                    }
                    other => eprintln!("Unexpected push kind: {:?}", other.kind),
                }
            } else {
                break;
            }
        }
    });

    // Spawn publisher task (Trades on Channel1)
    let pub_client = client.clone();
    let publisher_handle_1 = tokio::spawn(async move {
        let mut pub_con = pub_client.get_multiplexed_async_connection().await.unwrap();
        loop {
            let msg = Message::Trade {
                timestamp: OffsetDateTime::now_utc().time() - OffsetDateTime::UNIX_EPOCH.time(),
                instrument: "BTC/USD".to_string(),
                price: Decimal::new(45000, 0),
                quantity: Decimal::new(100, 2),
            };
            let mut buf = Vec::new();
            msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
            if let Err(e) = pub_con.publish(Channel::Channel1.to_string(), buf).await {
                eprintln!("Publish error: {}", e);
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    // Spawn publisher task (Ticks on Channel2)
    let pub_client = client.clone();
    let publisher_handle_2 = tokio::spawn(async move {
        let mut pub_con = pub_client.get_multiplexed_async_connection().await.unwrap();
        loop {
            let msg = Message::Tick {
                timestamp: OffsetDateTime::now_utc().time() - OffsetDateTime::UNIX_EPOCH.time(),
                instrument: "ETH/USD".to_string(),
                bid: Decimal::new(2500, 0),
                ask: Decimal::new(2501, 0),
            };
            let mut buf = Vec::new();
            msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
            if let Err(e) = pub_con.publish(Channel::Channel2.to_string(), buf).await {
                eprintln!("Publish error: {}", e);
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    receiver_handle.await?;
    publisher_handle_1.await?;
    publisher_handle_2.await?;

    Ok(())
}

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
enum Channel {
    Channel1,
    Channel2,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Message {
    #[serde(rename = "tick")]
    Tick {
        #[serde(with = "custom_serde::duration_from_nanos")]
        timestamp: time::Duration,
        instrument: String,
        bid: Decimal,
        ask: Decimal,
    },
    #[serde(rename = "trade")]
    Trade {
        #[serde(with = "custom_serde::duration_from_nanos")]
        timestamp: time::Duration,
        instrument: String,
        price: Decimal,
        quantity: Decimal,
    },
}

impl Message {
    fn timestamp(&self) -> time::Duration {
        match self {
            Message::Tick { timestamp, .. } => *timestamp,
            Message::Trade { timestamp, .. } => *timestamp,
        }
    }
}
