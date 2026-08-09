# Design

The core idea of _Bois_ is to provide a convenient **minimalistic** provisioning system for your _personal_ machines.
As such, some components of the system are managed by Bois to a certain degree, such as system services, installed packages and configuration files.

To some degree in this context means that the system's state isn't necessarily fully managed, but only as much as the user decides.

Bois is **not** intended to be used as a provisioning service for remote machines, but for machines that you're using.
It's also not intended to cover complex multiple-dependency deployments and the likes.

## Tasks

- Configuration management
  - Hooks (TDB)
- Package management
- Systemd service management

## Core concepts

- Unix only
- Can be tracked via Git
- Multiple host configs in a single repository
- Trait system (groups of tasks/configs hosts can opt into)
- Simple jinja-style templating
- Diffs between various states:
  - `current config -> current system`: Changes that must be made to reach the desired state.
  - `last-deployed -> current config`: Previously deployed changes that must now be cleaned up.
  - `last-deployed -> current system`: On-system changes that may be overwritten

## Configuration

### Host/Trait configuration

- Two directories for holding configurations:
  - `hosts/$hostname`: Config that **only** exists for a host with `$hostname`
  - `traits/$name`: Each host can opt-in to all configuration inside that trait.
- Hosts
  - Must be located in `hosts/$hostname`
  - Must have a `hosts/$hostname/host.yml` to configure that host.
    - Opt-in to a list of traits
  - May have **global** variable file `hosts/$hostname/vars.yml`
    Global means, that these variables are available for templating in all traits that're being executed.
- Normal traits
  - Must be located in `traits/$name`
  - May have a `traits/$name/trait.yml`

### File/Directory Configuration

- Located directly inside a trait (`traits/$name/`) or host directory (`hosts/$hostname/`)
- Is, by default, deployed to the configuration root specified in the global `bois.yml` or the respective host/trait config override.
  For example, with the target folder of `~/.config`, a `hosts/$hostname/nvim/` folder and all its contents is deployed to `~/.config/nvim/`.
- Configuration files may contain an in-file configuration block, which allows to configure:
  - Permissions
  - Ownership
  - Location
  - Enable Templating
  - Configure templating (e.g. delimiters)
- Configuration folder may contain a `dir.yml`, e.g. `hosts/$hostname/ssh/`, which allows to configure:
  - Location
  - Ownership
  - Folder permissions
  - Cleanup behavior for the directory's subtree.

### Configuration aggregation/merging

The idea for this configuration structure is, so defaults can be set at various levels (host, trait, directory), which are then active for the respective space.
This is, until a "deeper" configuration overwrites that default.

The hierarchy looks like this:

```txt
host < trait < directory < subdirectory < file
```

I.e. defaults on a host level are overwritten by all other more specific configurations.

## Deployment process

Internally, a representation of the managed system state is put into a single `State` struct
The currently deployed `State` struct is then serialized and saved to `deployed_state.yml` in the cache directory (`~/.cache/bois` in user mode, `/var/lib/bois` in system mode).
This state then allows us to track on-system changes and do cleanup between two deploys.

## Datastructures

### Folders

Example folder structure for a computer named `HOSTNAME_1`.

```txt
bois
|-- bois.yml
|-- traits
|   |- base
|      |-- trait.yml
|      |-- pacman.conf
|
|-- hosts
|   |- HOSTNAME_1
|   |  |-- host.yml
|   |  |-- modprobe.d
|   |      |-- nobeep.conf
|   |
|   |- HOSTNAME_2
|      |-- host.yml
|      |-- systemd
|          |-- network
|              |-- 10-ethernet.network
```

### Data load order

- At the very first, the config of the current host is loaded.
- That config then further specifies all traits that should be loaded.
- All trait configs are loaded in the order they're specified.

## Deployment

The deployment process is rather simple and can be devided into clear-cut steps.

1. Read configuration and template files. \
   In this step, all relevant files from the bois configuration directory are read and internally compiled into one large state struct.
1. Check the current deployment. \
   If there exists a previous deployment, the current system files are compared with the last known deployed state.
   This step detects any on-system changes of state that's managed by bois. \
   The user is then warned that those changes will get overwritten on the new deploy and asked for confirmation.
1. Compare a possible previously deployed state with the state to-be deployed.
   This results in a cleanup changeset that removes any managed state that has been removed from the bois config.
   After the cleanup has been executed, an intermediate state is persisted, so an aborted deploy doesn't blame the intentional cleanup on the user during the next run.
1. Compare the state to-be deployed with the actual system state.
   Based on this, a deterministic sequential changeset is created that consists of concrete executable steps to reach the desired system state.
1. Execute all steps of the changeset to the system.
   TODO: How do we handle error cases? What should be done during an error?
   How do we recover from this?
1. Save the serialized state to disk, so we can compare the current state during the next deployment.

### Order

The order in which files are deployed doesn't need to be super-configurable, but it should be deterministic.
Conflicting definitions for the same target path result in a hard conflict error with a good error message.

For this to work, Bois follows the following ordering:

- Recursively by **target** Folder/File names, just like `ls -R` is working.
  ```txt
  /etc/alsa/conf.d/10-samplerate.conf
  /etc/alsa/conf.d/50-arcam-av-ctl.conf
  /etc/thermald/thermal-cpu-cdev-order.xml
  /etc/tlp.d/00-template.conf
  ```

## Features

- Subcommands
  - `plan` Dry-run that shows all changes that would be executed on the system.
  - `deploy` Deploy all changes. Prompt the user for permission if untracked on-system changes would be overwritten.
  - `diff` Compare the current system against the target state. (currently packages only)
  - `absorb` Integrate on-system changes since the last deployment back into the configuration. (not yet implemented)
  - `init` Setup a new bois directory.
- Automatic target detection via hostname
  - Simple migration to new PC via directory name change
- "State management"
  - Save the current deployed state.
    Needed for diff and similar

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
  Removals should be executed in the order of dependencies, with the host trait being the first one.
- Changes and additions are executed afterwards
  They should also be executed in the order of dependencies, with the host trait being the first one.

Execution order of removals **inside** of traits/directories with the **same priority**.

- Files/directories are executed in alphabetic order.
  - Disable/stop services.
  - Remove configuration files
  - Uninstall packages

Execution order **inside** of traits/directories with the **same priority**.

- Files/directories are executed in alphabetic order.
  - Install packages
  - Add configuration files
  - Start/enable services

Keeping this order is important, as configuration files may depend on directories being created during package installation.
Services may depend on configuration files.

### Diffing during deployment

There're different scenarios that need different diffs and handling.
Let's start with the most simple one, a clean deployment.

#### First run

1. The configuration is read.
1. The "should-be" state is compared with the current state of the system.
1. A changeset is created that transforms the current state into the desired state.
1. Save the current "should-be" state in serialized form to disk.

This is rather straight forward.

#### Successive runs

It now starts to become a bit more tricky, as we also need to do **cleanup** and we want to detect any untracked changes by the user or programs on the system.

1. Read the configuration and determine the "should-be" state.
1. Compare the **previous** "should-be" state with the current system state.
   This shows us any changes that were made to the system since the last deployment.
   We want to inform the user about these changes and give them a chance to incorporate them before they're overwritten by the next deployment.
1. Compare the **previous** "should-be" state with the **current** "should-be" state.
   This allows us to see whether there're any:
   - Files, directories that need removal
   - Services that need to be stopped
   - Packages to be uninstalled.
     This will result in a "cleanup" changeset that will be executed before the new deployment runs.
1. At this point, we're done with the complex logic and we continue as if we do a first-time deployment.
1. The "should-be" state is compared with the current state of the system.
1. A changeset is created that transforms the current state into the desired state.
1. Save the current "should-be" state in serialized form to disk.
