# Templating

`bois` uses the [`minijinja`](https://docs.rs/minijinja/latest/minijinja/) templating engine.

It is based on the syntax and behavior of the [Jinja2](https://jinja.palletsprojects.com/en/stable/) template engine for Python.

## Documentation

Here're some links to get started with how to write templates with `minijinja`.

- [Syntax](https://docs.rs/minijinja/latest/minijinja/syntax/index.html)
- [Built-in filters](https://docs.rs/minijinja/latest/minijinja/filters/index.html#functions)
- [Built-in functions](https://docs.rs/minijinja/latest/minijinja/functions/index.html#functions)

### How to use templating in bois

Templating functionality is opt-in in bois.
To enable templating for a file, you must enable the `template` option in its [File configuration](./file_configuration.md) block.

```yaml
# bois_config
# template: true
# bois_config
```

Once this configuration flag is found, `bois` will treat the whole file as a template.
If there's a `vars.yml` file in the current host's directory, it'll be read and injected into the templating environment.

For example, consider the following `vars.yml` file in a host's directory.

```yml
some_secret: "lorem"
some_secret_list:
  - "ipsum"
  - "dolor"
some_secret_dict:
  lorem: sit
```

These variables can then be used like this:

```django,jinja
SECRET={{ some_secret }}

{% for item in some_secret_list %}
# Useless comment: {{ item }}
{% endfor %}

{% if 'lorem' in some_secret_dict %}
OTHER_SECRET={{ some_secret_dict['lorem'] }}
{% endif %}
```

Which results in the following output:

```
SECRET=lorem

# Useless comment: ipsum
# Useless comment: dolor

OTHER_SECRET=sit
```

### Pre-defined variables

`bois` pre-populates the templating environment with a few variables for your convenience:

- `host`: String - The name of the current host.
- `boi_groups`: `List<String>` - A list with all groups that're enabled for the host.

The following example checks whether the `encrypt` group is enabled for the current host.
If so, it adds the `do_encryption=true` flag to the configuration file.

```django,jinja
{% if "encrypt" in boi_groups %}
do_encryption=true
{% endif %}
```

### Pre-defined functions

On top of `minijinja`'s native filters and functions, `bois` exposes some functions itself.
Most of those functions are [integrations with password managers](../password_managers/password_managers.md), enabling you to inject secrets into your configuration files.

### Custom delimiters

It's possible to set custom delimiters for templating.
This is sometimes useful for files that already have a Jinja2-style templating syntax themselves or for other formats that heavily use curly braces like Latex.

To change the syntax from `["{{", "}}", "{%", "%}", "{#", "#}"]`, the `delimiters` option can be used:

```yml
delimiters:
  block: ["{%", "%}"]
  variable: ["{{", "}}"]
  comment: ["{#", "#}"]
```

For example, the following is really handy to not interfere with a configuration format that uses `#` as a comment. \
(The syntax highlighting is a bit off, as we're now using a slightly different syntax. Just imagine the `#` prefix would be highlighted as well.)

```django,jinja
# bois_config
# template: true
# delimiters:
#   block: ["#{%", "%}"]
#   variable: ["#{{", "}}"]
#   comment: ["#{#", "#}"]
# bois_config

...

#{# this is how a template comment now looks like #}

...

important_option=#{{ some_variable }}
```
