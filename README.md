# Bois

[![Book](https://img.shields.io/badge/Read%20the%20book-blue)](https://nukesor.github.io/bois/guide/setup.html)
[![Test Build](https://github.com/Nukesor/bois/actions/workflows/test.yml/badge.svg)](https://github.com/Nukesor/bois/actions/workflows/test.yml)
[![Crates.io](https://img.shields.io/crates/v/bois)](https://crates.io/crates/bois)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/github/downloads/nukesor/bois/total.svg)](https://github.com/nukesor/bois/releases)

**DON'T USE THIS YET.**
The project is highly experimental and very much work in progress. \
It's only public so I don't have to hassle around with deployment keys and such.

## What does it do?

Bois is an opinionated dotfile or system provisioning tool for your :sparkles:personal:sparkles: computers (hosts).

It allows you to manage your configuration files and to synchronize or share them between your hosts.
There's support for templating, per-host configuration and sets of shared configuration that can be opt-in on a per host basis.

On top of this, `bois` can also manage your installed packages and enabled services.

For a full tour, check out the [book](https://nukesor.github.io/bois/guide/how_to_dotfiles.html).

You could say that it aims to strike a balance between Chezmoi and Ansible/Saltstack, but on-host and for your own computers.

- [Features](https://github.com/Nukesor/bois#features)
- [Installation](https://github.com/Nukesor/bois#installation)
- [Design Goals](https://github.com/Nukesor/bois#design-goals)
- [Similar Projects](https://github.com/Nukesor/bois#similar-projects)
- [Contributing](https://github.com/Nukesor/bois#contributing)

## Features

- Configuration file management
  - Allow editing of deployed files
  - Diffing/Merging of deployed files vs. changed files in bois directory.
  - Safety first: Don't overwrite changes without a prompt.
- Granular system package management (via package managers)
- Granular system service management (e.g. Systemd).
  - Dis-/Enable services based on deployed files.
- Automatic Cleanup
  - Remove deployed files/directories.
  - Uninstall packages.
  - Disable/stop services.
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
cargo install --locked --path .
```

The final binaries will be located in `target/release/bois`.

## Design Goals

The main focus for bois is that it's supposed to be run on bare-metal **personal** machines, i.e. your desktop, laptop and maybe your home-server/NAS.
It's also supposed to be used from **inside** the system, in contrast to other tools, which provision systems from the outside (for example via ssh (`ansible`) or a orchestrator server (`chef`)).

Additionally, there're a few "buzzwordy" design goals that I aim for:

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
  Hence the scope is (for now) limited to the following parts of the system:
  - Configuration files
  - System packages (via package managers)
  - System services

Like all of my projects, this one is also designed to cover the 90-95% use-case.
It doesn't aim to be a jack of all trades solution and more complex use-cases might be dropped to keep the project **usable and maintainable**.
The idea is to reach a state where most people are happy with it and then enter some form of soft "feature freeze".

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
- Builtin support for multiple hosts
- Traits to allow modular package installation/configuration for various hosts.

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
- [Saltstack](https://saltproject.io/) uses a master server that's pinged by the server fleet to keep them in sync.
- [Chef](https://www.chef.io/) uses a master server to distribute provisioning scripts to the fleet. Configuration happens via code, which can be both a boon and a curse.

## Project History

- ~2020: Project ideation
- 2020-2022: On-off designing, technology choice and prototyping.
- November 2022: First actual commit in repository.
- December 2024: Project is in a raw functional state that's actually in use by a few friends.
- August 2026: Major rewrite of all internal components is finished. Most functionality is there, but UI still needs a rework.

## Why name it "bois"

Quite a few reasons. First of, I kinda like the sound of it "boiiiiis". And it has a sillier connotation than "boys", mostly from usage in mostly stupid but fun memes. I always have to think about the "yeah boi" dub of a particular very angry desert frog that lets out his ferocious war cry.

Also "bois" seems like a much more German way of writing that word. Feels more natural, as we barely use `y` in words.

There's also those super cute grafittis **all** over Hamburg. I kind of started naming them "the bois" and at some point even started making photos of every one I found. Those are still in my "bois" album. Here's one of those pictures:

![da bois](./.github/smol_bois.jpg)

Also, when I talk about any of my machines, I pretty much always call them "bois", like "Ne, das läuft auf dem boi da". And when I think about the bulk of them, I imagine those small graffiti bois. They're my bois.

So yeah, all of that culminates in the decision of calling this project "bois". I like it and I think it's funny.

## Contributing

Feature requests and pull requests are very much appreciated and welcome!

Anyhow, please talk to me a bit about your ideas before you start hacking!
It's always nice to know what you're working on and I might have a few suggestions or tips :)

There's also the [Design.md](https://github.com/Nukesor/bois/blob/main/dev_docs/Architecture/Design.md), which is supposed to give you a brief overview and introduction to the project.

Copyright &copy; 2024 Arne Beer ([@Nukesor](https://github.com/Nukesor))
