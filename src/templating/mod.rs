use std::{fmt::Write as _, ops::Range, path::Path};

use anyhow::{Context, Result};
use log::info;
use minijinja::{Environment, ErrorKind, UndefinedBehavior, syntax::SyntaxConfig};
use serde_yaml::Value;

use crate::{config::file::Delimiters, error::Error};

mod password_managers;
pub mod variables;

/// Take some template text, some values and render the template with the given values.
pub fn render_template(
    relative_path: &Path,
    content: &str,
    vars: &Value,
    syntax: &Option<Delimiters>,
) -> Result<String> {
    let name = relative_path.to_string_lossy();
    let mut env = Environment::new();
    env.set_trim_blocks(true);
    // Fail the render when an undefined variable is actually used.
    // SemiStrict still allows `{% if optional_var %}`-style guards.
    env.set_undefined_behavior(UndefinedBehavior::SemiStrict);
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

    env.add_template(&name, content)
        .map_err(|err| template_error(relative_path, content, vars, err))?;
    let template = env.get_template(&name).unwrap();

    let mut rendered = template
        .render(vars)
        .map_err(|err| template_error(relative_path, content, vars, err))?;
    // minijinja doesn't have a trailing newline, which is a bit annoying as many editors add one.
    rendered.push('\n');

    Ok(rendered)
}

/// Build a readable report from a minijinja compile/render error.
///
/// The error provides more context than minijinja's errir, which only carries the template name and
/// affected line.
/// Specifically, this function:
/// - Shows the error text based that info
/// - Underlines the affected span
/// - **Attempts** to figure out which part of the expression lead to the missing variable error.
/// - If this attempt succeeds, print a error message that provides some help on what other
///   variables there are.
fn template_error(
    relative_path: &Path,
    content: &str,
    vars: &Value,
    err: minijinja::Error,
) -> anyhow::Error {
    let is_undefined = matches!(err.kind(), ErrorKind::UndefinedError);

    // minijinja's span for attribute chains of 3+ segments excludes the base variable
    // (`.threads.rofl` for `{{ machine.threads.rofl }}`). We try to extend such spans leftwards
    // so we may report the full path.
    let expression_range = err
        .range()
        .map(|range| expand_base_variable(content, range));

    // Get the source text of the expression the error points at, e.g. `my_var` for `{{ my_var }}`.
    let expression = expression_range
        .clone()
        .and_then(|range| content.get(range))
        .map(str::trim)
        // If the base couldn't be recovered (e.g. `pass('x').y`), we fall back to a nameless
        // headline without a variable hint.
        .filter(|expression| !expression.is_empty() && !expression.starts_with('.'));

    let mut report = if is_undefined {
        match expression {
            Some(expression) => {
                format!("Undefined variable `{expression}` in template {relative_path:?}")
            }
            None => format!("Undefined variable in template {relative_path:?}"),
        }
    } else {
        // Equivalent to minijinja's own error display, minus the `(in name:line)` suffix,
        // which would duplicate the source snippet below.
        let mut headline = err.kind().to_string();
        if let Some(detail) = err.detail() {
            let _ = write!(headline, ": {detail}");
        }
        format!("{headline} in template {relative_path:?}")
    };

    // If there's a span range to the error, create a proper error snippet with underlines.
    if let Some(range) = expression_range {
        push_snippet(&mut report, content, range);
    } else if let Some(line) = err.line() {
        let _ = write!(report, " (line {line})");
    }

    // Also, list all available variables so that users can easily spot typos.
    if is_undefined && let Some(variable_hint) = available_variables(vars, expression) {
        let _ = write!(report, "\n{variable_hint}");
    }

    // Preserve underlying causes, e.g. subprocess failures from password manager functions.
    let mut source = std::error::Error::source(&err);
    while let Some(cause) = source {
        let _ = write!(report, "\nCaused by: {cause}");
        source = cause.source();
    }

    Error::Template(report).into()
}

/// Expand an error span to include the base variable of an attribute chain.
///
/// For attribute chains of three or more segments, minijinja's span starts at the first
/// dot (`.threads.rofl` for `{{ machine.threads.rofl }}`). The base identifier sits
/// directly before such a span, so extend the range leftwards over identifier characters.
fn expand_base_variable(content: &str, range: Range<usize>) -> Range<usize> {
    // Only spans that begin at a `.` are missing their base variable.
    if !content
        .get(range.clone())
        .is_some_and(|expression| expression.starts_with('.'))
    {
        return range;
    }

    let base_length: usize = content[..range.start]
        .chars()
        .rev()
        .take_while(|char| char.is_alphanumeric() || *char == '_')
        .map(char::len_utf8)
        .sum();

    (range.start - base_length)..range.end
}

/// Append the source line containing `range` to the report, with the range underlined:
///
/// ```text
/// 2 | size = {{ ui.size }}
///   |           ^^^^^^^
/// ```
fn push_snippet(report: &mut String, content: &str, range: Range<usize>) {
    let Some(before) = content.get(..range.start) else {
        return;
    };
    let line_start = before.rfind('\n').map(|pos| pos + 1).unwrap_or(0);
    let line_end = content[line_start..]
        .find('\n')
        .map(|pos| line_start + pos)
        .unwrap_or(content.len());
    let line_number = before.matches('\n').count() + 1;

    // Pad the underline to the expression's column, keeping tabs for it to stay aligned.
    let mut underline = String::new();
    for char in content[line_start..range.start].chars() {
        underline.push(if char == '\t' { '\t' } else { ' ' });
    }

    // The span may reach past the line end (e.g. multi-line expressions).
    let span_end = range.end.min(line_end);
    let width = content
        .get(range.start..span_end)
        .map(|span| span.chars().count())
        .unwrap_or(0)
        .max(1);
    underline.push_str(&"^".repeat(width));

    let gutter = line_number.to_string();
    let pad = " ".repeat(gutter.len());
    let line = &content[line_start..line_end];
    let _ = write!(report, "\n{gutter} | {line}\n{pad} | {underline}");
}

/// List the variables that exist at the failing lookup's position.
///
/// Since missing variables can be nested fields, the expressions must be handled in their
/// respective context.
/// - When the failing expression is a nested field, show all valid nested fields of that parent.
/// - If a top-level variable (even if nested), show valid top-level variables.
/// - For any other expression (e.g. `parent["child"]` accessors), show nothing, as the context of
///   the failing lookup is unknown.
/// - When the path dead-ends inside a value that's not a mapping (e.g. `machine.threads.rofl` for
///   `machine.threads: 8`), we explain the type mismatch instead.
fn available_variables(vars: &Value, expression: Option<&str>) -> Option<String> {
    let mut scope = vars;
    let mut resolved = Vec::new();

    if let Some(expression) = expression {
        // Make sure the expression only consists of alphanumeric keywords (+ underscore) or `.`
        // nesting delimiters.
        // More complex expressions such as `{{ parent["child"] }}` or `{{ parent[other_var] }}` are
        // too complex and need too much knowledge about the runtime. In those cases, we simply skip
        // the available variables.
        if !expression
            .chars()
            .all(|char| char.is_alphanumeric() || char == '_' || char == '.')
        {
            return None;
        }

        // Also reject invalid paths with empty segments. Those may, for example, appear when
        // encountering an leading dot whose parent cannot be determined. This may happen
        // due to minijinja's span not surrounding the full expression, but rather
        // only the last parent + current child.
        // E.g. for `parent.child.invalid`, minijinja's span would report `.child.invalid`.
        let segments: Vec<&str> = expression.split('.').collect();
        if segments.iter().any(|segment| segment.is_empty()) {
            return None;
        }

        // Go through the expression's segments.
        // For every segment, check if there's a respectively named entry in `vars`.
        // If so, we know that this entry exists and we can continue to the next nesting level.
        for segment in segments {
            let Some(next) = scope.get(segment) else {
                // The lookup failed on a value that cannot contain fields, such as a scalar or
                // `none`. We need to explain the type mismatch.
                if !resolved.is_empty() && scope.as_mapping().is_none() {
                    return Some(format!(
                        "`{}` is {}, not a mapping",
                        resolved.join("."),
                        type_name(scope),
                    ));
                }
                break;
            };
            scope = next;
            resolved.push(segment);
        }
    }

    let scope_name = resolved.last();

    // Get all available fields for the current variable scope.
    let mapping = scope.as_mapping()?;
    let mut keys: Vec<&str> = mapping.keys().filter_map(Value::as_str).collect();
    if keys.is_empty() {
        return None;
    }

    keys.sort_unstable();
    let variables = keys.join(", ");

    Some(match scope_name {
        Some(parent) => format!("Available fields in `{parent}`: {variables}"),
        None => format!("Available variables: {variables}"),
    })
}

/// A human-readable name for a YAML value's type, article included.
fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Sequence(_) => "a list",
        Value::Mapping(_) => "a mapping",
        Value::Tagged(_) => "a tagged value",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn vars(yaml: &str) -> Value {
        serde_yaml::from_str(yaml).unwrap()
    }

    fn render(content: &str, vars: &Value) -> Result<String> {
        render_template(&PathBuf::from("host/test.conf"), content, vars, &None)
    }

    /// Happy path test for variable rendering.
    #[test]
    fn provided_variables_render() {
        let rendered = render("Hello {{ name }}!", &vars("name: World")).unwrap();
        assert_eq!(rendered, "Hello World!\n");
    }

    /// Missing global variables, list other existing global variables in the error message.
    #[test]
    fn missing_variable_errors_with_report() {
        let error = render(
            "Hello {{ name }}!\nMissing: {{ missing_var }}",
            &vars("name: World\nhostname: milo"),
        )
        .unwrap_err();

        let expected = [
            "Undefined variable `missing_var` in template \"host/test.conf\"",
            "2 | Missing: {{ missing_var }}",
            "  |             ^^^^^^^^^^^",
            "Available variables: hostname, name",
        ]
        .join("\n");
        assert_eq!(error.to_string(), expected);
    }

    /// If a nested field is missing, list other existing parent fields in the error message.
    #[test]
    fn missing_nested_field_lists_parent_fields() {
        let error = render("size = {{ ui.size }}", &vars("ui:\n  font: Hack")).unwrap_err();

        let expected = [
            "Undefined variable `ui.size` in template \"host/test.conf\"",
            "1 | size = {{ ui.size }}",
            "  |           ^^^^^^^",
            "Available fields in `ui`: font",
        ]
        .join("\n");
        assert_eq!(error.to_string(), expected);
    }

    /// When a nested field is a scalar and accessed as a map, the error message must
    /// - Show the full path, since minijinja only reports the middle section.
    /// - Explain that the a scalar was accessed.
    #[test]
    fn scalar_map_access() {
        let error = render(
            "count = {{ machine.threads.rofl }}",
            &vars("machine:\n  threads: 8"),
        )
        .unwrap_err();

        let expected = [
            "Undefined variable `machine.threads.rofl` in template \"host/test.conf\"",
            "1 | count = {{ machine.threads.rofl }}",
            "  |            ^^^^^^^^^^^^^^^^^^^^",
            "`machine.threads` is a number, not a mapping",
        ]
        .join("\n");
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn missing_mid_segment_lists_parent_fields() {
        let error = render(
            "count = {{ machine.rofl.threads }}",
            &vars("machine:\n  threads: 8"),
        )
        .unwrap_err();

        let expected = [
            "Undefined variable `machine.rofl.threads` in template \"host/test.conf\"",
            "1 | count = {{ machine.rofl.threads }}",
            "  |            ^^^^^^^^^^^^^^^^^^^^",
            "Available fields in `machine`: threads",
        ]
        .join("\n");
        assert_eq!(error.to_string(), expected);
    }

    /// Entries like `ui["size"]` cannot be followed into the vars without parsing the expression,
    /// so no "Available ..." hint should be shown.
    #[test]
    fn bracket_accessors_skip_variable_hints() {
        for accessor in ["{{ ui[\"size\"] }}", "{{ ui['size'] }}"] {
            let error = render(accessor, &vars("ui:\n  font: Hack")).unwrap_err();

            let report = error.to_string();
            assert!(report.starts_with("Undefined variable `ui["), "{report}");
            assert!(!report.contains("Available"), "{report}");
        }
    }
}
