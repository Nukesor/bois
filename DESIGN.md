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
- Simple Templating (Tera?)
- Unix only

## Configuration

### Host/Group configuration

- Specific named files by group/hostname
- Variables for group/hostname in that file
- Packages?

### File/Directory Configuration

- In file configuration
  - Permissions
  - Ownership
  - Location
- In folder configuration
  - Folder permissions
  - File default permissions
  - Ownership
  - Location
- Some form of yaml/toml for variables

## Datastructures

### Folders

Example folder structure for a computer named `HOSTNAME_1`.

```txt
bois
|-- base.yml
|-- HOSTNAME_1.yml
|-- base
|   |-- pacman.conf
|
|-- HOSTNAME_1
|   |-- modprobe.d
|       |-- nobeep.conf
|
|-- .deployed
|   |-- etc
|       |-- modprobe.d
|           |-- nobeep.conf
|       |-- pacman.conf
```

## Features

- Subcommands
  - `diff` Compare the currently deployed config vs. the config that's in the repository.
    - `--apply` Optionally apply changes on the system on a chunk-by-chunk basis to the repository?
  - `deploy` Deploy all changes. Prompt the user for permission with a current file diff.
    - `--force` Don't prompt the user with diff for input.
- Simple addition of existing files in system via subcommand.
- Automatic target detection via hostname
  - Simple migration to new PC via directory name change
- "State management"
  - Save the current deployed state.
    Needed for diff and similar

### Config

```yaml
pre_hooks:
    - update_packages
post_hooks:
    - 'systemctl enable --now some.service'
variables:
    some_variable_1
```
