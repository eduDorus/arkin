use std::fmt;

use typed_builder::TypedBuilder;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]

pub struct Portfolio {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl fmt::Display for Portfolio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
