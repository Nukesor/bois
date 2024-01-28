# Design

The core idea of _Bois_ is to provide a convenient **minimalistic** configuration file manager for your **personal** machines.
Some core components of the system are to some degree managed by Bois, such as system services and installed packages.
To some degree in this context means that the system's state isn't fully managed, but only as far as it concerns configuration files.

Bois is **not** intended to be used as a provisioning service for remote machines, but for machines you're living on. It's also not intended to cover complex setups with multiple-dependency setups

## Tasks

- Configuration management
  - Hooks
  - Package management
  - systemd service management
- Diffs between system-, last-deployed- and config directory state.
- Changeset detection (Terraform style)

## Core concepts

- Can be tracked via Git
- Multiple system configs in a single repository
- Shared/Base rules (will be applied to all/some systems?)
- Simple Templating (Tera?)
- Unix only

## Configuration

### Host/Group configuration

- Multiple top-level directories represent groups
- Entry point groups
  - Entry point groups are named as the hostname of the respective machine
  - Defines other groups as dependencies
  - May have global variable files
  - May have local variable files
- Normal groups
  - Can **only** have local variable files

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

## Deployment process

All state is loaded in the local `State` struct, which
The state is saved to a temporary directory.

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
