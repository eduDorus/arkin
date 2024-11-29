use std::fmt;

use derive_builder::Builder;
use sqlx::{prelude::FromRow, Type};
use strum::Display;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::constants;

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "instance_type", rename_all = "snake_case")]
pub enum InstanceType {
    Live,
    Simulation,
    Backtest,
}

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "instance_status", rename_all = "snake_case")]
pub enum InstanceStatus {
    New,
    Running,
    Stopped,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Builder, PartialEq, Eq, FromRow)]
#[builder(setter(into))]
pub struct Instance {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: String,
    pub start_time: OffsetDateTime,
    #[builder(default = None)]
    pub end_time: Option<OffsetDateTime>,
    pub instance_type: InstanceType,
    pub status: InstanceStatus,
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let start_time_fmt = self
            .start_time
            .format(constants::TIMESTAMP_FORMAT)
            .expect("Failed to format start time");
        let end_time_fmt = match self.end_time {
            Some(end_time) => end_time.format(constants::TIMESTAMP_FORMAT).expect("Failed to format end time"),
            None => "None".to_string(),
        };
        write!(
            f,
            "name={} start_time={} end_time={} type={} status={}",
            self.name, start_time_fmt, end_time_fmt, self.instance_type, self.status
        )
    }
}
