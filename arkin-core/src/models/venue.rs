use std::fmt;

use derive_builder::Builder;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Builder)]
#[builder(setter(into))]
pub struct Venue {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub venue_type: String,
}

impl fmt::Display for Venue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
