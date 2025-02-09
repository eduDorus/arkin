mod commands;
mod utils;

pub mod prelude {
    pub use crate::commands::*;
    pub use crate::utils::parse_cli;
}
