use clap::Parser;


#[derive(Parser, Debug, Clone)]
#[clap(author, about, version)]
pub struct TaskArgs {
    /// The account to use for this task
    #[clap(long, short)]
    pub account: Option<String>,
    /// The network to use for this task
    #[clap(long, short)]
    pub network: Option<String>,
}