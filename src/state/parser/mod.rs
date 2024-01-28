use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "state/parser/syntax.pest"]
pub struct ConfigParser;
