use std::path::Path;

use anyhow::{bail, Context, Result};
use log::info;
use minijinja::{syntax::SyntaxConfig, Environment};
use serde_yaml::{Mapping, Value};

use crate::{
    helper::read_yaml, password_managers::add_password_manager_functions, state::file::Delimiters,
};

pub fn load_templating_vars(host_dir: &Path, hostname: &str) -> Result<Value> {
    // Read the `vars.yml` from the host directory if it exists.
    let vars_file_exists =
        host_dir.join("vars.yaml").exists() || host_dir.join("vars.yml").exists();
    let mut variables = if vars_file_exists {
        let value = read_yaml::<Value>(host_dir, "vars")?;
        match value {
            Value::Mapping(map) => map,
            _ => bail!("Expected map for variables. Got {value:#?}"),
        }
    } else {
        Mapping::new()
    };

    // Insert default variables
    variables.insert(
        serde_yaml::to_value("host").unwrap(),
        serde_yaml::to_value(hostname).unwrap(),
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
        env.set_syntax(
            SyntaxConfig::builder()
                .block_delimiters(syntax.block.0, syntax.block.1)
                .variable_delimiters(syntax.variable.0, syntax.variable.1)
                .comment_delimiters(syntax.comment.0, syntax.comment.1)
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
