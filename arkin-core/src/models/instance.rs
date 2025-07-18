use std::fmt;

use sqlx::Type;
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug, Type, Hash)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "instance_type", rename_all = "snake_case")]
pub enum InstanceType {
    Live,
    Simulation,
    Insights,
    Utility,
    Test,
}

#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq, Hash)]
pub struct Instance {
    pub id: Uuid,
    pub name: String,
    pub instance_type: InstanceType,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name={} type={}", self.name, self.instance_type)
    }
}
