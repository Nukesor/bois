use std::{env::var, path::Path};

use anyhow::{Context, Result, bail};
use log::info;
use minijinja::{Environment, syntax::SyntaxConfig};
use nix::unistd::{Gid, Uid};
use serde_yaml::{Mapping, Value};

use crate::{
    helper::read_yaml,
    password_managers::add_password_manager_functions,
    state::{file::Delimiters, host::HostConfig},
};

/// Read the `vars.yml` from a host directory if it exists.
///
/// While at it, populate the variables with other useful variables that're exposed by defaults.
/// These include:
/// - The hostname itself
pub fn get_host_vars(host_dir: &Path, hostname: &str, config: &HostConfig) -> Result<Value> {
    // First up, read the vars.yml file and convert it into a [serde_yaml::Value].
    let vars_file_exists =
        host_dir.join("vars.yaml").exists() || host_dir.join("vars.yml").exists();

    // We expect vars to a top level map, so yamls consisting of a single array will throw an
    // error. If no file is found, return an empty map.
    let mut variables = if vars_file_exists {
        let value = read_yaml::<Value>(host_dir, "vars")?;
        match value {
            Value::Mapping(map) => map,
            _ => bail!("Expected map for variables. Got {value:#?}"),
        }
    } else {
        Mapping::new()
    };

    // ----------- Default template variables -----------
    // The following block injects default variables that're always available during templating.

    // Insert the host variables
    variables.insert(
        serde_yaml::to_value("host").unwrap(),
        serde_yaml::to_value(hostname).unwrap(),
    );

    // Insert the list of all enabled groups for this host.
    variables.insert(
        serde_yaml::to_value("boi_groups").unwrap(),
        serde_yaml::to_value(config.groups.clone()).unwrap(),
    );

    // Insert environment dependant variables, specifically which user currently executes boi.
    variables.insert(
        serde_yaml::to_value("USER_ID").unwrap(),
        serde_yaml::to_value(Uid::current().as_raw()).unwrap(),
    );
    variables.insert(
        serde_yaml::to_value("USER").unwrap(),
        serde_yaml::to_value(var("USER").unwrap_or_default()).unwrap(),
    );
    variables.insert(
        serde_yaml::to_value("GROUP_ID").unwrap(),
        serde_yaml::to_value(Gid::current().as_raw()).unwrap(),
    );

    Ok(Value::Mapping(variables))
}

/// Take some template text, some values and render the template with the given values.
pub fn render_template(content: &str, vars: &Value, syntax: &Option<Delimiters>) -> Result<String> {
    let mut env = Environment::new();
    env.set_trim_blocks(true);
    add_password_manager_functions(&mut env);

    if let Some(syntax) = syntax {
        info!("Found custom syntax for template file");
        let syntax = syntax.to_owned();
        let syntax_error = format!("Encountered invalid custom templating syntax {syntax:#?}");
        let block = syntax.block();
        let variable = syntax.variable();
        let comment = syntax.comment();
        env.set_syntax(
            SyntaxConfig::builder()
                .block_delimiters(block.0, block.1)
                .variable_delimiters(variable.0, variable.1)
                .comment_delimiters(comment.0, comment.1)
                .build()
                .context(syntax_error)?,
        );
    }

    env.add_template("file", content)
        .context("Failed to pre-compile template.")?;
    let template = env.get_template("file").unwrap();
    let mut rendered = template.render(vars).context("Failed to render template")?;
    // minijinja doesn't have a trailing newline, which is a bit annoying as many editors add one.
    rendered.push('\n');

    Ok(rendered)
}
