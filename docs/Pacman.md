# Pacman

## Tips and tricks

### `bois diff`

If you encounter packages that're listed as explicitly installed, but want them be handled as a dependency so they no longer show up in the diff, there's a simple command for that:

```sh
sudo pacman -D --asdep $package_name
```

This command marks that package as a dependency and it'll no longer show up in the diff.
