use std::fmt;

use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder)]
pub struct Pipeline {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

impl fmt::Display for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name={} description={}", self.name, self.description)
    }
}
