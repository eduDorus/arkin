use std::fmt;

use sqlx::prelude::Type;
use strum::Display;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_type", rename_all = "snake_case")]
pub enum VenueType {
    Cex,
    Dex,
    Otc,
    UserFunds,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]
pub struct Venue {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub venue_type: VenueType,
}

impl fmt::Display for Venue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name.to_lowercase())
    }
}
