# Folder Config

Any folder inside a host or trait directory can have a `dir.yml` or `dir.yaml` file to configure how that folder and its contents should be deployed.

This is useful for:
- Overriding the destination path for a whole directory tree
- Setting ownership and permissions for all files in that directory
- Enabling services that belong to the configuration in that directory

## Example

Imagine you have a `udev` folder in your host directory that should be deployed to `/etc/udev/rules.d`:

```
 📂 hosts/
 └ 📂 ghost/
   └ 📂 udev/
     │ dir.yml
     │ 10-network.rules
     └ 20-usb.rules
```

The `dir.yml` might look like this:

```yml
# Deploy to an absolute path outside the default target directory
target_path: /etc/udev/rules.d

# Set ownership and permissions for the directory itself
owner: root
group: root
mode: 0o755

cleanup:
  # If this directory is removed from the config, it will be cleaned up.
  directories: true

# System services that should be enabled
services:
  systemd:
    - backup.timer
```

Now all files inside the `udev` folder will be deployed to `/etc/udev/rules.d`.

Note that `owner`, `group`, and `mode` only apply to the directory itself, not to the files inside it.
File ownership and permissions are set per file via its [file config](./file_config.md) block, via the host's/trait's defaults, or fall back to the deploying user.

## Configuration Options

- `target_path`: `PathBuf` (optional) - Override the destination path for this directory and all its contents.
  - If it's a relative path, it's treated as relative to the host's/trait's target directory
    (the `target_dir` override if set, otherwise the global target directory).
  - If it's an absolute path, that absolute path is used directly.
  - This override cascades to all child files and directories, unless they specify their own `target_path`.
- `owner`: `String` (optional) - The directory owner. Defaults to the current user.
- `group`: `String` (optional) - The directory's assigned group. Defaults to the current user's group.
- `mode`: `OctalInt` (optional) - The permissions for this directory (e.g., `0o755`). Defaults to `0o755`.
- `cleanup`: (optional) - Override the cleanup behavior for this directory and all its contents.
  - `directories`: `Boolean` - Whether directories are removed once they leave the configuration.
    Even if set to `true`, directories are only removed if they're empty.
    If a directory still contains unmanaged files, it's never removed.
    If not specified, the value is inherited from the parent directory (ultimately the host's/trait's `cleanup` setting).
- `services`: (optional) - A list of services sorted by service manager.
  This is identical to service sections on a trait or host level and allows service declarations close to the configuration files the services belong to.

## Path Inheritance

When a folder has a `target_path` override, all files and subdirectories inside inherit that override:

```
 📂 systemd/
 │ dir.yml (target_path: /etc/systemd/system)
 ├ 📂 timers/
 │ └ backup.timer
 └ 📁 services/
   └ backup.service
```

Both `timers/backup.timer` and `services/backup.service` will be deployed under `/etc/systemd/system/` unless they specify their own path override.

## Symlinks on the System

It's common to have symlinked directories on a system, such as `/lib/ -> /usr/lib/`.
Bois handles such symlinks in deployment paths as follows:

- If a directory in a deployment path is a symlink on the system and that directory has **no** declared permissions (`owner`, `group` or `mode`), the symlink is accepted, as long as it points to a directory.
  All files are then simply deployed through the link.
- As soon as any of `owner`, `group` or `mode` is declared for a directory, that directory is considered explicitly managed.
  Declared permissions can only be enforced on a real directory, so a symlink at its path is treated as a conflict: the link is deleted and replaced by a real directory.
- Symlinks that point to a file, dangling symlinks and link loops are always treated as conflicts and are replaced.
