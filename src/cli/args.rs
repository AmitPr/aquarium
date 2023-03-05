use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[clap(author, about, version)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[clap(name = "init", about = "Initialize a new project")]
    Init{
        // #[arg(name = "name", long, short)]
        name: String,
        #[clap(name = "dir", long, short)]
        dir: Option<String>,
    },
    #[clap(name = "contract", about = "Manage contracts")]
    Contract(ContractArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ContractArgs {
    #[clap(subcommand)]
    pub command: ContractCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ContractCommands {
    #[clap(name = "new", about = "Create a new contract")]
    List {},
}
