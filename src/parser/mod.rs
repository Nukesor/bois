use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/syntax.pest"]
pub struct ConfigParser;
