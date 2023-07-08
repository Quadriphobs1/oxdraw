# Building Oxdraw

This is a guide to how to build Oxdraw.

## See also

- [`ARCHITECTURE.md`](ARCHITECTURE.md)
- [`CODE_STYLE.md`](CODE_STYLE.md)
- [`CONTRIBUTING.md`](CONTRIBUTING.md)

## Getting started with the repository

First, install the Rust toolchain using the installer from <https://rustup.rs/>.

Then, clone the repository:

```sh
git clone git@github.com:Quadriphobs1/oxdraw.git
cd oxdraw
```

Finally, run the following script to install the dependencies and CLI tools needed for Rerun's build environment:

```sh
./scripts/setup_dev.sh
```

Experiencing permission denied error with running the above `.sh` file, run `chmod +x *.sh` on the file with the error

Make sure `cargo --version` prints `1.69.0` once you are done.

If you are using an Apple-silicon Mac (M1, M2), make sure `rustc -vV` outputs `host: aarch64-apple-darwin`. If not, this should fix it:

```sh
rustup set default-host aarch64-apple-darwin && rustup install 1.69.0
```

## Building and running the viewer

Use this command for building and running the viewer:

```sh
cargo oxdraw
```

This custom cargo command is enabled by an alias located in `.cargo/config.toml`.
