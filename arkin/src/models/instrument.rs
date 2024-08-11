use crate::constants;

use super::{types::Maturity, Price};
use anyhow::{anyhow, Result};
use std::{fmt, str::FromStr};

use strum::{Display, EnumDiscriminants, EnumString};

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Venue {
    Simulation,
    Binance,
}

#[derive(Display, Clone, EnumDiscriminants, PartialEq, Eq, Hash)]
#[strum_discriminants(name(InstrumentType))]
#[strum_discriminants(derive(EnumString, Display))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum Instrument {
    Holding(Holding),
    Spot(Spot),
    #[strum_discriminants(strum(serialize = "perp"))]
    Perpetual(Perpetual),
    Future(Future),
    #[strum_discriminants(strum(serialize = "option"))]
    Option(EuropeanOption),
}

impl Instrument {
    pub fn new(
        instrument_type: InstrumentType,
        venue: Venue,
        base: Asset,
        quote: Asset,
        maturity: Option<Maturity>,
        strike: Option<Price>,
        option_type: Option<OptionType>,
    ) -> Result<Self> {
        let instrument = match instrument_type {
            InstrumentType::Holding => Instrument::holding(venue, base),
            InstrumentType::Spot => Instrument::spot(venue, base, quote),
            InstrumentType::Perpetual => Instrument::perpetual(venue, base, quote),
            InstrumentType::Future => {
                Instrument::future(venue, base, quote, maturity.ok_or(anyhow!("Missing maturity"))?)
            }
            InstrumentType::Option => Instrument::option(
                venue,
                base,
                quote,
                strike.ok_or(anyhow!("Missing strike"))?,
                maturity.ok_or(anyhow!("Missing maturity"))?,
                option_type.ok_or(anyhow!("Missing option type"))?,
            ),
        };
        Ok(instrument)
    }
    pub fn holding(venue: Venue, asset: Asset) -> Self {
        Instrument::Holding(Holding::new(venue, asset))
    }

    pub fn spot(venue: Venue, base: Asset, quote: Asset) -> Self {
        Instrument::Spot(Spot::new(venue, base, quote))
    }

    pub fn perpetual(venue: Venue, base: Asset, quote: Asset) -> Self {
        Instrument::Perpetual(Perpetual::new(venue, base, quote))
    }

    pub fn future(venue: Venue, base: Asset, quote: Asset, maturity: Maturity) -> Self {
        Instrument::Future(Future::new(venue, base, quote, maturity))
    }

    pub fn option(
        venue: Venue,
        base: Asset,
        quote: Asset,
        strike: Price,
        maturity: Maturity,
        option_type: OptionType,
    ) -> Self {
        Instrument::Option(EuropeanOption::new(venue, base, quote, strike, maturity, option_type))
    }

    pub fn instrument_type(&self) -> &InstrumentType {
        match self {
            Instrument::Holding(_) => &InstrumentType::Holding,
            Instrument::Spot(_) => &InstrumentType::Spot,
            Instrument::Perpetual(_) => &InstrumentType::Perpetual,
            Instrument::Future(_) => &InstrumentType::Future,
            Instrument::Option(_) => &InstrumentType::Option,
        }
    }

    pub fn venue(&self) -> &Venue {
        match self {
            Instrument::Holding(holding) => &holding.venue,
            Instrument::Spot(spot) => &spot.venue,
            Instrument::Perpetual(perpetual) => &perpetual.venue,
            Instrument::Future(future) => &future.venue,
            Instrument::Option(option) => &option.venue,
        }
    }

    pub fn base(&self) -> &Asset {
        match self {
            Instrument::Holding(holding) => &holding.asset,
            Instrument::Spot(spot) => &spot.base,
            Instrument::Perpetual(perpetual) => &perpetual.base,
            Instrument::Future(future) => &future.base,
            Instrument::Option(option) => &option.base,
        }
    }

    pub fn quote(&self) -> &Asset {
        match self {
            Instrument::Holding(holding) => &holding.asset,
            Instrument::Spot(spot) => &spot.quote,
            Instrument::Perpetual(perpetual) => &perpetual.quote,
            Instrument::Future(future) => &future.quote,
            Instrument::Option(option) => &option.quote,
        }
    }

    pub fn maturity(&self) -> Option<&Maturity> {
        match self {
            Instrument::Future(future) => Some(&future.maturity),
            Instrument::Option(option) => Some(&option.maturity),
            _ => None,
        }
    }

    pub fn strike(&self) -> Option<&Price> {
        match self {
            Instrument::Option(option) => Some(&option.strike),
            _ => None,
        }
    }

    pub fn option_type(&self) -> Option<&OptionType> {
        match self {
            Instrument::Option(option) => Some(&option.option_type),
            _ => None,
        }
    }
}

// impl fmt::Display for Instrument {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Instrument::Holding(holding) => write!(f, "holding_{}", holding),
//             Instrument::Spot(spot) => write!(f, "spot_{}", spot),
//             Instrument::Perp(perpetual) => write!(f, "perp_{}", perpetual),
//             Instrument::Future(future) => write!(f, "future_{}", future),
//             Instrument::Option(option) => write!(f, "option_{}", option),
//         }
//     }
// }

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Asset {
    pub underlier: String,
}

impl From<&str> for Asset {
    fn from(underlier: &str) -> Self {
        Asset {
            underlier: underlier.to_lowercase(),
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.underlier)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Holding {
    pub venue: Venue,
    pub asset: Asset,
}

impl Holding {
    pub fn new(venue: Venue, asset: Asset) -> Self {
        Holding { venue, asset }
    }
}

impl fmt::Display for Holding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "holding_{}@{}", self.asset, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Spot {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
}

impl Spot {
    pub fn new(venue: Venue, base: Asset, quote: Asset) -> Self {
        Spot { venue, base, quote }
    }
}

impl fmt::Display for Spot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "spot_{}_{}@{}", self.base, self.quote, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Perpetual {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
}

impl Perpetual {
    pub fn new(venue: Venue, base: Asset, quote: Asset) -> Self {
        Perpetual { venue, base, quote }
    }
}

impl fmt::Display for Perpetual {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "perp_{}_{}@{}", self.base, self.quote, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Future {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
    pub maturity: Maturity,
}

impl Future {
    pub fn new(venue: Venue, base: Asset, quote: Asset, maturity: Maturity) -> Self {
        Future {
            venue,
            base,
            quote,
            maturity,
        }
    }
}

impl fmt::Display for Future {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted = self
            .maturity
            .value()
            .format(constants::INSTRUMENT_TIMESTAMP_FORMAT)
            .expect("Unable to format expiry");
        write!(f, "future_{}_{}_{}@{}", self.base, self.quote, formatted, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EuropeanOption {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
    pub strike: Price,
    pub maturity: Maturity,
    pub option_type: OptionType,
}

impl EuropeanOption {
    pub fn new(
        venue: Venue,
        base: Asset,
        quote: Asset,
        strike: Price,
        maturity: Maturity,
        option_type: OptionType,
    ) -> Self {
        EuropeanOption {
            venue,
            base,
            quote,
            strike,
            maturity,
            option_type,
        }
    }
}

impl fmt::Display for EuropeanOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted = self
            .maturity
            .value()
            .format(constants::INSTRUMENT_TIMESTAMP_FORMAT)
            .expect("Unable to format expiry");

        write!(
            f,
            "option_{}_{}_{:?}_{}_{}@{}",
            self.base, self.quote, formatted, self.strike, self.option_type, self.venue
        )
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum OptionType {
    Call,
    Put,
}

impl FromStr for OptionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "c" => Ok(OptionType::Call),
            "call" => Ok(OptionType::Call),
            "p" => Ok(OptionType::Put),
            "put" => Ok(OptionType::Put),
            _ => Err(anyhow!("Unknown option type: {}", s)),
        }
    }
}

impl fmt::Display for OptionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptionType::Call => write!(f, "c"),
            OptionType::Put => write!(f, "p"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instrument_type() {
        assert_eq!("holding".parse::<InstrumentType>().unwrap(), InstrumentType::Holding);
        assert_eq!("spot".parse::<InstrumentType>().unwrap(), InstrumentType::Spot);
        assert_eq!("perp".parse::<InstrumentType>().unwrap(), InstrumentType::Perpetual);
        assert_eq!("future".parse::<InstrumentType>().unwrap(), InstrumentType::Future);
        assert_eq!("option".parse::<InstrumentType>().unwrap(), InstrumentType::Option);

        // Check the other way around
        assert_eq!(InstrumentType::Holding.to_string(), "holding");
        assert_eq!(InstrumentType::Spot.to_string(), "spot");
        assert_eq!(InstrumentType::Perpetual.to_string(), "perp");
        assert_eq!(InstrumentType::Future.to_string(), "future");
        assert_eq!(InstrumentType::Option.to_string(), "option");
    }
}
