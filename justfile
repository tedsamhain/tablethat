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
    cargo clippy -- -D warnings

# cargo test
test:
    cargo test

# cargo build (debug)
build:
    cargo build

# Install plan and gloss binaries to ~/.cargo/bin, themes to platform config
install:
    cargo install --path .
    @mkdir -p ~/.config/plan/themes ~/.config/gloss/themes
    @cp themes/*.toml ~/.config/plan/themes/
    @cp themes/*.toml ~/.config/gloss/themes/
    @echo "Installed themes to ~/.config/plan/themes/ and ~/.config/gloss/themes/"

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
