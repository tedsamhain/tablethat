# tablethat dev tasks
# Install just: cargo install just
# Activate pre-commit hook: git config core.hooksPath .githooks

[private]
help:
    @just --list --unsorted

# cargo fmt
[group('dev')]
format:
    cargo fmt

# cargo fmt --check + cargo clippy
[group('dev')]
check:
    cargo fmt --check
    cargo clippy -- -D warnings -A clippy::unwrap_used

# cargo test
[group('dev')]
test:
    cargo test

# cargo build (debug)
[group('dev')]
build:
    cargo build

# cargo build (release)
[group('dev')]
build-release:
    cargo build --release

# Install tablethat binary to ~/.cargo/bin
[group('dev')]
install:
    cargo install --path .

# cargo deny + cargo audit
[group('security')]
audit:
    cargo deny check
    cargo audit

# Run format, linter, static checks and tests
[group('global')]
precommit:
    #!/usr/bin/env sh
    set -e
    just check
    just test

# Install dev tools + pre-commit hook
[group('global')]
setup-dev:
    cargo install cargo-deny cargo-audit just
    git config core.hooksPath .githooks
    @echo "Pre-commit hook enabled"
