use std::process::Command;

use anyhow::Result;

use aquarium::internal::Project;

use aquarium::internal::args::{Commands, RunTaskArgs, TaskCommands};
use aquarium::internal::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { name, dir } => {
            let project = Project::init(name, dir.map(|d| d.into()))?;
            println!("Project initialized at {}!", project.root.display());
        }
        Commands::Task(args) => run_task(args).await?,
        Commands::Contract(_) => todo!(),
    }
    Ok(())
}

async fn run_task(args: RunTaskArgs) -> Result<()> {
    match args.command {
        TaskCommands::List {  } => {
            let project = Project::load()?;
            let manifest = Command::new("cargo")
                .arg("read-manifest")
                .current_dir(&project.root.join(project.config.scripts_path.clone()))
                .output()?;
            let manifest = serde_json::from_slice::<serde_json::Value>(&manifest.stdout)?;
            let targets = manifest["targets"].as_array()
                .map(|targets|{
                    targets
                        .iter()
                        .filter_map(|t| t["name"].as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();
            println!("Available tasks:");
            for target in targets {
                println!("  - \"{}\"", target);
            }
            Ok(())
        }
        TaskCommands::Run { name } => {
            let project = Project::load()?;
            let manifest = Command::new("cargo")
                .arg("read-manifest")
                .current_dir(&project.root.join(project.config.scripts_path.clone()))
                .output()?;
            let manifest = serde_json::from_slice::<serde_json::Value>(&manifest.stdout)?;
            let targets = manifest["targets"].as_array()
                .map(|targets|{
                    targets
                        .iter()
                        .filter_map(|t| t["name"].as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();
            if !targets.contains(&name) {
                println!("Task \"{}\" not found", name);
                return Ok(());
            }
            let status = Command::new("cargo")
                .arg("run")
                .arg("--bin")
                .arg(name)
                .current_dir(&project.root.join(project.config.scripts_path.clone()))
                .status()?;
            println!("Task completed with status {}", status);
            Ok(())
        }
    }
}