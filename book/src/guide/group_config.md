# Group Config

Groups are a tool for reusing configuration files across multiple hosts.

All configuration that's shared between machines should be placed into groups.
For instance, all machines might share the same base packages, shell configuration, or editor setup.

Groups are located in the top-level `groups` directory.
The directory structure might look something like this:

```
 📂 groups/
 │ 📂 base/
 │ │ 📁 shell/
 │ │ 📁 git/
 │ │ group.yml
 │ └ vars.yml
 │ 📂 laptop/
 │ │ 📁 upower/
 │ │ group.yml
 │ └ vars.yml
 └ 📂 games/
   │ group.yml
   └ vars.yml
 📁 hosts/
```

- The `group.yml` file is optional.
  It allows you to set group-specific configuration and specify packages that should be installed when this group is included.
- All variables inside the `vars.yml` are exposed to the templating engine.
  Read the [templating docs](./templating.md) for detailed info.
  The top level of the `vars.yml` is expected to be an object.
- All other files that're located in a group's directory are considered configuration files that should be deployed to the system.
  In the example above, that would be the `shell`, `git`, and `upower` folders.

Groups are **enabled per host** by adding them to the `groups` list in the [host.yml](./host.md#hostyml).

## `group.yml`

The following is a full example of a `group.yml`:

```yml
# Override the target directory for all files in this group.
# If not set, the global target directory is used.
target_directory: /etc

# Packages that should be installed when this group is enabled.
packages:
  pacman:
    - git
    - vim
    - neovim

# Defaults that should be applied to all files in this group.
defaults:
  owner: root
  group: root
  file_mode: 0o644
  directory_mode: 0o755
```

- `target_directory`: `PathBuf` (optional) - Override the target directory for all configuration files in this group.
  - If it's a relative path, it's treated as relative to the global target directory.
  - If it's an absolute path, that absolute path is used.
- `packages`: `Map<String -> List<String>>` (optional) - A list of packages sorted by package manager.
  Look at [Package Management](../system_configuration/package_management/package_management.md) to see the list of available package managers.
- `defaults`: (optional) Set default file permissions for all configuration files that're inside this group directory.
  - `owner`: `String` - The file's owner
  - `group`: `String` - The file's assigned group
  - `file_mode`: `OctalInt` - The default permissions that'll be set for all files.
  - `directory_mode`: `OctalInt` - The default permissions that'll be set for all directories.
