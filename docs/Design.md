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
- Entry/Host point groups
  - Entry point groups are named as the hostname of the respective machine
  - Defines other groups as dependencies
  - May have global variable files
  - May have local variable files
  - Must have a `bois.yml`
- Normal groups
  - May have a `bois.yml`
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

### Merging

The idea for this configuration structure is, so defaults can be set at various levels (host, group, directory), which are then active for the respective space.
This is, until a "lower" configuration overwrites that default.

The hierarchy looks like this:

```txt
host < group < directory < subdirectory < file
```

I.e. defaults on a host level are overwritten by all other more specific configurations.

## Deployment process

All state is loaded in the local `State` struct, which
The state is saved to a temporary directory.

## Datastructures

### Folders

Example folder structure for a computer named `HOSTNAME_1`.

```txt
bois
|-- bois.yml
|-- base
|   |-- group.yml
|   |-- pacman.conf
|
|-- HOSTNAME_1
|   |-- host.yml
|   |-- modprobe.d
|       |-- nobeep.conf
|
|-- HOSTNAME_2
|   |-- host.yml
|   |-- systemd
|       |-- network
|           |-- 10-ethernet.network

|-- .deployed
|   |-- etc
|       |-- pacman.conf
|       |-- modprobe.d
|           |-- nobeep.conf
```

### Data load order

- At the very first, the group that's named like the current host is loaded.
  This group then further specifies other groups that should be loaded.

## Deployment

The deployment process is rather simple and can be devided into clear-cut steps.

1. Check for current deployment \
   If there exists a previous deployment, the actual deployed files are compared with the last known deployed state.
   This step detects any changes on files that weren't handled by Bois. \
   The user can then be warned that those changes might get overwritten on a new deploy.
1. Read configuration and template files. \
   In this step, all relevant files from the bois configuration directory are read and internally compiled into one large state struct.
1. Compare the a possible previously deployed state, the actual system state and the state to-be deployed.
   Based on this, a deterministic sequential changeset is created that consists of concrete executable steps to reach the desired system state.
1. Execute all steps of the changeset to the system.
   TODO: How do we handle error cases? What should be done during an error?
         How do we recover from this?
1. Save the serialized state to disk, so we can compare the current state during the next deployment.

### Order

The order in which files are deployed doesn't need to be super-configurable, but it should be deterministic.
How does one handle conflicts? Silent overwriting based on priority? Or hard conflict error with good error message?

For this to work, Bois follows the following ordering :

- `priority`: Configurable on a group, folder and file basis. Higher priority means earlier deployment/execution.
- Recursively by **target** Folder/File names, just like `ls -R` is working.
  ```txt
  /etc/alsa/conf.d/10-samplerate.conf
  /etc/alsa/conf.d/50-arcam-av-ctl.conf
  /etc/thermald/thermal-cpu-cdev-order.xml
  /etc/tlp.d/00-template.conf
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

### TODOS

#### Error handling

Introduce good error handling.
  The idea would be to have two different error handling types.
  1. Errors that happen during the preparation phase. This would include things like:
    - Conflicts
    - Changes that have been detected on the system and aren't yet incorporated.
    - Config errors (wrong enum variants), etc.
  2. Errors that happen during execution. These errors should result in the program exiting.
    - These errors need to be very descriptive.
    - They must clearly state at which operation the problem occured.

Determine a good way of handling errors from other binaries, that're being called.
E.g. pacman that has a network error.

#### Execution order

The order in which things are executed should be clearly defined.

Global execution order:
- At first, all removals should be executed.
  Removals should be executed in the order of dependencies, with the host group being the first one.
- Changes and additions are executed afterwards
  They should also be executed in the order of dependencies, with the host group being the first one.

Execution order of removals **inside** of groups/directories with the **same priority**.
- Files/directories are executed in alphabetic order.
  - Disable/stop services.
  - Remove configuration files
  - Uninstall packages

Execution order **inside** of groups/directories with the **same priority**.
- Files/directories are executed in alphabetic order.
  - Install packages
  - Add configuration files
  - Start/enable services

Keeping this order is important, as configuration files may depend on directories being created during package installation.
Services may depend on configuration files.
