use std::fmt;

use derive_builder::Builder;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
#[builder(setter(into))]
pub struct Pipeline {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl fmt::Display for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name={} description={}", self.name, self.description)
    }
}
