# Oxdraw architecture

This document describes the technical architecture of Oxdraw.

## See also
* [`BUILD.md`](BUILD.md)
* [`CODE_STYLE.md`](CODE_STYLE.md)
* [`CONTRIBUTING.md`](CONTRIBUTING.md)

## The major components

TODO


## Crates

In order to get an overview of our in-house crates and how they depend on each other, we recommend you run:

```
cargo install cargo-depgraph
cargo depgraph --all-deps --workspace-only --all-features --dedup-transitive-deps | dot -Tpng > deps.png
open deps.png
```

and:

```
cargo doc --no-deps --open
```

## Technologies we use

TODO