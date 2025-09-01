use std::{
    fs::{self, read_to_string},
    os::unix::fs::MetadataExt,
    path::Path,
};

use anyhow::{Result, bail};
use log::debug;
use winnow::{
    ModalResult, Parser,
    ascii::{newline, space0, till_line_ending},
    combinator::{alt, cut_err, delimited, not, opt, preceded, repeat, separated, terminated},
    error::{StrContext, StrContextValue},
    token::rest,
};

use super::file::{File, FileConfig};
use crate::error::Error;

pub struct ParsedFile<'s> {
    pub pre_config_block: Option<&'s str>,
    pub config_block: Option<String>,
    pub post_config_block: Option<&'s str>,
}

pub enum Line {
    ConfigDelimiter,
    Line(String),
}

/// The list of all accepted comment syntaxes that may be used to
/// comment a bois config block inside of any configuration file.
static COMMENT_PREFIXES: (&str, &str, &str, &str, &str, &str, &str, &str) =
    ("//", "--", "*/", "/*", "**", "*", "#", "%");

fn config_delimiter<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    delimited(
        alt(COMMENT_PREFIXES),
        delimited(space0, "bois_config", space0),
        newline,
    )
    .parse_next(input)
}

/// At least one line that comes before a bois config block is encountered.
///
/// The lines are separated by newlines and may not start with a bois config delimiter.
fn pre_config_lines(input: &mut &str) -> ModalResult<()> {
    separated(
        1..,
        preceded(not(config_delimiter), till_line_ending),
        newline,
    )
    .parse_next(input)
}

/// Parse content that comes before a config block.
/// It terminates with an optional newline.
/// So if this parser finishes, we're in two possible states:
/// - If no config block is found, this will consume all of the file.
/// - Otherwise, directly at the beginning of the `bois_config` block (no leading newline).
///
/// If a config block is directly at the start of the file, this may fail and backtrack to
/// allow the parsing of the [`config_block`].
fn pre_config_block<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    terminated(pre_config_lines.take(), opt(newline)).parse_next(input)
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
fn config_block<'s>(input: &mut &'s str) -> ModalResult<String> {
    let _ = config_delimiter.parse_next(input)?;
    let mut lines: Vec<&'s str> = cut_err(terminated(
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

    // The whole block might be indented by one or more spaces by the user.
    // For example:
    // ```
    // # template: true
    // # delimiters:
    // #   block: ["{{", "}}"]
    // ```
    // For yaml parsing to be clean, the lowest indentation level should be zero spaces.
    //
    // For this, we first up determine the minimum indentation level.
    // The max truncated indentation level is 10 spaces
    let min_indentation: usize = lines.iter().fold(12, |acc, line| {
        // Ignore empty lines with spaces.
        if line.trim().is_empty() {
            return acc;
        }

        let count = line.chars().take_while(|&c| c.is_whitespace()).count();
        std::cmp::min(acc, count)
    });

    // If the lines are at least one spaces indented, remove the minimum amount of indentation.
    let block = if min_indentation > 0 {
        lines
            .iter_mut()
            .map(|line| {
                // Ignore empty lines with spaces.
                if line.trim().is_empty() {
                    return line.to_string();
                }

                // Remove the first x chars, which are guaranteed to be whitespaces.
                let mut chars = line.chars();
                for _ in 0..min_indentation {
                    chars.next();
                }
                chars.as_str().to_string()
            })
            .fold(String::new(), |mut block, line| {
                // Join the block directly from the iterator.
                // -> Newline in front of all lines, except the first.
                if block.is_empty() {
                    block.push_str(&line);
                    block
                } else {
                    block.push('\n');
                    block.push_str(&line);

                    block
                }
            })
    } else {
        lines.join("\n")
    };

    Ok(block)
}

/// Parse a config file.
/// This check, if there's a bois config block somewhere in that file.
/// If we have a config block.
/// 1. Take all lines between `bois_config` and `bois_config`. For each line
///   - Strip any comment trailing spaces
///   - Strip any comment symbols
/// 2. Deserialize the config
pub fn config_file<'s>(input: &mut &'s str) -> ModalResult<ParsedFile<'s>> {
    let (pre, config, post) =
        (opt(pre_config_block), opt(config_block), opt(rest)).parse_next(input)?;

    Ok(ParsedFile {
        pre_config_block: pre,
        config_block: config,
        post_config_block: post.and_then(|block| (!block.is_empty()).then_some(block)),
    })
}

/// Read and, if applicable, parse a single configuration file.
pub fn read_file(root: &Path, relative_path: &Path) -> Result<File> {
    let path = root.join(relative_path);
    let file =
        fs::File::open(&path).map_err(|err| Error::IoPath(path.clone(), "opening file", err))?;

    let mode = file
        .metadata()
        .map_err(|err| Error::IoPath(path.clone(), "reading file", err))?
        .mode();

    let full_file_content =
        read_to_string(&path).map_err(|err| Error::IoPath(path.clone(), "reading file", err))?;

    let parsed_file = match config_file.parse(full_file_content.as_str()) {
        Ok(parsed_file) => parsed_file,
        Err(err) => {
            eprintln!("{err}");
            bail!("Encountered parsing error in file {path:?}. See log above.");
        }
    };

    // Concatenate the content from before and after the file.
    let mut content = parsed_file.pre_config_block.unwrap_or_default().to_string();
    if let Some(post_config_block) = parsed_file.post_config_block {
        content.push_str(post_config_block);
    }

    let mut config = FileConfig::default();
    if let Some(raw_config) = parsed_file.config_block {
        debug!("Found config block in file {path:?}:\n{raw_config}");
        config = serde_yaml::from_str(&raw_config)?;
    }

    Ok(File {
        relative_path: relative_path.to_path_buf(),
        mode,
        config,
        content,
    })
}
