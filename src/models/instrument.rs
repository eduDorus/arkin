use core::fmt;

use super::{types::Expiry, Price, Venue};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Instrument {
    Holding(Holding),
    Spot(SpotContract),
    Perpetual(PerpetualContract),
    Future(FutureContract),
    Option(OptionContract),
}

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instrument::Holding(holding) => write!(f, "{}", holding),
            Instrument::Spot(spot) => write!(f, "SPOT-{}", spot),
            Instrument::Perpetual(perpetual) => write!(f, "PERP-{}", perpetual),
            Instrument::Future(future) => write!(f, "FUTURE-{}", future),
            Instrument::Option(option) => write!(f, "OPTION-{}", option),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Asset {
    pub ticker: String,
}

impl Asset {
    pub fn new(ticker: &str) -> Self {
        Asset {
            ticker: ticker.to_uppercase(),
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ticker)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Holding {
    pub venue: Venue,
    pub asset: Asset,
}

impl Holding {
    pub fn new(venue: &Venue, asset: &Asset) -> Self {
        Holding {
            venue: venue.to_owned(),
            asset: asset.to_owned(),
        }
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
    pub fn new(venue: &Venue, base: &Asset, quote: &Asset) -> Self {
        SpotContract {
            venue: venue.to_owned(),
            base: base.to_owned(),
            quote: quote.to_owned(),
        }
    }
}

impl fmt::Display for SpotContract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}@{}", self.base, self.quote, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PerpetualContract {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
}

impl PerpetualContract {
    pub fn new(venue: &Venue, base: &Asset, quote: &Asset) -> Self {
        PerpetualContract {
            venue: venue.to_owned(),
            base: base.to_owned(),
            quote: quote.to_owned(),
        }
    }
}

impl fmt::Display for PerpetualContract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}@{}", self.base, self.quote, self.venue)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FutureContract {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
    pub expiry: Expiry,
}

impl FutureContract {
    pub fn new(venue: &Venue, base: &Asset, quote: &Asset, expiry: &Expiry) -> Self {
        FutureContract {
            venue: venue.to_owned(),
            base: base.to_owned(),
            quote: quote.to_owned(),
            expiry: expiry.to_owned(),
        }
    }
}

impl fmt::Display for FutureContract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}-{}@{}", self.base, self.quote, self.venue, self.expiry)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OptionContract {
    pub venue: Venue,
    pub base: Asset,
    pub quote: Asset,
    pub strike: Price,
    pub expiry: Expiry,
    pub option_type: OptionType,
}

impl OptionContract {
    pub fn new(
        venue: &Venue,
        base: &Asset,
        quote: &Asset,
        strike: &Price,
        expiry: &Expiry,
        option_type: &OptionType,
    ) -> Self {
        OptionContract {
            venue: venue.to_owned(),
            base: base.to_owned(),
            quote: quote.to_owned(),
            strike: strike.to_owned(),
            expiry: expiry.to_owned(),
            option_type: option_type.to_owned(),
        }
    }
}

impl fmt::Display for OptionContract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}-{}-{}-{}-{}@{}",
            self.base, self.quote, self.expiry, self.strike, self.option_type, self.venue
        )
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum OptionType {
    Call,
    Put,
}

impl fmt::Display for OptionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptionType::Call => write!(f, "C"),
            OptionType::Put => write!(f, "P"),
        }
    }
}
