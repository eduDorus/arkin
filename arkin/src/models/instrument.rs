use crate::constants;

use super::{types::Maturity, Price};
use anyhow::{anyhow, Result};
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumDiscriminants, EnumString};

#[derive(Debug, Display, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Venue {
    Simulation,
    Binance,
}

#[derive(Clone, EnumDiscriminants, PartialEq, Eq, Hash)]
#[strum_discriminants(name(InstrumentType))]
#[strum_discriminants(derive(EnumString, Display))]
pub enum Instrument {
    Holding(Holding),
    Spot(SpotContract),
    Perpetual(PerpetualContract),
    Future(FutureContract),
    Option(OptionContract),
}

impl Instrument {
    pub fn new(
        instrument_type: &InstrumentType,
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
        Instrument::Spot(SpotContract::new(venue, base, quote))
    }

    pub fn perpetual(venue: Venue, base: Asset, quote: Asset) -> Self {
        Instrument::Perpetual(PerpetualContract::new(venue, base, quote))
    }

    pub fn future(venue: Venue, base: Asset, quote: Asset, maturity: Maturity) -> Self {
        Instrument::Future(FutureContract::new(venue, base, quote, maturity))
    }

    pub fn option(
        venue: Venue,
        base: Asset,
        quote: Asset,
        strike: Price,
        maturity: Maturity,
        option_type: OptionType,
    ) -> Self {
        Instrument::Option(OptionContract::new(venue, base, quote, strike, maturity, option_type))
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

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instrument::Holding(holding) => write!(f, "holding_{}", holding),
            Instrument::Spot(spot) => write!(f, "spot_{}", spot),
            Instrument::Perpetual(perpetual) => write!(f, "perp_{}", perpetual),
            Instrument::Future(future) => write!(f, "future_{}", future),
            Instrument::Option(option) => write!(f, "option_{}", option),
        }
    }
}

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
        write!(f, "{}@{}", self.asset, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SpotContract {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
}

impl SpotContract {
    pub fn new(venue: Venue, base: Asset, quote: Asset) -> Self {
        SpotContract { venue, base, quote }
    }
}

impl fmt::Display for SpotContract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}@{}", self.base, self.quote, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PerpetualContract {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
}

impl PerpetualContract {
    pub fn new(venue: Venue, base: Asset, quote: Asset) -> Self {
        PerpetualContract { venue, base, quote }
    }
}

impl fmt::Display for PerpetualContract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}@{}", self.base, self.quote, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FutureContract {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
    pub maturity: Maturity,
}

impl FutureContract {
    pub fn new(venue: Venue, base: Asset, quote: Asset, maturity: Maturity) -> Self {
        FutureContract {
            venue,
            base,
            quote,
            maturity,
        }
    }
}

impl fmt::Display for FutureContract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted = self
            .maturity
            .value()
            .format(constants::INSTRUMENT_TIMESTAMP_FORMAT)
            .expect("Unable to format expiry");
        write!(f, "{}_{}_{}@{}", self.base, self.quote, formatted, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OptionContract {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
    pub strike: Price,
    pub maturity: Maturity,
    pub option_type: OptionType,
}

impl OptionContract {
    pub fn new(
        venue: Venue,
        base: Asset,
        quote: Asset,
        strike: Price,
        maturity: Maturity,
        option_type: OptionType,
    ) -> Self {
        OptionContract {
            venue,
            base,
            quote,
            strike,
            maturity,
            option_type,
        }
    }
}

impl fmt::Display for OptionContract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted = self
            .maturity
            .value()
            .format(constants::INSTRUMENT_TIMESTAMP_FORMAT)
            .expect("Unable to format expiry");

        write!(
            f,
            "{}_{}_{:?}_{}_{}@{}",
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
            OptionType::Call => write!(f, "C"),
            OptionType::Put => write!(f, "P"),
        }
    }
}
