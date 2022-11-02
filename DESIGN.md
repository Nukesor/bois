# Design

## Tasks

- Configuration management
  - Hooks?
- Package management?
- systemd service management?
  - Can be done via pre/post hooks?

## Core concepts

- Can be tracked via Git
- Multiple system configs in a single repository
- Shared/Base rules (will be applied to all/some systems?)
- Configuration follows same structure as real filesystem?
- Simple Templating (Tera?)
- Unix only

## Configuration

- In file configuration
  - Permissions
  - Ownership
- Some form of yaml/toml for variables

## Datastructures

### Folders

```txt
bois
|-- base.yml
|-- ${HOSTNAME_1}.yml
|-- base
|   |-- etc
|       |-- pacman.conf
|
|-- $HOSTNAME_1
|   |-- etc
|       |-- modprobe.d
|           |-- nobeep.conf
|-- .deployed
|   |-- etc
|       |-- modprobe.d
|           |-- nobeep.conf
|       |-- pacman.conf
```

## Features

- Subcommands
  - `add --base` Track an file on your system to your repository. By default, adds the file to your system specific folder.
  - `diff --apply` Compare the currently deployed config vs. the config that's in the repository.
    Optionally apply changes on the system on a chunk-by-chunk basis to the repository?
  - `deploy --force` Deploy all changes. Ask the user on critical changes (deletions) unless forced.
- Simple addition of files via subcommand
- Automatic target detection via hostname
  - Simple migration to new PC via directory name change
- Change detection on rendered files vs. existing files (`diff`)
- State management?
  - Save current deployed state.
  - Allows cleanup of configuration files that're no longer tracked.

### Config

```yaml
pre_hooks:
    - update_packages
post_hooks:
    - 'systemctl enable --now some.service'
variables:
    some_variable_1
```

### State

Either Json or some other more more compact state file.

```yaml
base:
    files:
        - name: /etc/pacman.conf
          hash: SHA256SUM of pacman.conf.
          origin: bois/.deployed/etc/pacman.conf
$HOSTNAME_1
    files:
        - name: /etc/pacman.conf
          hash: SHA256SUM of deployed file.
          origin: bois/.deployed/etc/pacman.conf
```
