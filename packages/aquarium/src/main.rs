mod cli;
mod project;

use anyhow::Result;

use project::Project;

use cli::args::Commands;
use cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("{:?}", cli);
    match cli.command {
        Commands::Init { name, dir } => {
            let project = Project::init(name, dir.map(|d| d.into()))?;
            println!("Project initialized at {}!", project.root.display());
        }
        Commands::Contract(_) => todo!(),
    }
    Ok(())
}
