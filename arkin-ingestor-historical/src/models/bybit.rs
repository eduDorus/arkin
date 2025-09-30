use rust_decimal::Decimal;
use serde::Deserialize;
use time::UtcDateTime;

use arkin_core::prelude::*;
// Public Trade
// // Parameter	Type	Comments
// id	string	Message id. Unique field for option
// topic	string	Topic name
// type	string	Data type. snapshot
// ts	number	The timestamp (ms) that the system generates the data
// data	array	Object. Sorted by the time the trade was matched in ascending order
// > T	number	The timestamp (ms) that the order is filled
// > s	string	Symbol name
// > S	string	Side of taker. Buy,Sell
// > v	string	Trade size
// > p	string	Trade price
// > L	string	Direction of price change. Unique field for Perps & futures
// > i	string	Trade ID
// > BT	boolean	Whether it is a block trade order or not
// > RPI	boolean	Whether it is a RPI trade or not
// > seq	integer	cross sequence
// > mP	string	Mark price, unique field for option
// > iP	string	Index price, unique field for option
// > mIv	string	Mark iv, unique field for option
// > iv	string	iv, unique field for option
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BybitRoot {
    pub topic: String,
    pub ts: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub data: Vec<Trade>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Trade {
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    #[serde(rename = "i")]
    pub trade_id: String,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "v")]
    pub quantity: Decimal,
    #[serde(rename = "S")]
    pub side: String,
    #[serde(rename = "BT")]
    pub block_trade: bool,
    #[serde(rename = "L", default)] // Perp-only: PlusTick/ZeroPlusTick/etc.
    pub tick_direction: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bybit_spot_trade() {
        let json_data = r#"{"topic":"publicTrade.BTCUSDT","ts":1735689653900,"type":"snapshot","data":[{"i":"2290000000587387108","T":1735689653898,"p":"93611.04","v":"0.000725","S":"Buy","s":"BTCUSDT","BT":false},{"i":"2290000000587387109","T":1735689653898,"p":"93611.77","v":"0.001378","S":"Buy","s":"BTCUSDT","BT":false},{"i":"2290000000587387110","T":1735689653898,"p":"93611.91","v":"0.004331","S":"Buy","s":"BTCUSDT","BT":false},{"i":"2290000000587387111","T":1735689653898,"p":"93612.31","v":"0.018594","S":"Buy","s":"BTCUSDT","BT":false},{"i":"2290000000587387112","T":1735689653898,"p":"93612.51","v":"0.03205","S":"Buy","s":"BTCUSDT","BT":false},{"i":"2290000000587387113","T":1735689653898,"p":"93612.93","v":"0.0001","S":"Buy","s":"BTCUSDT","BT":false}]}"#;
        let _ = serde_json::from_str::<BybitRoot>(json_data).unwrap();
    }

    #[test]

    fn test_bybit_perp_trade() {
        let json_data = r#"{"topic":"publicTrade.BTCUSDT","type":"snapshot","ts":1735689654138,"data":[{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93580.00","L":"PlusTick","i":"079fa26c-36af-5007-b4be-23079b78761f","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.200","p":"93580.00","L":"ZeroPlusTick","i":"b412cc85-2712-5387-bd55-8d4d6f6b6450","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.160","p":"93580.00","L":"ZeroPlusTick","i":"758b3ab3-7cec-510e-892a-c14d345080cd","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.021","p":"93580.10","L":"PlusTick","i":"85c3d15e-dd32-5a9f-8121-0f669bf88731","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93580.20","L":"PlusTick","i":"4fc17326-6cea-5f30-881c-713860ffe548","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.021","p":"93580.30","L":"PlusTick","i":"0b57a5d0-a8ed-5362-bd59-319a6aab08b0","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.021","p":"93580.30","L":"ZeroPlusTick","i":"fddda73a-5d26-5d38-ace4-617e544ca0d3","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.004","p":"93581.10","L":"PlusTick","i":"f5119c32-a585-5416-a130-ed0c91d5676b","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93581.10","L":"ZeroPlusTick","i":"2c43ac49-46a1-52a1-b7ff-725930ba1e0e","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.004","p":"93581.60","L":"PlusTick","i":"e2f1a224-d6ea-513d-8b96-e1a2fded0930","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93581.90","L":"PlusTick","i":"ae95107a-399f-5b0b-82b7-778600a2962c","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.004","p":"93582.00","L":"PlusTick","i":"1fe29240-d0ca-5f52-b115-a8165ed905d0","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.004","p":"93582.00","L":"ZeroPlusTick","i":"1e487de3-6baf-5367-8833-44c76d0cef29","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93582.00","L":"ZeroPlusTick","i":"cb2a9fb4-a53c-5bf5-a9ba-ba2bb52a89f7","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.004","p":"93582.50","L":"PlusTick","i":"5e1f59fe-a343-5d65-8172-bdf75778624c","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.004","p":"93582.60","L":"PlusTick","i":"5cfab178-6614-56b9-bef9-42a8edf627d2","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.187","p":"93582.80","L":"PlusTick","i":"de5d243b-fade-5b9f-8c0b-e4d625742dc1","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93583.00","L":"PlusTick","i":"36cb5905-b359-590e-8088-745120346684","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93583.10","L":"PlusTick","i":"4cd8236e-bdd7-5941-85f3-8a2899ff4855","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.004","p":"93583.30","L":"PlusTick","i":"8158fee1-23a7-5fd8-a1e9-cad73d9ea600","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.004","p":"93583.90","L":"PlusTick","i":"0798f67e-194c-58e1-88ea-bbe64f625797","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93583.90","L":"ZeroPlusTick","i":"a5004f89-2208-5b51-9399-cc002ca5de6d","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.003","p":"93584.20","L":"PlusTick","i":"c40cf631-faa0-5f37-adbc-c1295551bdd5","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93584.80","L":"PlusTick","i":"1f33a71e-0cf0-58ef-bf4a-522076be008b","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93585.10","L":"PlusTick","i":"b1e290d8-b18a-5770-a74b-5beaeb34dd92","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.036","p":"93585.20","L":"PlusTick","i":"f64527f8-e4af-5116-821f-804efce7e819","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93585.40","L":"PlusTick","i":"47ef8223-209a-591a-b71a-a1353e0dd8b4","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93585.80","L":"PlusTick","i":"1a731724-19c5-53b1-9144-861d1f135214","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93586.40","L":"PlusTick","i":"42a83872-7943-50db-ac5e-741b1c1d21e2","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93586.70","L":"PlusTick","i":"e007a330-cb2c-58f8-a735-8221c699e97e","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93587.70","L":"PlusTick","i":"20014f41-a176-5e9d-8ab7-2038b17d848c","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.185","p":"93587.70","L":"ZeroPlusTick","i":"b7490be6-0f9d-5ce2-8f3e-6685d98e57bb","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.002","p":"93587.80","L":"PlusTick","i":"b10e7c9d-0416-5fe5-bc44-5b312756739a","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.006","p":"93588.00","L":"PlusTick","i":"e04432e4-c88a-58c5-975e-ce3634e01b8b","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.006","p":"93588.10","L":"PlusTick","i":"df279f80-8a8a-5f7d-96d1-12272fc92f1b","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93588.40","L":"PlusTick","i":"6fd545a9-5ef5-56d6-8c34-4f8e02a8a72c","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93588.60","L":"PlusTick","i":"e351c627-63d8-50b9-a61d-394bcf6ccad4","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.006","p":"93588.70","L":"PlusTick","i":"67f29c67-8e41-5601-976a-bafd25b1e8a7","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.010","p":"93589.00","L":"PlusTick","i":"3a311b8a-11a7-5dae-a5be-9e0c3ea82dfd","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.037","p":"93589.30","L":"PlusTick","i":"59da5a0a-e05c-5069-a5c0-6610eceddb5d","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93589.50","L":"PlusTick","i":"0a798498-c60d-5130-bbdb-5641a69bac0d","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.020","p":"93589.50","L":"ZeroPlusTick","i":"a8558ffa-8a30-5a16-b55f-7ad7c7949db6","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.011","p":"93590.00","L":"PlusTick","i":"f4368fb0-7613-5d7b-bd5a-8726d9e1c9a6","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.043","p":"93590.00","L":"ZeroPlusTick","i":"5b2aceae-1ff5-5fc4-b1c8-55517bc054b4","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.011","p":"93590.30","L":"PlusTick","i":"3b10e110-8c70-5cc3-a1aa-54ba30d056cf","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.002","p":"93590.30","L":"ZeroPlusTick","i":"abd2db65-7a1d-5790-b5cc-57b9684faf68","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93590.50","L":"PlusTick","i":"4ef533d5-0fc5-5406-91f9-8f0540ee151c","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.011","p":"93590.60","L":"PlusTick","i":"b3441927-131e-56b9-8005-6dc2fd78bee7","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.003","p":"93590.80","L":"PlusTick","i":"e4d6fd71-45ca-51c3-929d-2cc870aab152","BT":false}]}"#;
        let _ = serde_json::from_str::<BybitRoot>(json_data).unwrap();
    }
}
