use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[clap(
    name = "bois",
    about = "A configuration management tool for your system or user dotfiles.",
    author,
    version
)]
pub struct Arguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// The name of the machine.
    /// This is usually automatically deducted via the hostname.
    #[clap(short, long)]
    pub name: String,
}
