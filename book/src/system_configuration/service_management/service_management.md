# Service Management

Bois contains support for managing system services.

This allows you to enable services based on traits, folders or per host.

The rules are:

- Services are **enabled**, but not started, unless a service explicitly requests an immediate start via the `start` flag.
- Once a service leaves the configuration, it's **stopped and disabled** during the cleanup phase of the next deployment.
- Bois doesn't manage the _running_ state of services. A service that's already enabled but was stopped manually won't be started again.
- The [mode](../../guide/bois_config.md#modes) bois runs in determines which service manager instance is targeted: user mode manages the user's own instance (e.g. `systemctl --user`), system mode the system's.

Services can be declared in a host config, a trait config or a folder's `dir.yml`:

```yaml
services:
  systemd:
    - systemd-timesyncd
    - backup.timer
```

Supported service managers:

- [Systemd](./systemd.md)
