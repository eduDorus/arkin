use std::collections::BTreeMap;
use time::OffsetDateTime;

use crate::http::sign::sign_hmac as sign;
/// Binance API Credentials.
///
/// Communication with Binance API USER_DATA endpoints requires
/// valid API credentials.
///
/// Note: Production and TESTNET API Credentials are not
/// interchangeable.
///
/// [API Documentation](https://developers.binance.com/docs/rebate/quick-start#api-key-restrictions)
///
#[derive(PartialEq, Eq, Clone)]
pub struct Credentials {
    pub api_key: String,
    pub signature: Signature,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Signature {
    Hmac(HmacSignature),
    Ed25519(Ed25519Signature),
}

#[derive(PartialEq, Eq, Clone)]
pub struct HmacSignature {
    pub api_secret: String,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Ed25519Signature {
    pub key: String,
}

// #[derive(PartialEq, Eq, Clone)]
// pub struct RsaSignature {
//     pub key: String,
//     pub password: Option<String>,
// }

impl Credentials {
    pub fn from_hmac(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Credentials {
            api_key: api_key.into(),
            signature: Signature::Hmac(HmacSignature {
                api_secret: api_secret.into(),
            }),
        }
    }

    pub fn from_ed25519(api_key: impl Into<String>, key: impl Into<String>) -> Self {
        Credentials {
            api_key: api_key.into(),
            signature: Signature::Ed25519(Ed25519Signature { key: key.into() }),
        }
    }

    pub fn sign(&self, data: &str) -> String {
        match &self.signature {
            Signature::Hmac(hmac) => sign(data, &hmac.api_secret).unwrap(),
            Signature::Ed25519(_) => todo!(),
        }
    }

    pub fn ws_auth_params(&self) -> BTreeMap<String, String> {
        let mut params = BTreeMap::new();
        let timestamp = (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64;
        let signature = self.sign(&format!("timestamp={}", timestamp));
        params.insert("apiKey".to_string(), self.api_key.clone());
        params.insert("timestamp".to_string(), timestamp.to_string());
        params.insert("signature".to_string(), signature);
        params
    }
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials").field("api_key", &"[redacted]").finish()
    }
}
