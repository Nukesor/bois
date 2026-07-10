use anyhow::{Context, Result};
use log::info;
use minijinja::{Environment, syntax::SyntaxConfig};
use serde_yaml::Value;

use crate::config::file::Delimiters;

mod password_managers;
pub mod variables;

/// Take some template text, some values and render the template with the given values.
pub fn render_template(content: &str, vars: &Value, syntax: &Option<Delimiters>) -> Result<String> {
    let mut env = Environment::new();
    env.set_trim_blocks(true);
    password_managers::add_password_manager_functions(&mut env);

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
