# Bump all deps, including incompatible version upgrades
bump:
    just ensure-cargo-installed upgrade
    cargo update
    cargo upgrade --incompatible
    cargo test --workspace

# If you change anything in here, make sure to also adjust the lint CI job!
# Lint all parts of the code
lint:
    just ensure-cargo-installed sort
    cargo fmt --all -- --check
    cargo sort --workspace --check
    cargo clippy --tests --workspace -- -D warnings

# Format all parts of the code
format:
    just ensure-cargo-installed sort
    cargo fmt
    cargo sort --workspace

# Serve the book locally
book:
    just ensure-command mdbook
    mdbook serve book

# Ensures that a required cargo subcommand is installed
ensure-cargo-installed *args:
    #!/bin/bash
    cargo --list | grep -q {{ args }}
    if [[ $? -ne 0 ]]; then
        echo "error: cargo-{{ args }} is not installed"
        exit 1
    fi

# Ensures that a required command is installed
ensure-command command:
    #!/bin/bash
    if ! command -v {{ command }} > /dev/null 2>&1 ; then
        echo "Couldn't find executable '{{ command }}'"
        exit 1
    fi
