mod project;
mod cli;

pub use aquarium_lib::*;
pub use aquarium_macro::task;
pub mod internal {
    pub use {anyhow::Result as AnyhowResult, tokio};
    pub use crate::project::Project;
}