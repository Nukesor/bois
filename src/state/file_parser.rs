use std::{fs::read_to_string, path::Path};

use anyhow::{bail, Result};
use log::debug;
use winnow::{
    ascii::{newline, space0, till_line_ending},
    combinator::{alt, cut_err, delimited, not, opt, preceded, repeat, separated, terminated},
    error::{StrContext, StrContextValue},
    token::any,
    PResult, Parser,
};

use super::file::{File, FileConfig};
use crate::error::Error;

pub struct ParsedFile {
    config: Option<String>,
    content: String,
}

pub enum Line {
    ConfigDelimiter,
    Line(String),
}

/// The list of all accepted comment syntaxes that may be used to
/// comment a bois config block inside of any configuration file.
static COMMENT_PREFIXES: (&str, &str, &str, &str, &str, &str, &str, &str) =
    ("//", "--", "*/", "/*", "**", "*", "#", "%");

fn config_delimiter<'s>(input: &mut &'s str) -> PResult<&'s str> {
    delimited(
        alt(COMMENT_PREFIXES),
        delimited(space0, "bois_config", space0),
        newline,
    )
    .parse_next(input)
}

/// A line that comes before a bois config block is encountered.
fn pre_config_line(input: &mut &str) -> PResult<()> {
    separated(
        0..,
        preceded(not(config_delimiter), till_line_ending),
        newline,
    )
    .parse_next(input)
}

/// Parse content that comes before a config block.
/// If no config block is found, this will consume all of the file.
///
/// If a config block directly at the start of the file, this may fail
/// and backtrack to allow the parsing of the config_block.
fn pre_config_block<'s>(input: &mut &'s str) -> PResult<&'s str> {
    pre_config_line.take().parse_next(input)
}

/// Parse a config block, which is everything inside a `bois_config` delimiter line.
///
/// Example:
/// ```yaml
/// # bois_config
/// # template: true <--- Starts at the start of this line
/// #
/// # path: ~/somewhere/else <--- Ends after this line
/// # bois_config
fn config_block<'s>(input: &mut &'s str) -> PResult<String> {
    let _ = config_delimiter.parse_next(input)?;
    let lines: Vec<&'s str> = cut_err(terminated(
        repeat(
            1..,
            delimited(
                alt(COMMENT_PREFIXES),
                preceded(not((space0, "bois_config")), till_line_ending),
                newline,
            ),
        ),
        config_delimiter,
    ))
    .context(StrContext::Label("full bois_config block"))
    .context(StrContext::Expected(StrContextValue::Description(
        "A commented block that ends with a commented 'bois_config' on its own line",
    )))
    .parse_next(input)?;

    Ok(lines.join("\n"))
}

/// Parse until the end of the file.
fn until_eof(input: &mut &str) -> PResult<()> {
    repeat(0.., any).parse_next(input)
}

/// Parse a config file.
/// This check, if there's a bois config block somewhere in that file.
/// If we have a config block.
/// 1. Take all lines between `bois_config` and `bois_config`. For each line
///   - Strip any comment trailing spaces
///   - Strip any comment symbols
/// 2. Deserialize the config
fn config_file(input: &mut &str) -> PResult<ParsedFile> {
    let (pre_config_block, config_block, post_config_block) = (
        opt(pre_config_block),
        opt(config_block),
        opt(until_eof.take()),
    )
        .parse_next(input)?;

    let mut content = String::new();
    if let Some(pre_config_block) = pre_config_block {
        content.push_str(pre_config_block);
    };
    if let Some(post_config_block) = post_config_block {
        content.push_str(post_config_block);
    };

    Ok(ParsedFile {
        config: config_block,
        content,
    })
}

/// Read and, if applicable, parse a single configuration file.
pub fn read_file(root: &Path, relative_path: &Path) -> Result<File> {
    let path = root.join(relative_path);

    let full_file_content =
        read_to_string(&path).map_err(|err| Error::IoPath(path.clone(), "reading file at", err))?;

    let parsed_file = match config_file.parse(full_file_content.as_str()) {
        Ok(parsed_file) => parsed_file,
        Err(err) => {
            println!("{}", err);
            bail!("Encountered parsing error in file {path:?}. See log above.");
        }
    };

    let mut config = FileConfig::default();
    if let Some(raw_config) = parsed_file.config {
        debug!("Found config block in file {path:?}:\n{raw_config}");
        config = serde_yaml::from_str(&raw_config)?;
    }

    Ok(File {
        relative_path: relative_path.to_path_buf(),
        config,
        content: parsed_file.content,
    })
}
