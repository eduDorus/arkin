use std::fmt;

use sqlx::Type;
use strum::Display;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "instance_type", rename_all = "snake_case")]
pub enum InstanceType {
    Live,
    Simulation,
    Utility,
}

#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq)]
pub struct Instance {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub instance_type: InstanceType,
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name={} type={}", self.name, self.instance_type)
    }
}
