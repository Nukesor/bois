# Hosts

Hosts are an important concept in `bois`.
Since `bois` is designed for your personal computers, your machines are configured on a `hostname` basis.

The configuration files for your machines are located in the `host` directory. \
Imagine having two machines named `strelok` and `artifact` (which are also their respective `hostname`s).
The directory structure might look something like this:

```
 📁 groups/
 📂 hosts/
 │ 📂 artifact/
 │ │ 📁 udev/
 │ │ 📁 X11/
 │ │ pacman.conf
 │ │ host.yml
 │ └ vars.yml
 └ 📂 strelok/
   │ host.yml
   └ vars.yml
```

- The `host.yml` file is required to exist in every host directory.
  It allows you to set host-specific configuration defaults and determines which groups are going to be included for this host.
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
  In the example above, that would be the `X11` and `udev` folders, as well as the `pacman.conf` for the `artifact` host.

Let's anticipate the next chapter a tiny bit, which will be about [groups](./groups). Groups are a tool to allow reuse of configuration files across multiple hosts.

In contrast to [groups](./groups), host configuration files are always **exclusive** for a specific host.
This allows you have a strict distinction between reusable logic, which is kept inside of groups, and machine specific configuration, which is located the machine's respective host directory.

## `host.yml`

The following is a full example of a `host.yml`:

```yml
# Groups that're required by this host.
groups:
  - base
  - laptop
  - games

# Packages that should always be installed for this host.
packages:
  pacman:
    - linux
    - base-devel
    - tuned

# Defaults that should be applied to all files.
file_defaults:
  owner: root
  group: root
  file_mode: 0o644
  directory_mode: 0o755
```

- `groups`: `List<String>` The list of groups that're enabled for this host.
  The group names correspond to the group's directory names inside the top-level `groups` directory.
- `packages`: `Map<String -> List<String>>`: A list of packages sorted by package manager.
  Look at [Package Management](../system_configuration/package_management/package_management.md) to see the list of available package managers.
- `file_defaults` Set defaults file permissions for all configuration files that're inside this host directory.
  - `owner`: `String` - The file's owner
  - `group`: `String` - The file's assigned group
  - `file_mode`: `OctalInt` - The default permissions that'll be set for all files.
  - `directory_mode`: `OctalInt` - The default permissions that'll be set for all directories.
