use std::path::Path;

use anyhow::{bail, Context, Result};
use minijinja::Environment;
use serde_yaml::{Mapping, Value};

use crate::{helper::read_yaml, password_managers::add_password_manager_functions};

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
pub fn render_template(content: &str, vars: &Value) -> Result<String> {
    let mut env = Environment::new();
    env.add_template("file", content)
        .context("Failed to pre-compile template.")?;
    add_password_manager_functions(&mut env);

    let template = env.get_template("file").unwrap();
    template.render(vars).context("Failed to render template")
}
