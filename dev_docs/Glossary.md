# Glossary

This document describes the terminology used throughout the bois codebase.

## Configuration sources

- **bois directory**: The root directory that holds the entire configuration, i.e. `bois.yml`, `hosts/`, and `traits/`.
- **run mode**: Whether bois manages a user's dotfiles (`user`) or provisions a system (`system`).
- **host**: A machine that's managed by bois.
- **host config**: A single machine's configuration. Its required config file lives at
  `<bois directory>/hosts/<name>/<name>.yml`, or at `<bois directory>/hosts/<name>.yml` for a host without a directory.
- **trait**: A reusable, self-contained configuration unit (e.g. `desktop`, `audio`). Its optional config file lives at
  `<bois directory>/traits/<name>/<name>.yml`, or at `<bois directory>/traits/<name>.yml` for a trait without a directory.
  Hosts opt into traits.
- **source directory**: The directory of a single host or trait, i.e. where its files are read from.
  Also referred to as "host directory" and "trait directory".
- **source file**: A raw file as read from a source directory, before templating, with its config block still attached.
- **origin**: The host or trait a tree node came from.
- **file config**: A `bois_config` block embedded at the top of a source file.
- **target directory**: The directory files are deployed into by default (`~/.config` in user mode, `/etc` in system mode).
- **target path**: The absolute path on the system a node will be deployed to.

## States

During the comparison phase three different types of state are compared:

- **desired** state: What the system should look like according to the configuration.
- **previous** state: What was deployed on the system during the last successful deploy, persisted as `deployed_state.yml` in the cache directory.
- **actual** state: What is on the system right now, including:
  - The filesystem
  - Installed packages
  - Dis-/Enabled services

## The path tree

- **tree**: The fully resolved representation of all files and directories to deploy.
- **node**: One item in the tree, either a file or a directory.
- **entry**: Used to describe an path entry on the filesystem.
  This term has been introduced so we can differentiate between files in the bois directory (nodes) and actual entries on the system.
- **actual entry**: An entry that is found at a path on the filesystem **right now**.

### Directory types

Handling directories requires a bit of extra care, as we have to decide when to manage permissions and/or allow symlinks.

- **declared permissions**: At least one of mode/owner/group was set for a directory, via `dir.yml` or the defaults cascade.
- **implicit directory**: A tree directory that only exists because it is a parent component of some node's target path.
  For example `/etc` for a file targeting `/etc/udev/rules.d/foo` and `/etc` being the host's `default_target`.
  This is the "weakest" kind of directory and as such never cleaned up or having its permission managed.
- **backed directory**: A tree directory that corresponds to a real directory in a host or trait source tree, but not having any declared permissions.
  For example, `/etc/udev` and `/etc/udev/rules.d` for a file targeting `/etc/udev/rules.d/foo` with there being a trait that has the `default_target: /etc` and a config file at `udev/rules.d/foo`.
- **explicit directory**: A directory with declared permissions.
  Any directory that has any kind of explicit permission set, either on itself or via the host's/trait's `permission_defaults`.

## Comparison and changesets

- **changeset**: A set of operations to execute, including:
  - package installs/uninstalls
  - path operations
  - service enables/disables.
- **drift**: Changes made on the system since the last deploy that aren't reflected in the config.
- **deploy changeset**: The result of the comparison of "desired state vs. actual state".
  Effectively, what the set of changes that must be performed so the system matches the config for a host.
- **drift set**: The result of the comparison of "previous state vs. actual state".
  I.e. what the user changed since the last deploy.
- **cleanup changeset**: The result of the comparison of "previous state vs. desired state".
  Anything that was previously deployed by bois, but is no longer wanted and as such must be cleaned up.
- **adopted**: Drift on the system, which has already been included into the configuration of the bois directory
  For example, a user edited a deployed file and made the same edit in the bois directory.
