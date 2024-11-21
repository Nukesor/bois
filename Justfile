# Bump all deps, including incompatible version upgrades
bump:
    just ensure_installed upgrade
    cargo update
    cargo upgrade --incompatible
    cargo test --workspace

# If you change anything in here, make sure to also adjust the lint CI job!
lint:
    just ensure_installed sort
    cargo fmt --all -- --check
    cargo sort --workspace --check
    cargo clippy --tests --workspace -- -D warnings

format:
    just ensure_installed sort
    cargo fmt
    cargo sort --workspace

ensure_installed *args:
    #!/bin/bash
    cargo --list | grep -q {{ args }}
    if [[ $? -ne 0 ]]; then
        echo "error: cargo-{{ args }} is not installed"
        exit 1
    fi
