# Hosts

Hosts are an important concept in `bois`.
Since `bois` is designed for your personal computers, hosts are configured on a `hostname` basis.

The configuration files for your hosts are located in the `hosts` directory. \
Imagine having two hosts named `ghost` and `fang` (which are also their respective `hostname`s).
The directory structure might look something like this:

```
 📁 traits/
 📂 hosts/
 │ 📂 fang/
 │ │ 📁 udev/
 │ │ 📁 X11/
 │ │ pacman.conf
 │ │ host.yml
 │ └ vars.yml
 └ 📂 ghost/
   │ host.yml
   └ vars.yml
```

- The `host.yml` file is required to exist in every host directory.
  It allows you to set host-specific configuration defaults and determines which traits are going to be included for this host.
- All variables inside the `vars.yml` are exposed to the templating engine.
  Read the [templating docs](./templating.md) for detailed info.
  The top level of the `vars.yml` is expected to be an object.
  I.e.
  ```yml
  encrypt: false
  machine:
    threads: 8
    is_laptop: true
  ```
- All other files that're located in a host's directory are considered configuration files that should be deployed to the system.
  In the example above, that would be the `X11` and `udev` folders, as well as the `pacman.conf` for the host `fang` .

Let's look ahead to the next chapter real quick, which will be about [traits](./trait_config.md). Traits are a tool to allow reuse of configuration files across multiple hosts.

In contrast to [traits](./trait_config.md), host configuration files are always **exclusive** for a specific host.
This allows you have a strict distinction between reusable logic, which is kept inside of traits, and host specific configuration, which is located the host's respective directory.

## `host.yml`

The following is a full example of a `host.yml`:

```yml
# Traits that're required by this host.
traits:
  - base
  - laptop
  - games

# Packages that should always be installed for this host.
packages:
  pacman:
    - linux
    - base-devel
    - tuned

# Services that should be enabled for this host.
services:
  systemd:
    - systemd-timesyncd
    - backup.timer

# Defaults that should be applied to all files.
file_defaults:
  owner: root
  group: root
  file_mode: 0o644
  directory_mode: 0o755

# Controls what should be cleaned up once it's removed from this host's configuration.
cleanup:
  directories: true
```

- `traits`: `List<String>` The list of traits that're enabled for this host.
  The trait names correspond to the trait's directory names inside the top-level `traits` directory.
- `target_dir`: `PathBuf` (optional) - Override the target directory for all configuration files in this host directory.
  Must be an absolute path (`~` is expanded). If not set, the global target directory is used.
- `packages`: `Map<String -> List<String>>`: A list of packages sorted by package manager.
  Look at [Package Management](../system_configuration/package_management/package_management.md) to see the list of available package managers.
- `services`: (optional) - A list of services sorted by service manager.
  Listed services are enabled during deployment. Once removed, they're stopped and disabled.
  A service can either be a plain name, or an object with a `name` and a `start` flag to also start the service right away when it gets enabled.
  Look at [Service Management](../system_configuration/service_management/service_management.md) to see the list of available service managers.
- `file_defaults` Set defaults file permissions for all configuration files that're inside this host directory.
  - `owner`: `String` - The file's owner
  - `group`: `String` - The file's assigned group
  - `file_mode`: `OctalInt` - The default permissions that'll be set for all files.
  - `directory_mode`: `OctalInt` - The default permissions that'll be set for all directories.
- `cleanup`: (optional) Controls what should be cleaned up once it's removed from this host's configuration.
  Files and packages are always cleaned up; this only covers resources where cleanup is opt-in.
  - `directories`: `Boolean` - Whether directories are removed once they leave the configuration. Defaults to `false`.
    Even if set to `true`, directories are only removed if they're empty.
    If a directory still contains unmanaged files, it's never removed.
    This setting applies to the whole host directory and can be overridden per subtree via a folder's [dir.yml](./folder_config.md).
