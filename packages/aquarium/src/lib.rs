pub mod project;
pub(crate) mod cli;
pub mod client;
pub mod config;

pub use aquarium_macro::task;
pub mod internal {
    pub use {anyhow::Result as AnyhowResult, tokio};
    pub use crate::project::Project;
    pub use crate::cli::*;
}

pub use {
    client::query::*,
    client::signing::*,
    config::env::*,
    config::network::*,
    config::refs::*,
    config::account
};