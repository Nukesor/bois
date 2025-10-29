# Bois Config

The top-level `bois.yml` file configures global settings for `bois`.
This file is optional - if it doesn't exist, `bois` will use sensible defaults.

## Location

Bois looks at the following locatin (in this order) for a `bois.yml`:

In **User** Mode:

- `~/.config/dotfiles/bois.yml`
- `~/.config/bois/bois.yml`
- `~/.dotfiles/bois.yml`
- `~/.dots/bois.yml`
- `~/.bois/bois.yml`

In **System** Mode:

- `/etc/bois/bois.yml`

You can also specify a custom config file location using the `--config` flag.

## Example

Here's a full example of a `bois.yml`:

```yml
# The machine name used to select the host directory.
# If not set, the system hostname is used.
name: my-laptop

# The directory containing your bois configuration (hosts, groups, etc).
# Defaults to the directory where this bois.yml is located.
bois_dir: ~/dotfiles

# The target directory where configuration files are deployed.
# User mode: ~/.config (default)
# System mode: /etc/bois (default)
target_dir: ~/.config

# Cache directory for storing deployment state.
# User mode: ~/.cache/bois (default)
# System mode: /var/lib/bois (default)
cache_dir: ~/.cache/bois

# Runtime directory for temporary files.
# User mode: ~/run/user/$YOUR_USER_ID/bois (default)
# System mode: /var/lib/bois (default)
runtime_dir: /run/user/1000/bois

# Additional environment variables for password managers or other integrations.
envs:
  PASSWORD_STORE_DIR: ~/.password-store
  GOPASS_SESSION: some-session-token

# Operating mode: User or System
# User mode deploys to user directories (~/.config)
# System mode deploys to system directories (/etc)
# Defaults to System when running as root, User otherwise.
mode: User
```

## Configuration Options

All fields are optional:

- `name`: `String` - The machine name, used to select which host directory to use.
  Defaults to the system hostname.
- `bois_dir`: `PathBuf` - The directory containing your bois configuration (hosts, groups, etc).
  - By default, it picks the first directory it finds at the following locations:
    - `~/.config/dotfiles`
    - `~/.config/bois`
    - `~/.dotfiles`
    - `~/.dots`
    - `~/.bois`
  - System mode default:
    - `/etc/bois`
- `target_dir`: `PathBuf` - The target directory where configuration files are deployed.
  - User mode defaults:
    - `$XDG_CONFIG_DIR/`
    - `~/.config` (fallback)
  - System mode default:
    - `/etc/bois`
- `cache_dir`: `PathBuf` - Cache directory for storing deployment state.
  - User mode default:
    - `XDG_CACHE_DIR/bois`
    - `~/.cache/bois`
  - System mode default:
    - `/var/lib/bois`
- `runtime_dir`: `PathBuf` - Runtime directory for temporary files.
  - User mode defaults:
    - `$XDG_RUNTIME_DIR/bois`
    - `~/.cache/bois` (fallback)
  - System mode default:
    - `/var/lib/bois`
- `envs`: `Map<String -> String>`
  This can be used to set additional environment variables that should be loaded into bois environment.
  That's useful for password manager integration which often requires special configuration or session variables.
- `mode`: `User | System` The mode of operatation. By default, this is detected based on the current user: `root` users run in `System` mode while non-root users run in `User` mode.
  - `User`: Deploy to user directories and perform actions as user, such as running `systemctl` with `--user` flag
  - `System`: Deploy to system directories and perform actions as root, such as installing packages as root or running `systemctl` as root.

## Modes

Bois operates in two modes that determine default directories and behavior:

### User Mode
- Target directory: `~/.config`
- Cache directory: `~/.cache/bois`
- Runtime directory: `$XDG_RUNTIME_DIR/bois`
- Systemctl: Called with `--user` flag
- Use case: Managing personal dotfiles

### System Mode
- Target directory: `/etc/bois`
- Cache directory: `/var/lib/bois`
- Runtime directory: `/var/lib/bois`
- Systemctl: Called without `--user` flag
- Use case: Managing system-wide configuration (requires root)
