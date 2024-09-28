use std::process::Command;

use minijinja::{Error, ErrorKind, Value};

/// Integration for the `pass` password manager
/// https://www.passwordstore.org/
///
/// Request passwords in templates via
/// ```
/// {{ pass("social/reddit.com") }}
/// ```
///
/// This works in two modes:
/// 1. Single password mode. The pass file is expected to have a single line with the password.
/// 2. Yaml mode with two options:
///    - There's a password + yaml separated by a `===` in the same file
///      ```yaml
///      my super secret pass
///      ===
///      user: my@email.de
///      ```
///    - Pure yaml
///      ```yaml
///      user: my@email.de
///      pass: my super secret pass
///      ```
pub fn pass(key: &str, parse_mode: Option<String>) -> Result<Value, Error> {
    let result = Command::new("pass")
        .arg("show")
        .arg(key)
        .env(
            "PASSWORD_STORE_DIR",
            "/home/nuke/.local/share/password-store",
        )
        .output();

    let output = match result {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::new(
                ErrorKind::UndefinedError,
                format!("Failed to execute `pass show {key}`: {err:?}"),
            ));
        }
    };

    if !output.status.success() {
        return Err(Error::new(
            ErrorKind::UndefinedError,
            format!(
                "`pass show {key}` failed with: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    let content = String::from_utf8_lossy(&output.stdout);

    // If there's no special parse mode, just take the first line and treat it as the password.
    let Some(parse_mode) = parse_mode else {
        return Ok(content.lines().next().unwrap_or_default().into());
    };

    // The user requested yaml mode, so we treat the content of this pass document at least
    // partially as yaml.
    if parse_mode == "yaml" {
        // Check if there's a `===` yaml document start somewhere.
        // If so, treat the content after the `===` as yaml.
        // Otherwise treat the whole file as yaml.
        let yaml = content
            .split_once("===")
            .map(|(_, _second)| _second.to_string())
            .unwrap_or(content.to_string());

        // Try to parse the yaml.
        match serde_yaml::from_str(&yaml) {
            Ok(value) => return Ok(value),
            Err(err) => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    format!("Failed to parse yaml from `pass show {key}`: {err:?}"),
                ))
            }
        }
    }

    Err(Error::new(
        ErrorKind::UndefinedError,
        format!("Found unexpected parse_mode '{parse_mode}' in 'pass' function"),
    ))
}
