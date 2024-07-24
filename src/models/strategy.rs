use super::{Direction, Instrument};

#[derive(Clone)]
pub struct Signal {
    pub strategy: String,
    pub instrument: Instrument,
    pub direction: Direction,
}
