use crate::utils::custom_serde;
use serde::Deserialize;
use serde_this_or_that::{as_f64, as_u64};
use time::UtcDateTime;

// https://api.tardis.dev/v1/exchanges
// {
//     "id": "okex-options",
//     "name": "OKX Options",
//     "enabled": true,
//     "supportsDatasets": true,
//     "availableSince": "2020-02-01T00:00:00.000Z",
//     "availableChannels": [
//         "option/trade",
//         "option/depth",
//         "option/depth_l2_tbt",
//         "option/ticker",
//         "option/summary",
//         "option/instruments",
//         "index/ticker",
//         "system/status",
//         "option/trades",
//         "trades",
//         "trades-all",
//         "books-l2-tbt",
//         "bbo-tbt",
//         "books",
//         "tickers",
//         "opt-summary",
//         "status",
//         "instruments",
//         "index-tickers",
//         "open-interest",
//         "mark-price",
//         "price-limit",
//         "estimated-price",
//         "public-block-trades",
//         "public-struc-block-trades",
//         "option-trades"
//     ]
// },
#[derive(Debug, Deserialize)]
pub struct OkexOptionsTrade {
    pub arg: OkexOptionsTradeArg,
    pub data: Vec<OkexOptionsTradeData>,
}

#[derive(Debug, Deserialize)]
pub struct OkexOptionsTradeArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub instrument: String,
}

#[derive(Debug, Deserialize)]
pub struct OkexOptionsTradeData {
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    #[serde(rename = "instId")]
    pub instrument: String,
    #[serde(rename = "tradeId", deserialize_with = "as_u64")]
    pub trade_id: u64,
    #[serde(rename = "px", deserialize_with = "as_f64")]
    pub price: f64,
    #[serde(rename = "sz", deserialize_with = "as_u64")]
    pub quantity: u64,
    pub side: String,
}

// https://www.okx.com/docs-v5/en/#order-book-trading-market-data-ws-order-book-channel
#[derive(Debug, Deserialize)]
pub struct OkexOptionsBook {
    pub arg: OkexOptionsBookArg,
    pub action: String,
    pub data: Vec<OkexOptionsBookData>,
}

#[derive(Debug, Deserialize)]
pub struct OkexOptionsBookArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub instrument: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkexOptionsBookData {
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    pub checksum: i64,
    pub seq_id: i64,
    pub prev_seq_id: i64,
    pub asks: Vec<OkexOptionsBookUpdate>,
    pub bids: Vec<OkexOptionsBookUpdate>,
}

// An example of the array of asks and bids values: ["411.8", "10", "0", "4"]
// - "411.8" is the depth price
// - "10" is the quantity at the price (number of contracts for derivatives, quantity in base currency for Spot and Spot Margin)
// - "0" is part of a deprecated feature and it is always "0"
// - "4" is the number of orders at the price.
#[derive(Debug, Deserialize)]
pub struct OkexOptionsBookUpdate {
    #[serde(deserialize_with = "as_f64")]
    pub price: f64,
    #[serde(deserialize_with = "as_u64")]
    pub quantity: u64,
    #[serde(deserialize_with = "as_f64")]
    pub deprecated_feature: f64,
    #[serde(deserialize_with = "as_u64")]
    pub num_orders: u64,
}

#[derive(Debug, Deserialize)]
pub struct OkexOptionsOpenInterest {
    pub arg: OkexOptionsOpenInterestArg,
    pub data: Vec<OkexOptionsOpenInterestData>,
}

#[derive(Debug, Deserialize)]
pub struct OkexOptionsOpenInterestArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub instrument: String,
}

#[derive(Debug, Deserialize)]
pub struct OkexOptionsOpenInterestData {
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    #[serde(rename = "instId")]
    pub instrument: String,
    #[serde(rename = "instType")]
    pub instrument_type: String,
    #[serde(rename = "oi", deserialize_with = "as_u64")]
    pub open_interest: u64,
    #[serde(rename = "oiCcy", deserialize_with = "as_f64")]
    pub open_interest_currency: f64,
}

#[derive(Debug, Deserialize)]
pub struct OkexOptionsTick {
    pub arg: OkexOptionsTickArg,
    pub data: Vec<OkexOptionsTickData>,
}

#[derive(Debug, Deserialize)]
pub struct OkexOptionsTickArg {
    pub channel: String,
    #[serde(rename = "instFamily")]
    pub instrument_family: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkexOptionsTickData {
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    #[serde(rename = "instId")]
    pub instrument: String,
    #[serde(rename = "instType")]
    pub instrument_type: String,
    #[serde(rename = "uly")]
    pub underlying: String,
    #[serde(deserialize_with = "as_f64")]
    pub delta: f64,
    #[serde(deserialize_with = "as_f64")]
    pub gamma: f64,
    #[serde(deserialize_with = "as_f64")]
    pub vega: f64,
    #[serde(deserialize_with = "as_f64")]
    pub theta: f64,
    #[serde(deserialize_with = "as_f64")]
    pub lever: f64,
    #[serde(deserialize_with = "as_f64")]
    pub mark_vol: f64,
    #[serde(rename = "volLv", deserialize_with = "as_f64")]
    pub atm_vol: f64,
    #[serde(deserialize_with = "as_f64")]
    pub bid_vol: f64,
    #[serde(deserialize_with = "as_f64")]
    pub ask_vol: f64,
    #[serde(deserialize_with = "as_f64")]
    pub real_vol: f64,
    #[serde(rename = "deltaBS", deserialize_with = "as_f64")]
    pub delta_bs: f64,
    #[serde(rename = "gammaBS", deserialize_with = "as_f64")]
    pub gamma_bs: f64,
    #[serde(rename = "vegaBS", deserialize_with = "as_f64")]
    pub vega_bs: f64,
    #[serde(rename = "thetaBS", deserialize_with = "as_f64")]
    pub theta_bs: f64,
    #[serde(rename = "fwdPx", deserialize_with = "as_f64")]
    pub forward_price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn test_okex_options_trade() {
        let json_data = r#"{"arg":{"channel":"trades-all","instId":"BTC-USD-231229-42000-C"},"data":[{"instId":"BTC-USD-231229-42000-C","tradeId":"877","px":"0.0195","sz":"50","side":"buy","ts":"1701388808817"}]}"#;
        let _ = serde_json::from_str::<OkexOptionsTrade>(json_data).unwrap();
    }

    #[tokio::test]
    #[traced_test]
    async fn test_okex_options_book() {
        let json_data = r#"{"arg":{"channel":"books","instId":"ETH-USD-240628-1400-P"},"action":"snapshot","data":[{"asks":[["0.038","200","0","1"],["0.0385","1550","0","1"],["0.0395","2888","0","1"],["0.0435","1500","0","1"]],"bids":[["0.035","1550","0","1"],["0.031","200","0","1"],["0.0305","2888","0","1"],["0.0285","1500","0","1"],["0.026","500","0","1"]],"ts":"1701388800300","checksum":750243667,"seqId":7966474,"prevSeqId":-1}]}"#;
        let _ = serde_json::from_str::<OkexOptionsBook>(json_data).unwrap();
    }

    #[tokio::test]
    #[traced_test]
    async fn test_okex_options_open_interest() {
        let json_data = r#"{"arg":{"channel":"open-interest","instId":"ETH-USD-240126-1900-C"},"data":[{"instId":"ETH-USD-240126-1900-C","instType":"OPTION","oi":"4229","oiCcy":"422.9","ts":"1701388801195"}]}"#;
        let _ = serde_json::from_str::<OkexOptionsOpenInterest>(json_data).unwrap();
    }

    #[tokio::test]
    #[traced_test]
    async fn test_okex_options_summary() {
        let json_data = r#"{"arg":{"channel":"opt-summary","instFamily":"BTC-USD"},"data":[{"instType":"OPTION","instId":"BTC-USD-231229-40000-P","uly":"BTC-USD","delta":"-0.7094655246","gamma":"4.3187802379","vega":"0.0010549466","theta":"-0.0008715828","lever":"12.0068139439","markVol":"0.4686517873","bidVol":"0.4611296832","askVol":"0.4895109741","realVol":"","deltaBS":"-0.6261794835","gammaBS":"0.0000768666","thetaBS":"-25.384564876","vegaBS":"39.7986007518","ts":"1701388800085","fwdPx":"38026.372899316","volLv":"0.4535749446"},{"instType":"OPTION","instId":"BTC-USD-231201-44000-C","uly":"BTC-USD","delta":"0","gamma":"0","vega":"0","theta":"0","lever":"10000","markVol":"0.6553139462","bidVol":"0","askVol":"2.4218801562","realVol":"","deltaBS":"0","gammaBS":"0","thetaBS":"0","vegaBS":"0","ts":"1701388800085","fwdPx":"37728.5916503342","volLv":"0.3414095728"},{"instType":"OPTION","instId":"BTC-USD-231208-36500-P","uly":"BTC-USD","delta":"-0.2702741941","gamma":"6.1979844407","vega":"0.0004605754","theta":"-0.0012721977","lever":"107.0993566526","markVol":"0.4052029232","bidVol":"0.3979584228","askVol":"0.408944729","realVol":"","deltaBS":"-0.2609370699","gammaBS":"0.0001499623","thetaBS":"-45.0791073201","vegaBS":"17.375531255","ts":"1701388800085","fwdPx":"37804.9875020763","volLv":"0.4078636708"},{"instType":"OPTION","instId":"BTC-USD-240126-27000-P","uly":"BTC-USD","delta":"-0.0690123695","gamma":"0.6309326872","vega":"0.0004784745","theta":"-0.0002671061","lever":"137.2300596713","markVol":"0.6289562915","bidVol":"0.6225673486","askVol":"0.6335536547","realVol":"","deltaBS":"-0.0617253363","gammaBS":"0.0000130655","thetaBS":"-9.3558064867","vegaBS":"18.0507885673","ts":"1701388800085","fwdPx":"38318.8185378696","volLv":"0.5522040278"},{"instType":"OPTION","instId":"BTC-USD-231208-36500-C","uly":"BTC-USD","delta":"0.6950154763","gamma":"4.2674050995","vega":"0.0004605754","theta":"-0.0012712522","lever":"22.8068059659","markVol":"0.4052029232","bidVol":"0.3753754602","askVol":"0.4193206848","realVol":"","deltaBS":"0.7388620369","gammaBS":"0.0001499623","thetaBS":"-55.455997925","vegaBS":"17.375531255","ts":"1701388800085","fwdPx":"37804.9875020763","volLv":"0.4078636708"},{"instType":"OPTION","instId":"BTC-USD-231215-39000-C","uly":"BTC-USD","delta":"0.3629566605","gamma":"3.6806027271","vega":"0.0007576444","theta":"-0.0011571881","lever":"44.8098869506","markVol":"0.4378403139","bidVol":"0.4336639178","askVol":"0.4467864501","realVol":"","deltaBS":"0.385273164","gammaBS":"0.000116804","thetaBS":"-47.6204841328","vegaBS":"28.5826667552","ts":"1701388800085","fwdPx":"37882.6952554346","volLv":"0.427269681"},{"instType":"OPTION","instId":"BTC-USD-231202-38000-C","uly":"BTC-USD","delta":"0.3948725552","gamma":"14.7433793823","vega":"0.0002337954","theta":"-0.0036124243","lever":"141.9291799023","markVol":"0.4120324296","bidVol":"0.4101654296","askVol":"0.4321380419","realVol":"","deltaBS":"0.4019183227","gammaBS":"0.0004117385","thetaBS":"-144.6489815227","vegaBS":"8.8200978084","ts":"1701388800085","fwdPx":"37754.0652543451","volLv":"0.4011634988"},{"instType":"OPTION","instId":"BTC-USD-231203-37000-P","uly":"BTC-USD","delta":"-0.2346331578","gamma":"11.1581019425","vega":"0.0002432263","theta":"-0.0018552401","lever":"258.003237143","markVol":"0.3559559902","bidVol":"0.3491303955","askVol":"0.3613374023","realVol":"","deltaBS":"-0.2307572374","gammaBS":"0.0002833303","thetaBS":"-65.7103253556","vegaBS":"9.1758843696","ts":"1701388800085","fwdPx":"37768.3860176349","volLv":"0.3511281355"},{"instType":"OPTION","instId":"BTC-USD-231229-40000-C","uly":"BTC-USD","delta":"0.3421194963","gamma":"2.2156101959","vega":"0.0010549466","theta":"-0.0008721388","lever":"31.8496748689","markVol":"0.4686517873","bidVol":"0.4647917852","askVol":"0.469674588","realVol":"","deltaBS":"0.373516991","gammaBS":"0.0000768666","thetaBS":"-36.5170251108","vegaBS":"39.7986007518","ts":"1701388800085","fwdPx":"38026.372899316","volLv":"0.4535749446"},{"instType":"OPTION","instId":"BTC-USD-231203-38500-P","uly":"BTC-USD","delta":"-0.7574175089","gamma":"12.4361707168","vega":"0.0002627722","theta":"-0.0021192999","lever":"41.3365520182","markVol":"0.3763741337","bidVol":"0.3491303955","askVol":"0.4431243481","realVol":"","deltaBS":"-0.7332258443","gammaBS":"0.0002894932","thetaBS":"-66.1361302708","vegaBS":"9.9132658548","ts":"1701388800085","fwdPx":"37768.3860176349","volLv":"0.3511281355"},{"instType":"OPTION","instId":"BTC-USD-231202-43000-C","uly":"BTC-USD","delta":"0.0038941309","gamma":"0.2336823155","vega":"0.0000070844","theta":"-0.0002133677","lever":"10000","markVol":"0.8031445923","bidVol":"0","askVol":"1.0156329687","realVol":"","deltaBS":"0.0039526688","gammaBS":"0.0000064006","thetaBS":"-8.1319693366","vegaBS":"0.2672644155","ts":"1701388800085","fwdPx":"37754.0652543451","volLv":"0.4011634988"},{"instType":"OPTION","instId":"BTC-USD-231215-44000-P","uly":"BTC-USD","delta":"-1.0870705005","gamma":"3.6047294927","vega":"0.0002885768","theta":"-0.0005171025","lever":"6.0636627545","markVol":"0.5136796955","bidVol":"0","askVol":"0","realVol":"","deltaBS":"-0.922153677","gammaBS":"0.0000379207","thetaBS":"-7.6334601585","vegaBS":"10.8867630299","ts":"1701388800085","fwdPx":"37882.6952554346","volLv":"0.427269681"},{"instType":"OPTION","instId":"BTC-USD-240126-27000-C","uly":"BTC-USD","delta":"0.6356022566","gamma":"-0.7782965651","vega":"0.0004784745","theta":"-0.0002671061","lever":"3.3039020962","markVol":"0.6289562915","bidVol":"0","askVol":"0","realVol":"","deltaBS":"0.9382746636","gammaBS":"0.0000130655","thetaBS":"-16.7167908055","vegaBS":"18.0507885673","ts":"1701388800085","fwdPx":"38318.8185378696","volLv":"0.5522040278"},{"instType":"OPTION","instId":"BTC-USD-231229-39000-P","uly":"BTC-USD","delta":"-0.6184305715","gamma":"4.3257769239","vega":"0.0011012974","theta":"-0.0008919271","lever":"15.2772139399","markVol":"0.4592971648","bidVol":"0.4550261798","askVol":"0.4641814349","realVol":"","deltaBS":"-0.5529736103","gammaBS":"0.0000818782","thetaBS":"-27.1139799495","vegaBS":"41.5472176861","ts":"1701388800085","fwdPx":"38026.372899316","volLv":"0.4535749446"},{"instType":"OPTION","instId":"BTC-USD-231215-39000-P","uly":"BTC-USD","delta":"-0.6665398598","gamma":"5.739595768","vega":"0.0007576444","theta":"-0.0011571881","lever":"19.3001667124","markVol":"0.4378403139","bidVol":"0.4272552392","askVol":"0.4666228363","realVol":"","deltaBS":"-0.6147268359","gammaBS":"0.000116804","thetaBS":"-36.3747977396","vegaBS":"28.5826667552","ts":"1701388800085","fwdPx":"37882.6952554346","volLv":"0.427269681"},{"instType":"OPTION","instId":"BTC-USD-240628-10000-C","uly":"BTC-USD","delta":"0.2429414581","gamma":"-0.4535618181","vega":"0.0001611849","theta":"-0.0000331596","lever":"1.3345029486","markVol":"0.8654124664","bidVol":"0","askVol":"0","realVol":"","deltaBS":"0.9922841262","gammaBS":"0.0000008567","thetaBS":"-3.3266935174","vegaBS":"6.0808151617","ts":"1701388800085","fwdPx":"39566.4074709356","volLv":"0.5999160596"},{"instType":"OPTION","instId":"BTC-USD-231202-38000-P","uly":"BTC-USD","delta":"-0.6116442369","gamma":"16.7564129667","vega":"0.0002337954","theta":"-0.0036124243","lever":"73.7323945439","markVol":"0.4120324296","bidVol":"0.3881928173","askVol":"0.4516692529","realVol":"","deltaBS":"-0.5980816772","gammaBS":"0.0004117385","thetaBS":"-123.3198810974","vegaBS":"8.8200978084","ts":"1701388800085","fwdPx":"37754.0652543451","volLv":"0.4011634988"},{"instType":"OPTION","instId":"BTC-USD-231208-35000-P","uly":"BTC-USD","delta":"-0.1060545806","gamma":"3.08000714","vega":"0.0002539971","theta":"-0.0007633173","lever":"318.0583946199","markVol":"0.4408152902","bidVol":"0.439462246","askVol":"0.4431243481","realVol":"","deltaBS":"-0.1029105039","gammaBS":"0.0000760197","thetaBS":"-27.6526724248","vegaBS":"9.5822217402","ts":"1701388800085","fwdPx":"37804.9875020763","volLv":"0.4078636708"},{"instType":"OPTION","instId":"BTC-USD-231203-38500-C","uly":"BTC-USD","delta":"0.2619562583","gamma":"10.3974231823","vega":"0.0002627722","theta":"-0.0021192999","lever":"207.5594260484","markVol":"0.3763741337","bidVol":"0.3723237084","askVol":"0.3796479125","realVol":"","deltaBS":"0.2667741556","gammaBS":"0.0002894932","thetaBS":"-84.7303811105","vegaBS":"9.9132658548","ts":"1701388800085","fwdPx":"37768.3860176349","volLv":"0.3511281355"},{"instType":"OPTION","instId":"BTC-USD-231202-43000-P","uly":"BTC-USD","delta":"-1.1350590812","gamma":"2.5115887399","vega":"0.0000070844","theta":"-0.0002133677","lever":"7.1936365041","markVol":"0.8031445923","bidVol":"0","askVol":"3.5473661865","realVol":"","deltaBS":"-0.9960473311","gammaBS":"0.0000064006","thetaBS":"16.003591671","vegaBS":"0.2672644155","ts":"1701388800085","fwdPx":"37754.0652543451","volLv":"0.4011634988"},{"instType":"OPTION","instId":"BTC-USD-231215-44000-C","uly":"BTC-USD","delta":"0.0744127532","gamma":"1.2817629851","vega":"0.0002885768","theta":"-0.0005171025","lever":"291.2420898377","markVol":"0.5136796955","bidVol":"0.5127042871","askVol":"0.5200284912","realVol":"","deltaBS":"0.0778463229","gammaBS":"0.0000379207","thetaBS":"-20.3209012175","vegaBS":"10.8867630299","ts":"1701388800085","fwdPx":"37882.6952554346","volLv":"0.427269681"},{"instType":"OPTION","instId":"BTC-USD-240329-90000-P","uly":"BTC-USD","delta":"-2.279149846","gamma":"4.7773238053","vega":"0.0005528933","theta":"-0.0001788669","lever":"0.7545576985","markVol":"0.7721124559","bidVol":"0","askVol":"0","realVol":"","deltaBS":"-0.9538701465","gammaBS":"0.0000058057","thetaBS":"13.9550107872","vegaBS":"20.85829012","ts":"1701388800085","fwdPx":"38825.4999072088","volLv":"0.5836395107"},{"instType":"OPTION","instId":"BTC-USD-240628-10000-P","uly":"BTC-USD","delta":"-0.0097988312","gamma":"0.0519187605","vega":"0.0001611849","theta":"-0.0000331596","lever":"480.0866280202","markVol":"0.8654124664","bidVol":"0.7324304101","askVol":"0","realVol":"","deltaBS":"-0.0077158737","gammaBS":"0.0000008567","thetaBS":"-1.1672473579","vegaBS":"6.0808151617","ts":"1701388800085","fwdPx":"39566.4074709356","volLv":"0.5999160596"},{"instType":"OPTION","instId":"BTC-USD-231208-35000-C","uly":"BTC-USD","delta":"0.8195656513","gamma":"1.2287666758","vega":"0.0002539971","theta":"-0.000761285","lever":"12.9327706605","markVol":"0.4408152902","bidVol":"0","askVol":"0.5334561987","realVol":"","deltaBS":"0.896888603","gammaBS":"0.0000760197","thetaBS":"-37.56064808","vegaBS":"9.5822217402","ts":"1701388800085","fwdPx":"37804.9875020763","volLv":"0.4078636708"},{"instType":"OPTION","instId":"BTC-USD-240329-90000-C","uly":"BTC-USD","delta":"0.0389202888","gamma":"0.1411835356","vega":"0.0005528933","theta":"-0.0001788669","lever":"138.7046311318","markVol":"0.7721124559","bidVol":"0.7586754748","askVol":"0.7775963354","realVol":"","deltaBS":"0.0461298534","gammaBS":"0.0000058057","thetaBS":"-7.101418634","vegaBS":"20.85829012","ts":"1701388800085","fwdPx":"38825.4999072088","volLv":"0.5836395107"},{"instType":"OPTION","instId":"BTC-USD-231201-44000-P","uly":"BTC-USD","delta":"-1.1662273808","gamma":"2.3324547616","vega":"0","theta":"0","lever":"6.0158560822","markVol":"0.6553139462","bidVol":"0","askVol":"5","realVol":"","deltaBS":"-0.9999999999","gammaBS":"0","thetaBS":"3.2555786048","vegaBS":"0","ts":"1701388800085","fwdPx":"37728.5916503342","volLv":"0.3414095728"},{"instType":"OPTION","instId":"BTC-USD-240126-40000-P","uly":"BTC-USD","delta":"-0.6472707623","gamma":"3.1112053788","vega":"0.0015613752","theta":"-0.0007717416","lever":"8.8767363623","markVol":"0.5568780792","bidVol":"0.5494778952","askVol":"0.5654995916","realVol":"","deltaBS":"-0.5346167463","gammaBS":"0.0000481545","thetaBS":"-22.3525719952","vegaBS":"58.9039732621","ts":"1701388800085","fwdPx":"38318.8185378696","volLv":"0.5522040278"},{"instType":"OPTION","instId":"BTC-USD-231202-39500-C","uly":"BTC-USD","delta":"0.0883639884","gamma":"4.6939412893","vega":"0.0000979283","theta":"-0.0020212182","lever":"732.9294166761","markVol":"0.5503940401","bidVol":"0.5420011035","askVol":"0.5639737158","realVol":"","deltaBS":"0.0897283763","gammaBS":"0.0001291074","thetaBS":"-78.1243967136","vegaBS":"3.6944138373","ts":"1701388800085","fwdPx":"37754.0652543451","volLv":"0.4011634988"},{"instType":"OPTION","instId":"BTC-USD-231202-36750-C","uly":"BTC-USD","delta":"0.8200373623","gamma":"7.2139376294","vega":"0.0001417118","theta":"-0.0023284003","lever":"34.8477115366","markVol":"0.4381473629","bidVol":"0","askVol":"0.4907316748","realVol":"","deltaBS":"0.8487336512","gammaBS":"0.0002346944","thetaBS":"-105.2179488765","vegaBS":"5.3461791125","ts":"1701388800085","fwdPx":"37754.0652543451","volLv":"0.4011634988"},{"instType":"OPTION","instId":"BTC-USD-240223-50000-P","uly":"BTC-USD","delta":"-1.1003481523","gamma":"3.2373412515","vega":"0.0014536739","theta":"-0.0005198963","lever":"3.0334785479","markVol":"0.6069187901","bidVol":"0","askVol":"0","realVol":"","deltaBS":"-0.7706936042","gammaBS":"0.0000274784","thetaBS":"-8.6314067749","vegaBS":"54.8408689272","ts":"1701388800085","fwdPx":"38576.8500821878","volLv":"0.5611220615"},{"instType":"OPTION","instId":"BTC-USD-231229-35000-P","uly":"BTC-USD","delta":"-0.2547217649","gamma":"2.9443176655","vega":"0.0008582022","theta":"-0.0006874546","lever":"53.5958222343","markVol":"0.4540548565","bidVol":"0.4461760998","askVol":"0.4522796032","realVol":"","deltaBS":"-0.2360635942","gammaBS":"0.0000645415","thetaBS":"-23.2432098487","vegaBS":"32.3762805725","ts":"1701388800085","fwdPx":"38026.372899316","volLv":"0.4535749446"},{"instType":"OPTION","instId":"BTC-USD-240927-140000-P","uly":"BTC-USD","delta":"-3.4146522486","gamma":"7.0343852523","vega":"0.0013030502","theta":"-0.0001664048","lever":"0.4014539524","markVol":"0.769629947","bidVol":"0","askVol":"0","realVol":"","deltaBS":"-0.9237065404","gammaBS":"0.0000054361","thetaBS":"22.1466240347","vegaBS":"49.1584843288","ts":"1701388800085","fwdPx":"40319.3387431518","volLv":"0.6084134493"},{"instType":"OPTION","instId":"BTC-USD-231215-40000-P","uly":"BTC-USD","delta":"-0.7850662027","gamma":"5.3819633984","vega":"0.0006739431","theta":"-0.0010584767","lever":"14.0627623405","markVol":"0.4502308674","bidVol":"0.4333587426","askVol":"0.4925627258","realVol":"","deltaBS":"-0.7139564182","gammaBS":"0.0001010406","thetaBS":"-31.3561184284","vegaBS":"25.4249757085","ts":"1701388800085","fwdPx":"37882.6952554346","volLv":"0.427269681"},{"instType":"OPTION","instId":"BTC-USD-240223-25000-P","uly":"BTC-USD","delta":"-0.064184479","gamma":"0.5025327127","vega":"0.0005427341","theta":"-0.0002019333","lever":"123.0751628764","markVol":"0.627796665","bidVol":"0.6158534948","askVol":"0.6347743554","realVol":"","deltaBS":"-0.0560593628","gammaBS":"0.000009918","thetaBS":"-6.9774836953","vegaBS":"20.4750244092","ts":"1701388800085","fwdPx":"38576.8500821878","volLv":"0.5611220615"},{"instType":"OPTION","instId":"BTC-USD-231201-39000-C","uly":"BTC-USD","delta":"0.0308736295","gamma":"3.9000111074","vega":"0.000021177","theta":"-0.0006197643","lever":"4701.9702392644","markVol":"0.5853173183","bidVol":"0.5859463281","askVol":"0.6640711718","realVol":"","deltaBS":"0.0310863063","gammaBS":"0.0001050148","thetaBS":"-23.4672305335","vegaBS":"0.7989186245","ts":"1701388800085","fwdPx":"37728.5916503342","volLv":"0.3414095728"},{"instType":"OPTION","instId":"BTC-USD-231201-36250-C","uly":"BTC-USD","delta":"0.9520434631","gamma":"-0.5031724183","vega":"0.0000071443","theta":"-0.0001994789","lever":"25.486101775","markVol":"0.5584248518","bidVol":"0","askVol":"2.1337947949","realVol":"","deltaBS":"0.9912805347","gammaBS":"0.0000371342","thetaBS":"-10.1831557567","vegaBS":"0.2695253124","ts":"1701388800085","fwdPx":"37728.5916503342","volLv":"0.3414095728"},{"instType":"OPTION","instId":"BTC-USD-231201-48000-P","uly":"BTC-USD","delta":"-1.2722480518","gamma":"2.5444961036","vega":"0","theta":"0","lever":"3.6731208665","markVol":"0.6553192998","bidVol":"0","askVol":"5","realVol":"","deltaBS":"-1","gammaBS":"0","thetaBS":"3.5515402961","vegaBS":"0","ts":"1701388800085","fwdPx":"37728.5916503342","volLv":"0.3414095728"},{"instType":"OPTION","instId":"BTC-USD-240126-40000-C","uly":"BTC-USD","delta":"0.3966027579","gamma":"1.0234583383","vega":"0.0015613752","theta":"-0.0007717416","lever":"14.539005423","markVol":"0.5568780792","bidVol":"0.5551236358","askVol":"0.5615323144","realVol":"","deltaBS":"0.4653832536","gammaBS":"0.0000481545","thetaBS":"-33.257733949","vegaBS":"58.9039732621","ts":"1701388800085","fwdPx":"38318.8185378696","volLv":"0.5522040278"},{"instType":"OPTION","instId":"BTC-USD-231229-35000-C","uly":"BTC-USD","delta":"0.6654151283","gamma":"1.1040438787","vega":"0.0008582022","theta":"-0.0006866021","lever":"10.1814588508","markVol":"0.4540548565","bidVol":"0","askVol":"0.9774860723","realVol":"","deltaBS":"0.7636328803","gammaBS":"0.0000645415","thetaBS":"-32.9336023994","vegaBS":"32.3762805725","ts":"1701388800085","fwdPx":"38026.372899316","volLv":"0.4535749446"},{"instType":"OPTION","instId":"BTC-USD-240223-50000-C","uly":"BTC-USD","delta":"0.1947106747","gamma":"0.6472235973","vega":"0.0014536739","theta":"-0.0005227539","lever":"29.6019787953","markVol":"0.6069187901","bidVol":"0.6015102618","askVol":"0.6118862176","realVol":"","deltaBS":"0.2284922001","gammaBS":"0.0000274784","thetaBS":"-21.6645739069","vegaBS":"54.8408689272","ts":"1701388800085","fwdPx":"38576.8500821878","volLv":"0.5611220615"},{"instType":"OPTION","instId":"BTC-USD-231229-39000-C","uly":"BTC-USD","delta":"0.4068648239","gamma":"2.2751861329","vega":"0.0011012974","theta":"-0.0008922014","lever":"25.089040785","markVol":"0.4592971648","bidVol":"0.4559417053","askVol":"0.4605193328","realVol":"","deltaBS":"0.4467228643","gammaBS":"0.0000818782","thetaBS":"-37.9580266475","vegaBS":"41.5472176861","ts":"1701388800085","fwdPx":"38026.372899316","volLv":"0.4535749446"},{"instType":"OPTION","instId":"BTC-USD-231215-40000-C","uly":"BTC-USD","delta":"0.2708276643","gamma":"3.2701756642","vega":"0.0006739431","theta":"-0.0010584767","lever":"65.7206510354","markVol":"0.4502308674","bidVol":"0.4467864501","askVol":"0.4547210046","realVol":"","deltaBS":"0.2860435817","gammaBS":"0.0001010406","thetaBS":"-42.8901557548","vegaBS":"25.4249757085","ts":"1701388800085","fwdPx":"37882.6952554346","volLv":"0.427269681"},{"instType":"OPTION","instId":"BTC-USD-240329-27000-P","uly":"BTC-USD","delta":"-0.1282564091","gamma":"0.8015485207","vega":"0.0010593645","theta":"-0.0002638792","lever":"48.7847661596","markVol":"0.5944996356","bidVol":"0.5850308026","askVol":"0.5993740356","realVol":"","deltaBS":"-0.1077582069","gammaBS":"0.0000144473","thetaBS":"-8.7899998113","vegaBS":"39.9652706759","ts":"1701388800085","fwdPx":"38825.4999072088","volLv":"0.5836395107"},{"instType":"OPTION","instId":"BTC-USD-231208-26000-P","uly":"BTC-USD","delta":"-0.001635831","gamma":"0.0426321832","vega":"0.0000072321","theta":"-0.0000450941","lever":"10000","markVol":"0.9145320237","bidVol":"0","askVol":"1.1132890234","realVol":"","deltaBS":"-0.0015752238","gammaBS":"0.0000010433","thetaBS":"-1.6835611867","vegaBS":"0.2728384826","ts":"1701388800085","fwdPx":"37804.9875020763","volLv":"0.4078636708"},{"instType":"OPTION","instId":"BTC-USD-231201-32000-C","uly":"BTC-USD","delta":"0.8481653678","gamma":"-1.6963307357","vega":"0","theta":"0","lever":"6.586112707","markVol":"0.6240517212","bidVol":"0","askVol":"5","realVol":"","deltaBS":"1","gammaBS":"0","thetaBS":"-2.3676935307","vegaBS":"0","ts":"1701388800085","fwdPx":"37728.5916503342","volLv":"0.3414095728"},{"instType":"OPTION","instId":"BTC-USD-240329-40000-C","uly":"BTC-USD","delta":"0.4111735737","gamma":"0.3773221907","vega":"0.0022745385","theta":"-0.000552668","lever":"8.3976927359","markVol":"0.5799129958","bidVol":"0.5774014233","askVol":"0.5860989157","realVol":"","deltaBS":"0.530253901","gammaBS":"0.0000317997","thetaBS":"-24.5847275723","vegaBS":"85.8085601748","ts":"1701388800085","fwdPx":"38825.4999072088","volLv":"0.5836395107"},{"instType":"OPTION","instId":"BTC-USD-231208-26000-C","uly":"BTC-USD","delta":"0.6859677698","gamma":"-1.3325750186","vega":"0.0000072321","theta":"-0.0000365408","lever":"3.2024993516","markVol":"0.9145320237","bidVol":"0","askVol":"4.3823254541","realVol":"","deltaBS":"0.998223883","gammaBS":"0.0000010433","thetaBS":"-8.7780471433","vegaBS":"0.2728384826","ts":"1701388800085","fwdPx":"37804.9875020763","volLv":"0.4078636708"},{"instType":"OPTION","instId":"BTC-USD-240927-45000-C","uly":"BTC-USD","delta":"0.3544794367","gamma":"0.0173244504","vega":"0.0036150705","theta":"-0.0003616562","lever":"5.7213614762","markVol":"0.6029154174","bidVol":"0.5883114356","askVol":"0.6160060824","realVol":"","deltaBS":"0.5292630095","gammaBS":"0.0000192516","thetaBS":"-16.5945052363","vegaBS":"136.3810658307","ts":"1701388800085","fwdPx":"40319.3387431518","volLv":"0.6084134493"},{"instType":"OPTION","instId":"BTC-USD-231203-37000-C","uly":"BTC-USD","delta":"0.7450247483","gamma":"9.1987861301","vega":"0.0002432263","theta":"-0.0018552401","lever":"41.2915771081","markVol":"0.3559559902","bidVol":"0.2270603271","askVol":"0.3881928173","realVol":"","deltaBS":"0.7692427625","gammaBS":"0.0002833303","thetaBS":"-83.5801248639","vegaBS":"9.1758843696","ts":"1701388800085","fwdPx":"37768.3860176349","volLv":"0.3511281355"},{"instType":"OPTION","instId":"BTC-USD-231201-32000-P","uly":"BTC-USD","delta":"0","gamma":"0","vega":"0","theta":"0","lever":"10000","markVol":"0.6240517212","bidVol":"0","askVol":"2.1093807812","realVol":"","deltaBS":"0","gammaBS":"0","thetaBS":"0","vegaBS":"0","ts":"1701388800085","fwdPx":"37728.5916503342","volLv":"0.3414095728"},{"instType":"OPTION","instId":"BTC-USD-240628-60000-C","uly":"BTC-USD","delta":"0.2085356131","gamma":"0.2598195705","vega":"0.0025311537","theta":"-0.0003904496","lever":"15.1358944871","markVol":"0.6489103097","bidVol":"0.6407252713","askVol":"0.6506434643","realVol":"","deltaBS":"0.274603727","gammaBS":"0.0000179424","thetaBS":"-16.5117438537","vegaBS":"95.4895464383","ts":"1701388800085","fwdPx":"39566.4074709356","volLv":"0.5999160596"},{"instType":"OPTION","instId":"BTC-USD-240329-40000-P","uly":"BTC-USD","delta":"-0.6190798195","gamma":"2.4378289773","vega":"0.0022745385","theta":"-0.000552668","lever":"6.6964112039","markVol":"0.5799129958","bidVol":"0.5740444964","askVol":"0.5828945764","realVol":"","deltaBS":"-0.4697460989","gammaBS":"0.0000317997","thetaBS":"-15.2263144962","vegaBS":"85.8085601748","ts":"1701388800085","fwdPx":"38825.4999072088","volLv":"0.5836395107"},{"instType":"OPTION","instId":"BTC-USD-240329-27000-C","uly":"BTC-USD","delta":"0.5671646313","gamma":"-0.5892935601","vega":"0.0010593645","theta":"-0.0002638792","lever":"3.0761927252","markVol":"0.5944996356","bidVol":"0","askVol":"0","realVol":"","deltaBS":"0.892241793","gammaBS":"0.0000144473","thetaBS":"-15.1069286377","vegaBS":"39.9652706759","ts":"1701388800085","fwdPx":"38825.4999072088","volLv":"0.5836395107"},{"instType":"OPTION","instId":"BTC-USD-240126-18000-C","uly":"BTC-USD","delta":"0.4611064976","gamma":"-0.8589981006","vega":"0.0000831832","theta":"-0.0000629483","lever":"1.8825224642","markVol":"0.8525971146","bidVol":"0","askVol":"0","realVol":"","deltaBS":"0.992308658","gammaBS":"0.0000016756","thetaBS":"-7.191868123","vegaBS":"3.1381461797","ts":"1701388800085","fwdPx":"38318.8185378696","volLv":"0.5522040278"},{"instType":"OPTION","instId":"BTC-USD-240223-25000-C","uly":"BTC-USD","delta":"0.5833449344","gamma":"-0.7925261143","vega":"0.0005427341","theta":"-0.0001985369","lever":"2.7794647038","markVol":"0.627796665","bidVol":"0","askVol":"0","realVol":"","deltaBS":"0.9431264415","gammaBS":"0.000009918","thetaBS":"-13.3120301638","vegaBS":"20.4750244092","ts":"1701388800085","fwdPx":"38576.8500821878","volLv":"0.5611220615"},{"instType":"OPTION","instId":"BTC-USD-240628-60000-P","uly":"BTC-USD","delta":"-1.3079061231","gamma":"3.2927030431","vega":"0.0025311537","theta":"-0.0003904496","lever":"1.7167091677","markVol":"0.6489103097","bidVol":"0","askVol":"0","realVol":"","deltaBS":"-0.7253962729","gammaBS":"0.0000179424","thetaBS":"-3.5550668968","vegaBS":"95.4895464383","ts":"1701388800085","fwdPx":"39566.4074709356","volLv":"0.5999160596"},{"instType":"OPTION","instId":"BTC-USD-240223-28000-C","uly":"BTC-USD","delta":"0.6077346191","gamma":"-0.5891065295","vega":"0.0008551462","theta":"-0.0002967879","lever":"3.4538182048","markVol":"0.5908920079","bidVol":"0","askVol":"0","realVol":"","deltaBS":"0.897269256","gammaBS":"0.000016603","thetaBS":"-17.2620420327","vegaBS":"32.2609920712","ts":"1701388800085","fwdPx":"38576.8500821878","volLv":"0.5611220615"},{"instType":"OPTION","instId":"BTC-USD-231201-48000-C","uly":"BTC-USD","delta":"0","gamma":"0","vega":"0","theta":"0","lever":"10000","markVol":"0.6553192998","bidVol":"0","askVol":"3.437503125","realVol":"","deltaBS":"0","gammaBS":"0","thetaBS":"0","vegaBS":"0","ts":"1701388800085","fwdPx":"37728.5916503342","volLv":"0.3414095728"},{"instType":"OPTION","instId":"BTC-USD-240927-45000-P","uly":"BTC-USD","delta":"-0.7616102959","gamma":"2.2495039158","vega":"0.0036150705","theta":"-0.0003616562","lever":"3.4379229072","markVol":"0.6029154174","bidVol":"0.586709266","askVol":"0.6212703541","realVol":"","deltaBS":"-0.4707369904","gammaBS":"0.0000192516","thetaBS":"-7.303913514","vegaBS":"136.3810658307","ts":"1701388800085","fwdPx":"40319.3387431518","volLv":"0.6084134493"},{"instType":"OPTION","instId":"BTC-USD-240223-30000-C","uly":"BTC-USD","delta":"0.6064970199","gamma":"-0.3798167495","vega":"0.0011043963","theta":"-0.0003732694","lever":"4.0668807093","markVol":"0.5736951052","bidVol":"0","askVol":"0","realVol":"","deltaBS":"0.8523857173","gammaBS":"0.0000220851","thetaBS":"-20.1350061868","vegaBS":"41.6641261412","ts":"1701388800085","fwdPx":"38576.8500821878","volLv":"0.5611220615"},{"instType":"OPTION","instId":"BTC-USD-231202-36500-C","uly":"BTC-USD","delta":"0.8532178982","gamma":"5.0715245292","vega":"0.0001151416","theta":"-0.0020079419","lever":"28.769682491","markVol":"0.4650369891","bidVol":"0","askVol":"1.2194899829","realVol":"","deltaBS":"0.8879767107","gammaBS":"0.0001796642","thetaBS":"-93.8315589051","vegaBS":"4.3437987852","ts":"1701388800085","fwdPx":"37754.0652543451","volLv":"0.4011634988"},{"instType":"OPTION","instId":"BTC-USD-231229-31000-C","uly":"BTC-USD","delta":"0.7391184567","gamma":"-0.5515334704","vega":"0.0003780481","theta":"-0.0003485758","lever":"5.2740886161","markVol":"0.5255348905","bidVol":"0","askVol":"0","realVol":"","deltaBS":"0.9287246755","gammaBS":"0.0000245642","thetaBS":"-20.9601026833","vegaBS":"14.2621317613","ts":"1701388800085","fwdPx":"38026.372899316","volLv":"0.4535749446"},{"instType":"OPTION","instId":"BTC-USD-231201-37750-P","uly":"BTC-USD","delta":"-0.5242335389","gamma":"39.3856480641","vega":"0.0001204114","theta":"-0.0020706174","lever":"225.2872127065","markVol":"0.3439235413","bidVol":"0.2832125585","askVol":"0.3491303955","realVol":"","deltaBS":"-0.5197947606","gammaBS":"0.0010162086","thetaBS":"-76.652069902","vegaBS":"4.5426080567","ts":"1701388800085","fwdPx":"37728.5916503342","volLv":"0.3414095728"},{"instType":"OPTION","instId":"BTC-USD-231229-32000-P","uly":"BTC-USD","delta":"-0.1026302309","gamma":"1.4237993885","vega":"0.0004743426","theta":"-0.0004196994","lever":"149.7618236691","markVol":"0.5014734588","bidVol":"0.4980558789","askVol":"0.5078214843","realVol":"","deltaBS":"-0.0959529617","gammaBS":"0.0000322999","thetaBS":"-14.7490225149","vegaBS":"17.8949081341","ts":"1701388800085","fwdPx":"38026.372899316","volLv":"0.4535749446"}]}"#;
        let _ = serde_json::from_str::<OkexOptionsTick>(json_data).unwrap();
    }
}
