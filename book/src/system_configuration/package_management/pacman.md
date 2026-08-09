# Pacman

## Configuration

Packages can be added by adding a `package.pacman` section to either a `trait.yml` or the `host.yml`.
For example:

```yaml
# Packages that should be installed when this trait is enabled.
packages:
  pacman:
    - git
```

All pacman packages that're defined in the `host.yml` and of all enabled `trait.yml` files will then be installed for the given host.

## Tips and tricks

### `bois diff`

If you encounter packages that're listed as explicitly installed, but want them be handled as a dependency so they no longer show up in the diff, there's a simple command for that:

```sh
sudo pacman -D --asdep $package_name
```

This command marks that package as a dependency and it'll no longer show up in the diff.
