use anyhow::Result;

use aquarium::Project;

use aquarium::cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("{:?}", cli);
    let project = Project::load_or_default()?;
    Ok(())
}