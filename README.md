# Bois

## What is Bois

Bois is an opinionated system provisioning tool for your **personal** machines, (which I lovingly call `my bois`).

It allows you to manage your system configuration files (mostly everything in `/etc`), so it lives in a git repository and can easily be re-used by your other devices.
This means re-usability of your system configuration via templating and optional deployment on a per-host basis.
On top of this, it also manages your installed packages and enabled services.

You could say that it aims to strike a balance between Chezmoi and Ansible/Saltstack, but on-host and for system configuration.

- [Features](https://github.com/Nukesor/bois#features)
- [Installation](https://github.com/Nukesor/bois#installation)
- [Design Goals](https://github.com/Nukesor/bois#design-goals)
- [Similar Projects](https://github.com/Nukesor/bois#similar-projects)
- [Contributing](https://github.com/Nukesor/bois#contributing)

## Features

- Configuration file management
  - Allow editing of deployed files
  - Diffing/Merging of deployed files vs. changed files in bois directory.
  - Safety first. Don't overwrite changes without a prompt.
- Granular system package management (via package managers)
- Granular system service management (e.g. Systemd).
  - Start/enable services based on deployed files.
- Cleanup
  - Remove deployed files/directories if removed from bois.
  - Uninstall packages if removed from bois.
  - Disable/stop services if removed from bois.
- Also designed for usage as user dotfile manager.

## Installation

There are a few different ways to install bois.

#### Prebuild Binaries

Statically linked (if possible) binaries for Linux (incl. ARM), Mac OS and Windows are built on each release. \
You can download the binary (`bois`) for each release on the [release page](https://github.com/Nukesor/bois/releases). \
Just download the binary for your system, rename it to `bois` and place it in your `$PATH`/programs folder.

#### Via Cargo

Bois is built for the current `stable` Rust version.
It might compile on older versions, but this isn't tested or officially supported.

```bash
cargo install --locked bois
```

This will install bois to `$CARGO_HOME/bin/bois` (default is `~/.cargo/bin/bois`)

#### From Source

Bois is built for the current `stable` Rust version.
It might compile on older versions, but this isn't tested or officially supported.

```bash
git clone git@github.com:Nukesor/bois
cd bois
cargo build --release --locked --path .
```

The final binaries will be located in `target/release/bois`.

## Design Goals

The main focus for bois is that it's supposed to be run on bare-metal **personal** machines, i.e. your desktop, laptop and maybe your home-server/NAS.
It's also supposed to be used from **inside** the system, in contrast to other tools, which provision systems from the outside (e.g. ssh or a orchestrator server).
It aims to strike a balance between Chezmoi and Ansible, but on-host and for systems configuration.

Additionally, there're a few "buzzwordy" design goals that need to be achieved:

- Idempotency - Deploying identical files will be always result in the same outcome.
- Insight - It must be easy to inspect any actions done by Bois, both beforehand and retrospectivelly.
- Convenience - The CLI UI must be convenient and intuitive to use.
  I.e. editing system files and deploying changes should work seamless and without too much of a merge/prompt hell.
- Clear semantics - Commands should be well named, the documentation should be concise and precise.
  There should be as little ambiguity as possible. E.g. avoid subcommands like `update` and `upgrade` that do entirely different things.
- Safety - In contrast to other provisioning tools, Bois is to be safe.
  E.g. changes since the last deploy are not to be overwritten without a prompt.
  The idea is to be rather a bit too verbose than sorry, at least by default.
- Opinionated - Bois isn't supposed to be a solution for everything and everyone.
  Its feature scope will be limited to some basic functionality, I don't want to build a second Ansible.
  Hence the scope is limited to the following parts of the system:
  - Configuration files
  - System packages (via package managers)
  - System services

## Similar Projects

#### Personal Computer Provisioning

- [`pets`](https://github.com/ema/pets) follows a very similar idea as bois.
  It's main point in difference is, that it's designed to be used for a single machine per repository, without templating.

  Additionally, its focus lies on configuration management and not so much on further system state such as packages or services.

- [`aconfmgr`](https://github.com/CyberShadow/aconfmgr) is very close to what bois aims to be, but focused on ArchLinux.
  It features:
  - Configuration file management
  - Diffing and merging
  - Package installation and removal

What bois has on top:

- Templating
- Builtin support for multiple machines
- Groups to allow modular package installation/configuration for various hosts.

#### System configuration manager

- [`etckeeper`](https://etckeeper.branchable.com/) is basically a dotfile manager but for your `/etc` system configuration.
  It doesn't track any additional info such as installed packages and anything outside `/etc`.

#### Dotfile managers

Bois is designed to be used as both, a system configuration manager as well as a dotfile manager.
For dotfiles specifically, there're a few well-established solutions out there.

- [chezmoi](https://chezmoi.io/) is a mature and powerful library to manage dotfiles for multiple systems.
  It contains pretty much all features of a good dotfiles manager, such as
  - Templating
  - Password manager integration
  - Encryption
  - Diffing and merging, which are two great features bois uses as well.
- [toml-bombadil](https://oknozor.github.io/toml-bombadil/) which is a bit of a newcomer, but also pretty nice and the tool I used before bois.
  It features templating and multi-system support and hooks, however it's configuration is a bit cumbersome for complex systems.

#### External Cluster/Server Fleet Provisioning

If you plan to manage a bunch of servers from the outside, please consider using an alternative solution.
There's a plenthora of server provisioning tools that work in different ways and follow different paradigms doing so.
To name just a few examples I personally worked with:

- [Ansible](https://www.ansible.com/) can be used to provision a fleet of servers via SSH. It's a mature solution albeit a bit slow.
- [Saltstack](https://saltproject.io/) is uses a custom protocol and usually deploys a master server that's pinged by the server fleet.
- [Chef](https://www.chef.io/) uses a master server to distribute provisioning scripts to the fleet. Configuration happens via code, which can be both a boon and a curse.

## Contributing

Feature requests and pull requests are very much appreciated and welcome!

Anyhow, please talk to me a bit about your ideas before you start hacking!
It's always nice to know what you're working on and I might have a few suggestions or tips :)

There's also the [Design.md](https://github.com/Nukesor/bois/blob/main/docs/Design.md), which is supposed to give you a brief overview and introduction to the project.

Copyright &copy; 2024 Arne Beer ([@Nukesor](https://github.com/Nukesor))
