# Trait Config

Traits are a tool for reusing configuration files across multiple hosts.

All configuration that's shared between machines should be placed into traits.
For instance, all machines might share the same base packages, shell configuration, or editor setup.

Traits are located in the top-level `traits` directory.
The directory structure might look something like this:

```
 📂 traits/
 │ 📂 base/
 │ │ 📁 shell/
 │ │ 📁 git/
 │ └ trait.yml
 │ 📂 laptop/
 │ │ 📁 upower/
 │ └ trait.yml
 └ 📂 games/
   └ trait.yml
 📁 hosts/
```

- The `trait.yml` file is optional.
  It allows you to set trait-specific configuration and specify packages that should be installed when this trait is included.
- All other files that're located in a trait's directory are considered configuration files that should be deployed to the system.
  In the example above, that would be the `shell`, `git`, and `upower` folders.

Templating variables are defined in the host's `vars.yml` and are available in all traits as well.
Read the [templating docs](./templating.md) for detailed info.

Traits are **enabled per host** by adding them to the `traits` list in the [host.yml](./host_config.md#hostyml).

## `trait.yml`

The following is a full example of a `trait.yml`:

```yml
# Override the target directory for all files in this trait.
# If not set, the global target directory is used.
target_directory: /etc

# Packages that should be installed when this trait is enabled.
packages:
  pacman:
    - git
    - vim
    - neovim

# Defaults that should be applied to all files in this trait.
defaults:
  owner: root
  group: root
  file_mode: 0o644
  directory_mode: 0o755

# Controls what should be cleaned up once it's removed from this trait's configuration.
cleanup:
  directories: true
```

- `target_directory`: `PathBuf` (optional) - Override the target directory for all configuration files in this trait.
  Must be an absolute path (`~` is expanded). If not set, the global target directory is used.
- `packages`: `Map<String -> List<String>>` (optional) - A list of packages sorted by package manager.
  Look at [Package Management](../system_configuration/package_management/package_management.md) to see the list of available package managers.
- `defaults`: (optional) Set default file permissions for all configuration files that're inside this trait directory.
  - `owner`: `String` - The file's owner
  - `group`: `String` - The file's assigned group
  - `file_mode`: `OctalInt` - The default permissions that'll be set for all files.
  - `directory_mode`: `OctalInt` - The default permissions that'll be set for all directories.
- `cleanup`: (optional) Controls what should be cleaned up once it's removed from this trait's configuration.
  Files and packages are always cleaned up; this only covers resources where cleanup is opt-in.
  - `directories`: `Boolean` - Whether directories are removed once they leave the configuration. Defaults to `false`.
    Even if set to `true`, directories are only removed if they're empty.
    If a directory still contains unmanaged files, it's never removed.
    This setting applies to the whole trait directory and can be overridden per subtree via a folder's [dir.yml](./folder_config.md).
