## Bois

Bois is an opinionated system provisioning tool for your **personal** machines. \
I call my own machines jokingly my `bois`, hence the name.

It enables you to manage configuration files while being straight forward to use, making it easy to share them with other devices.
This means re-usability of your configuration via templating and optional deployment on a per-host basis.

This effectively means that `bois` can be used for both system configuration (as `root`) and as a manager for your dotfiles.
On top of handling system config files, it's also able to manage your installed packages and enabled services.

You could say that it aims to strike a balance between Chezmoi and Ansible/Saltstack, but on-host and for **your** bois.

## Short Overview of the Most Prominent Features

- System configuration file management
  - Allow editing of deployed files
  - Diffing/Merging of deployed files vs. changed files in bois directory.
  - Safety first. Don't overwrite changes without a prompt.
- System package management (via package managers)
- System service management (e.g. Systemd).
  - Dis-/enable services based on deployed files.
- Cleanup
  - Remove deployed files/directories if removed from bois.
  - Uninstall packages if removed from bois.
  - Disable services if removed from bois.
- Also designed for usage as user dotfile manager.
