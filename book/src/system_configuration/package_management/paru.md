# Paru

## Setting up `paru`

Installing AUR packages with `paru` is a bit tricky, as `root` isn't allowed to build packages.

The current way to work around this is to create a dedicated user, which will run `paru` for root.
It needs to be able to call `pacman` though, so there's a bit of setup that needs to be done.

At this point of this writing, `bois` still expects this user to be named `aur`.

1. Create an `aur` user.
   ```sh
   useradd --home-dir /var/lib/aur --create-home aur
   ```
2. Allow `aur` to call pacman as with `root` permissions to install packages.
   ```
   aur ALL=(ALL) NOPASSWD: /usr/bin/pacman
   ```

## Configuration

Packages can be added by adding a `packages.paru` section to either a trait config or the host config.
For example:

```yaml
# Packages that should be installed when this trait is enabled.
packages:
  paru:
    - pueue-git
```

All paru packages that're defined in the host config and the configs of all enabled traits will then be installed for the given host.
