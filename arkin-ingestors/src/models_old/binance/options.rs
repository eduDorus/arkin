use crate::utils::custom_serde;
use rust_decimal::Decimal;
use serde::Deserialize;
use time::UtcDateTime;

// https://binance-docs.github.io/apidocs/voptions/en/#websocket-market-streams
// https://api.tardis.dev/v1/exchanges
// {
//     "id": "binance-european-options",
//     "name": "Binance European Options",
//     "enabled": true,
//     "supportsDatasets": true,
//     "availableSince": "2023-06-15T00:00:00.000Z",
//     "availableChannels": [
//         "trade",
//         "depth100",
//         "index",
//         "markPrice",
//         "ticker",
//         "openInterest"
//     ]
// }
#[derive(Debug, Deserialize)]
pub struct BinanceOptionsTrade {
    pub stream: String,
    pub data: BinanceOptionsTradeData,
}

#[derive(Debug, Deserialize)]
pub struct BinanceOptionsTradeData {
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "t")]
    pub trade_id: i64,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "S")]
    pub side: i64,
    #[serde(rename = "b")]
    pub bid_order_id: i64,
    #[serde(rename = "a")]
    pub ask_order_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct BinanceOptionsBook {
    pub stream: String,
    pub data: BinanceOptionsBookData,
}

#[derive(Debug, Deserialize)]
pub struct BinanceOptionsBookData {
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "u")]
    pub update_id: i64,
    #[serde(rename = "pu")]
    pub same_update_id: i64, // same as update id in event
    #[serde(rename = "b")]
    pub bids: Vec<BinanceOptionsBookUpdate>,
    #[serde(rename = "a")]
    pub asks: Vec<BinanceOptionsBookUpdate>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceOptionsBookUpdate {
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct BinanceOptionsTick {
    pub stream: String,
    pub data: BinanceOptionsTickData,
}

#[derive(Debug, Deserialize)]
pub struct BinanceOptionsTickData {
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "o")]
    pub open: Decimal,
    #[serde(rename = "h")]
    pub high: Decimal,
    #[serde(rename = "l")]
    pub low: Decimal,
    #[serde(rename = "c")]
    pub close: Decimal,
    #[serde(rename = "V")]
    pub volume: Decimal,
    #[serde(rename = "A")]
    pub quote_asset_volume: Decimal,
    #[serde(rename = "P")]
    pub price_change_pct: Decimal,
    #[serde(rename = "p")]
    pub price_change: Decimal,
    #[serde(rename = "Q")]
    pub last_trade_quantity: Decimal,
    #[serde(rename = "F")]
    pub first_trade_id: i64,
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    #[serde(rename = "n")]
    pub num_trades: i64,
    #[serde(rename = "bo")]
    pub bid_price: Decimal,
    #[serde(rename = "bq")]
    pub bid_amount: Decimal,
    #[serde(rename = "b")]
    pub bid_implied_volatility: Decimal,
    #[serde(rename = "ao")]
    pub ask_price: Decimal,
    #[serde(rename = "aq")]
    pub ask_amount: Decimal,
    #[serde(rename = "a")]
    pub ask_implied_volatility: Decimal,
    #[serde(rename = "d")]
    pub delta: Decimal,
    #[serde(rename = "g")]
    pub gamma: Decimal,
    #[serde(rename = "t")]
    pub theta: Decimal,
    #[serde(rename = "v")]
    pub vega: Decimal,
    #[serde(rename = "vo")]
    pub implied_volatility: Decimal,
    #[serde(rename = "mp")]
    pub mark_price: Decimal,
    #[serde(rename = "hl")]
    pub bid_max_price: Decimal,
    #[serde(rename = "ll")]
    pub ask_min_price: Decimal,
    #[serde(rename = "eep")]
    pub estimated_strike_price: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct BinanceOptionsOpenInterest {
    pub stream: String,
    pub data: Vec<BinanceOptionsOpenInterestData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceOptionsOpenInterestData {
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "o")]
    pub open_interest_contracts: Decimal,
    #[serde(rename = "h")]
    pub open_interest_usdt: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_optionss_trade() {
        let json_data = r#"{"stream":"ETH-231202-2100-C@trade","data":{"e":"trade","E":1701388852591,"s":"ETH-231202-2100-C","t":"20","p":"6.9","q":"0.17","b":"4674893542539575296","a":"4692907941024256001","T":1701388852588,"S":"1"}}"#;
        let _ = serde_json::from_str::<BinanceOptionsTrade>(json_data).unwrap();
    }

    #[test]
    fn test_binance_optionss_book() {
        let json_data = r#"{"stream":"ETH-231208-1950-C@depth100@100ms","data":{"e":"depth","E":1701388808343,"T":1701388808341,"s":"ETH-231208-1950-C","u":2690466,"pu":2690466,"b":[["91.5","38.87"],["13.2","4.17"]],"a":[["142.4","31"],["145.4","33.74"],["145.5","6.96"],["158.2","5.26"]]}}"#;
        let _ = serde_json::from_str::<BinanceOptionsBook>(json_data).unwrap();
    }

    #[test]
    fn test_binance_optionss_ticker() {
        let json_data = r#"{"stream":"ETH-231215-2150-C@ticker","data":{"e":"24hrTicker","E":1701388809057,"T":1701388809000,"s":"ETH-231215-2150-C","o":"44.1","h":"44.1","l":"44.1","c":"44.1","V":"0","A":"0","P":"0","p":"0","Q":"6.3","F":"0","L":"0","n":0,"bo":"40.2","ao":"41","bq":"6.97","aq":"31.97","b":"0.48034787","a":"0.48577666","d":"0.33050441","t":"-2.48327689","g":"0.00184462","v":"1.47365585","vo":"0.48306227","mp":"40.5","hl":"321.7","ll":"0.1","eep":"0"}}"#;
        let _ = serde_json::from_str::<BinanceOptionsTick>(json_data).unwrap();
    }

    #[test]
    fn test_binance_optionss_open_interest() {
        let json_data = r#"{"stream":"BTC@openInterest@231229","data":[{"e":"openInterest","E":1701389040045,"s":"BTC-231229-26000-P","o":"18.33","h":"691027.0106999883"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-37000-P","o":"63.01","h":"2375428.911304215"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-30000-C","o":"163.33","h":"6157416.347933939"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-38000-P","o":"78.59","h":"2962783.020780801"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-32000-P","o":"149.47","h":"5634904.92576799"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-60000-C","o":"443.28","h":"1.6711317692476314E7"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-25000-C","o":"71.85","h":"2708690.1646914436"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-36000-C","o":"74.25","h":"2799168.3330318676"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-27000-P","o":"220.82","h":"8324745.472055178"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-20000-C","o":"2.03","h":"76529.4507212753"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-15000-P","o":"110.66","h":"4171797.5452297167"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-39000-P","o":"0.47","h":"17718.6412999997"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-80000-P","o":"0.0","h":"0.0"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-31000-C","o":"42.1","h":"1587137.869638271"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-22000-P","o":"109.97","h":"4145785.0718318447"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-33000-P","o":"154.13","h":"5810583.369295646"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-45000-P","o":"66.94","h":"2523586.9119616593"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-40000-P","o":"28.22","h":"1063872.462736152"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-26000-C","o":"31.2","h":"1176216.188425512"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-37000-C","o":"66.32","h":"2500213.385140383"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-32000-C","o":"24.92","h":"939464.9812680692"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-28000-P","o":"243.18","h":"9167700.407093462"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-23000-P","o":"48.82","h":"1840476.7409914583"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-50000-C","o":"685.22","h":"2.5832271045927223E7"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-34000-P","o":"167.14","h":"6301050.440174362"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-15000-C","o":"0.05","h":"1884.9618404255"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-38000-C","o":"99.38","h":"3746550.1540297237"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-27000-C","o":"219.2","h":"8263672.708425392"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-80000-C","o":"321.1","h":"1.210522493921256E7"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-29000-P","o":"97.28","h":"3667381.756731853"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-45000-C","o":"172.39","h":"6498971.433419039"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-33000-C","o":"20.41","h":"769441.4232616891"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-22000-C","o":"28.09","h":"1058971.5619510459"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-35000-P","o":"143.59","h":"5413233.413333951"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-40000-C","o":"157.89","h":"5952332.499695644"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-24000-P","o":"49.02","h":"1848016.5883531603"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-39000-C","o":"0.93","h":"35060.2902319143"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-30000-P","o":"207.87","h":"7836540.355384974"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-28000-C","o":"164.86","h":"6215096.180250959"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-23000-C","o":"1.46","h":"55040.8857404246"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-60000-P","o":"0.0","h":"0.0"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-34000-C","o":"58.2","h":"2194095.582255282"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-25000-P","o":"104.04","h":"3922228.5975573803"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-36000-P","o":"71.11","h":"2680792.729453146"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-29000-C","o":"62.84","h":"2369020.0410467684"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-20000-P","o":"127.81","h":"4818339.456495663"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-31000-P","o":"98.98","h":"3731470.4593063197"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-35000-C","o":"127.98","h":"4824748.32675311"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-50000-P","o":"81.68","h":"3079273.662519097"},{"e":"openInterest","E":1701389040045,"s":"BTC-231229-24000-C","o":"10.75","h":"405266.7956914825"}]}"#;
        let _ = serde_json::from_str::<BinanceOptionsOpenInterest>(json_data).unwrap();
    }
}
