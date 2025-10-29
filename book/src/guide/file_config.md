# File Config

Individual files can be configured by adding a `bois_config` block inside the file itself.
The configuration block is commented out using the file's native comment syntax, so it doesn't interfere with the actual configuration.

This allows you to:
- Override the destination path for a specific file
- Rename files when deploying them
- Set custom ownership and permissions
- Enable templating for dynamic configuration
- Customize template delimiters to avoid conflicts

## Example

Here's a bash script with a `bois_config` block:

```bash
#!/bin/bash
# bois_config
# template: true
# owner: root
# group: root
# mode: 0o755
# path: /usr/local/bin/
# bois_config

echo "Hello from {{ host }}"
```

The configuration is extracted from between the two `# bois_config` delimiter lines, and the actual file content (without the config block) is deployed.

## Supported Comment Syntaxes

The parser supports multiple comment prefixes: `#`, `//`, `--`, `/*`, `*/`, `**`, `*`, `%`

This means you can use `bois_config` blocks in:
- Shell scripts, Python, Ruby, YAML (`#`)
- C, C++, JavaScript, Rust (`//` or `/* */`)
- SQL, Lua, Haskell (`--`)
- LaTeX (`%`)

## Configuration Options

- `path`: `PathBuf` (optional) - Override the destination path for this file.
  - If it's a relative path, it's treated as relative to the target directory.
  - If it's an absolute path, that absolute path is used directly.
  - Takes precedence over any folder-level path overrides.
- `rename`: `String` (optional) - Override the filename when deploying.
  Useful for deploying dotfiles without having dots in your bois directory.
  ```yml
  # bois_config
  # rename: .bashrc
  # bois_config
  ```
- `owner`: `String` (optional) - The file owner. Defaults to the current user.
- `group`: `String` (optional) - The file's assigned group. Defaults to the current user's group.
- `mode`: `OctalInt` (optional) - File permissions (e.g., `0o644`). If not set, the source file's permissions are preserved.
- `template`: `Boolean` (optional) - Enable Jinja2 templating for this file. Defaults to `false`.
  Read the [templating docs](./templating.md) for detailed info.
- `delimiters`: `Object` (optional) - Customize Jinja2 template delimiters. Useful when the default `{{ }}` / `{% %}` syntax conflicts with the file's content.
  ```yml
  # bois_config
  # template: true
  # delimiters:
  #   prefix: "#"
  #   block: ["{%", "%}"]
  #   variable: ["{{", "}}"]
  #   comment: ["{#", "#}"]
  # bois_config
  ```
  - `prefix`: `String` (optional) - Prefix all delimiters with this string (e.g., `#` to make templates look like comments).
  - `block`: `[String, String]` (optional) - Delimiters for logic blocks. Defaults to `["{%", "%}"]`.
  - `variable`: `[String, String]` (optional) - Delimiters for variables. Defaults to `["{{", "}}"]`.
  - `comment`: `[String, String]` (optional) - Delimiters for comments. Defaults to `["{#", "#}"]`.

## Full Example with Custom Delimiters

When working with files that already use `{{ }}` syntax (like systemd service files or some shell scripts), you can prefix delimiters to avoid conflicts:

```systemd
[Unit]
Description=Backup Service
# bois_config
# template: true
# delimiters:
#   prefix: "#"
# bois_config

#{% if host == "production" %}
ExecStart=/usr/bin/backup --important-data
#{% else %}
ExecStart=/usr/bin/backup --test-mode
#{% endif %}
```

With the `#` prefix, template blocks become `#{%` and `#{{`, making them valid comments while still being processed by the template engine.
