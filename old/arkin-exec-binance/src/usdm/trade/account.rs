use typed_builder::TypedBuilder;

use crate::http::{Credentials, Method, Request};

/// `GET /fapi/v3/account`
///
/// Get current account information.
///
/// Weight(IP): 20
///
/// # Example
///
/// ```
/// use binance_spot_connector_rust::trade;
///
/// let request = trade::account();
/// ```
#[derive(Debug, Clone, TypedBuilder)]
pub struct AccountRequest {
    #[builder(default)]
    recv_window: Option<i64>,
    #[builder(default)]
    credentials: Option<Credentials>,
}

impl AccountRequest {
    pub fn new() -> Self {
        Self {
            recv_window: None,
            credentials: None,
        }
    }

    pub fn recv_window(mut self, recv_window: i64) -> Self {
        self.recv_window = Some(recv_window);
        self
    }

    pub fn credentials(mut self, credentials: &Credentials) -> Self {
        self.credentials = Some(credentials.clone());
        self
    }
}

impl From<AccountRequest> for Request {
    fn from(request: AccountRequest) -> Request {
        let mut params = vec![];

        if let Some(recv_window) = request.recv_window {
            params.push(("recvWindow".to_owned(), recv_window.to_string()));
        }

        Request {
            path: "fapi/v3/account".to_owned(),
            method: Method::Get,
            params,
            credentials: request.credentials,
            sign: true,
        }
    }
}

impl Default for AccountRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::AccountRequest;
    use crate::http::{Credentials, Method, Request};

    static API_KEY: &str = "api-key";
    static API_SECRET: &str = "api-secret";

    #[test]
    fn trade_account_convert_to_request_test() {
        let credentials = Credentials::from_hmac(API_KEY.to_owned(), API_SECRET.to_owned());

        let request: Request = AccountRequest::new().recv_window(5000).credentials(&credentials).into();

        assert_eq!(
            request,
            Request {
                path: "fapi/v3/account".to_owned(),
                credentials: Some(credentials),
                method: Method::Get,
                params: vec![("recvWindow".to_owned(), "5000".to_string()),],
                sign: true
            }
        );
    }
}
