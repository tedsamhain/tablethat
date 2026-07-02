# tablethat dev tasks — plan + gloss
# Install just: cargo install just
# Activate pre-commit hook: git config core.hooksPath .githooks

[private]
help:
    @just --list --unsorted

# cargo fmt
format:
    cargo fmt

# cargo fmt --check + cargo clippy
check:
    cargo fmt --check
    cargo clippy -- -D warnings -A clippy::unwrap_used

# cargo test
test:
    cargo test

# cargo build (debug)
build:
    cargo build

# Install plan and gloss binaries to ~/.cargo/bin
install:
    cargo install --path .

# cargo build (release)
build-release:
    cargo build --release

# Install dev tools
install-dev:
    cargo install cargo-deny cargo-audit

# Install dev tools + pre-commit hook
setup-dev: install-dev
    git config core.hooksPath .githooks
    @echo "Pre-commit hook enabled"

# cargo deny + cargo audit
audit:
    cargo deny check
    cargo audit

# Run format, linter, static checks and tests
precommit:
    #!/usr/bin/env sh
    set -e
    just check
    just test
