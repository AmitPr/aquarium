use anyhow::Result;

use aquarium::Project;

use aquarium::cli::args::Commands;
use aquarium::cli::Cli;
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
