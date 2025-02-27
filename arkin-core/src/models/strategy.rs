use std::fmt;

use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq, Hash)]
pub struct Strategy {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "name={} description={}",
            self.name,
            self.description.as_deref().unwrap_or("None")
        )
    }
}
