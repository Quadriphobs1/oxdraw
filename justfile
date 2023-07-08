# Install just: https://github.com/casey/just
#
# Then run `just --list` to see the available commands

export RUSTDOCFLAGS := "--deny warnings --deny rustdoc::missing_crate_level_docs"

default:
  @just --list


### Common
# Format all of our code
format: toml-format
    cargo fmt --all

# Lint all of our code
lint: toml-lint rs-lint

### Rust

# Generate and open the documentation for Rerun and all of its Rust dependencies.
#
# `--keep-going` makes sure we don't to abort the build process in case of errors.
# This is an unstable flag, available only on nightly.
rs-doc:
    cargo +nightly doc --all --open --keep-going --all-features -Zunstable-options

# Lint all of Rust code
rs-lint:
    #!/usr/bin/env bash
    set -euxo pipefail
    cargo cranky --quiet --all-features -- --deny warnings
    typos
    cargo doc --quiet --no-deps --all-features
    cargo doc --quiet --document-private-items --no-deps --all-features
    cargo test --quiet --doc --all-features # runs all doc-tests

### TOML

# Format .toml files
toml-format:
    taplo fmt

# Lint .toml files
toml-lint:
    taplo fmt --check
