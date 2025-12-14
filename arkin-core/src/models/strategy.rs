use std::fmt;

use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq, Hash)]
pub struct Strategy {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
