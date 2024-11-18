# Paru

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
