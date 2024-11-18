# Installation

There're bunch of ways to install `bois`:

### System Package Manager

The recommended and most convenient way to install `bois` is via your distribution's package manager.

You can check whether `bois` is available for your package manager with the following table. For more detail, just click on the image.

<a href="https://repology.org/project/bois/versions"><img src="https://repology.org/badge/vertical-allrepos/bois.svg?exclude_unsupported=1" alt="Packaging status"></a>

### Pre-built Static Binaries

Statically linked executables for ARM/Linux are built on each release. \
You can find the binary for your system on the [release page](https://github.com/Nukesor/bois/releases).

Just download it, rename it to `bois` and place it somewhere in your `$PATH`/program folder.

### Install via `cargo`

If you have the rust toolchain installed, you can build the latest release or directly from the Git repository

**Latest release**:

```sh
cargo install --locked bois
```

**Latest commit on the `git` repository**:

```sh
cargo install --locked --git https://github.com/Nukesor/bois.git bois
```
