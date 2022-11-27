use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[clap(
    name = "bois",
    about = "A configuration management tool for your bois.",
    author,
    version
)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,
}
