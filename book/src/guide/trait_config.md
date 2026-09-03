# Trait Config

Traits are a tool for reusing configuration files across multiple hosts.

All configuration that's shared between hosts should be placed into traits.
For instance, all hosts might share the same base packages, shell configuration, or editor setup.

Traits are located in the top-level `traits` directory.
The directory structure might look something like this:

```
 📂 traits/
 │ 📂 base/
 │ │ 📁 shell/
 │ │ 📁 git/
 │ └ base.yml
 │ 📂 laptop/
 │ │ 📁 upower/
 │ └ laptop.yml
 └ games.yml
 📁 hosts/
```

- A trait's config file is optional. A trait with its own directory keeps the config file inside
  that directory at `traits/<traitname>/<traitname>.yml`, like `base` and `laptop` above.
  A trait can also consist of only a config file without a directory; it then lives directly
  at `traits/<traitname>.yml`, like `games` above.
  The config file allows you to set trait-specific configuration and specify packages that should be installed when this trait is included.
- All other files that're located in a trait's directory are considered configuration files that should be deployed to the system.
  In the example above, that would be the `shell`, `git`, and `upower` folders.

Templating variables are defined in the host's `vars.yml` and are available in all traits as well.
Read the [templating docs](./templating.md) for detailed info.

Traits are **enabled per host** by adding them to the `traits` list in the [host config](./host_config.md#the-host-config-file).

## The trait config file

The following is a full example of a trait config file:

```yml
# Override the target directory for all files in this trait.
# If not set, the global target directory is used.
target_dir: /etc

# Packages that should be installed when this trait is enabled.
packages:
  pacman:
    - git
    - vim
    - neovim

# Services that should be enabled when this trait is enabled.
services:
  systemd:
    - systemd-timesyncd
    - backup.timer

# Default permissions that should be applied to all files and directories in this trait.
permission_defaults:
  owner: root
  group: root
  file_mode: 0o644
  directory_mode: 0o755

# Controls what should be cleaned up once it's removed from this trait's configuration.
cleanup:
  directories: true
```

- `target_dir`: `PathBuf` (optional) - Override the target directory for all configuration files in this trait.
  Must be an absolute path (`~` is expanded). If not set, the global target directory is used.
- `packages`: `Map<String -> List<String>>` (optional) - A list of packages sorted by package manager.
  Look at [Package Management](../system_configuration/package_management/package_management.md) to see the list of available package managers.
- `services`: `Map<String -> List<String|Object>>` (optional) - A list of services sorted by service manager.
  Listed services are enabled during deployment. Once removed, they're stopped and disabled.
  A service can either be a plain name, or an object with a `name` and a `start` flag to also start the service right away when it gets enabled.
  Look at [Service Management](../system_configuration/service_management/service_management.md) to see the list of available service managers.
- `permission_defaults`: (optional) Set default permissions for all configuration files and directories that're inside this trait directory.
  Each field can be set on its own.
  - `owner`: `String` (optional) - The default owner for all files and directories.
  - `group`: `String` (optional) - The default group for all files and directories.
  - `file_mode`: `OctalInt` (optional) - The default permissions that'll be set for all files.
  - `directory_mode`: `OctalInt` (optional) - The default permissions that'll be set for all directories.
- `cleanup`: (optional) Controls what should be cleaned up once it's removed from this trait's configuration.
  Files and packages are always cleaned up; this only covers resources where cleanup is opt-in.
  - `directories`: `Boolean` - Whether directories are removed once they leave the configuration. Defaults to `false`.
    Even if set to `true`, directories are only removed if they're empty.
    If a directory still contains unmanaged files, it's never removed.
    This setting applies to the whole trait directory and can be overridden per subtree via a folder's [dir.yml](./folder_config.md).
