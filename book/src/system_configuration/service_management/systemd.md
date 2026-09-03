# Systemd

## Configuration

Services can be added by adding a `services.systemd` section to a host config, a trait config or a folder's `dir.yml`.
For example:

```yaml
# Services that should be enabled when this trait is enabled.
services:
  systemd:
    - systemd-timesyncd
    - backup.timer
```

All systemd services that're defined in the host config, in the configs of enabled traits and in `dir.yml` files will then be enabled for the given host.

Unit names without an explicit unit suffix (such as `.service` or `.timer`) refer to `.service` units, just like they do when calling `systemctl`.
I.e. `systemd-timesyncd` and `systemd-timesyncd.service` are treated as the same unit.

## User and System Mode

Depending on the current [mode](../../guide/bois_config.md#modes) bois runs in, different systemd units are targeted:

- In `System` mode, the system's systemd instance is managed.
- In `User` mode, all `systemctl` calls are made with the `--user` flag, so the user's own systemd instance is managed instead.
  Unit files for user services live in directories such as `~/.config/systemd/user/`.

## Deployment

Services are enabled at the very end of a deployment, after packages have been installed and files have been deployed.
That way, unit files that're installed by packages or deployed by bois itself already exist by the time they're enabled.

Services are only **enabled** (e.g. `systemctl enable myunit`), not _started_.
If a service should be started right away at the moment it gets enabled, use the explicit declaration syntax:

```yaml
services:
  systemd:
    - name: docker
      start: true
```

This enables the unit via `systemctl enable --now`.

Note that the `start` flag only takes effect at the moment the service gets enabled.
Bois doesn't manage the _running_ state of services.
This means that a service that's already enabled but was stopped manually won't be started again on the next deployment.

## Cleanup

Once a service is removed from the configuration, it's **stopped and disabled** (`systemctl disable --now`) during the cleanup phase of the next deployment.

Services are cleaned up before files and packages are removed, while the service's unit files still exist on the system.
