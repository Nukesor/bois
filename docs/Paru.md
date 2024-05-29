# AUR Packages

Installing AUR packages is a bit tricky, as `root` isn't allowed to build packages in general.

The current way to work around this is to create a dedicated user, which will run `paru` for root.
It needs to be able to call `pacman` though, so there's a bit of setup that needs to be done.

1. Create the user.
   ```sh
   useradd --home-dir /var/lib/aur --create-home aur
   ```
2. Allow `aur` to install call pacman to actually install packages and/or dependencies.
   ```
   aur ALL=(ALL) NOPASSWD: /usr/bin/pacman
   ```
3. TODO: Make the user configurable.
