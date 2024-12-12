use crate::ws::Stream;

/// User Data Stream.
///
/// A User Data Stream listenKey is valid for 60 minutes after creation.
///
/// Possible Updates:
///
/// * `outboundAccountPosition` is sent any time an account balance has
/// changed and contains the assets that were possibly changed by
/// the event that generated the balance change.
///
/// * `balanceUpdate` occurs during the following: Deposits or
/// withdrawals from the account; Transfer of funds between
/// accounts (e.g. Spot to Margin).
///
/// * `executionReport` occurs when an order is updated. If the order is
/// an OCO, an event will be displayed named `ListStatus` in addition
/// to the `executionReport` event.
///
/// [API Documentation](https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream)
pub struct UserDataStream {
    method: String,
}

impl UserDataStream {
    pub fn new() -> Self {
        Self {
            method: "userDataStream.start".to_string(),
        }
    }
}

impl From<UserDataStream> for Stream {
    /// Returns stream name as `<listen_key>`
    fn from(stream: UserDataStream) -> Stream {
        Stream::new(&stream.method)
    }
}
