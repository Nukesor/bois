use std::process::{Command, Stdio};

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
///
/// TODO:
/// Check if there's a smarter way to handle the case where the gpg key hasn't been added to the
/// gpg-agent yet.
pub fn pass(key: &str, parse_mode: Option<String>) -> Result<Value, Error> {
    // Run the command as if we know that the gpg key is already unlocked
    // If it isn't, we must handle this in a follow-up command we attach to our pipes.
    let result = Command::new("pass")
        .arg("show")
        .arg(key)
        .env(
            "PASSWORD_STORE_DIR",
            "/home/nuke/.local/share/password-store",
        )
        .output();

    // If the command fails this early, something went fundamentally wrong.
    let mut output = match result {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::new(
                ErrorKind::UndefinedError,
                format!("Failed to execute `pass show {key}`: {err:?}"),
            ));
        }
    };

    // If the first command failed, we just assume that the gpg key hasn't been unlocked
    // and added to gpg-agent yet. We need to give the user a chance to do this.
    if !output.status.success() {
        // Call pass again with the I/O attached to the current process
        // This will result in password prompt to be shown.
        let spawn_result = Command::new("pass")
            .arg("show")
            .arg(key)
            .env(
                "PASSWORD_STORE_DIR",
                "/home/nuke/.local/share/password-store",
            )
            .spawn();

        // Handle the case where the command spawn.
        let mut child = match spawn_result {
            Ok(child) => child,
            Err(err) => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    format!("Failed to execute `pass show {key}`: {err:?}"),
                ));
            }
        };

        // Handle case where the command failed to complete.
        let exit_code = match child.wait() {
            Ok(exit_code) => exit_code,
            Err(err) => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    format!("Failed to execute `pass show {key}`: {err:?}"),
                ));
            }
        };

        // Handle case where the command exited normally, but failed for some reason.
        if !exit_code.success() {
            return Err(Error::new(
                ErrorKind::UndefinedError,
                format!("Failed to execute `pass show {key}`"),
            ));
        }

        // The key should now be added to the gpg-agent. Try to decrypt the password once more.
        let result = Command::new("pass")
            .arg("show")
            .arg(key)
            .env(
                "PASSWORD_STORE_DIR",
                "/home/nuke/.local/share/password-store",
            )
            .output();

        // If it fails again, we finally fail for good.
        output = match result {
            Ok(output) => output,
            Err(err) => {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    format!("Failed to execute `pass show {key}`: {err:?}"),
                ));
            }
        };
    };

    let content = String::from_utf8_lossy(&output.stdout);

    // If there's no special parse mode, just take the first line and treat it as the password.
    let Some(parse_mode) = parse_mode else {
        return Ok(content.lines().next().unwrap_or_default().into());
    };

    // The user requested yaml mode, so we treat the content of this pass document at least
    // partially as yaml.
    if parse_mode == "yaml" {
        // Check if there's multile lines.
        // If so, treat the content after the first line as yaml.
        // Otherwise treat the whole file as yaml.
        let yaml = content
            .split_once("\n")
            .map(|(_, other_lines)| other_lines.to_string())
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
