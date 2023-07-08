# Oxdraw code style

## See also

- [`ARCHITECTURE.md`](ARCHITECTURE.md)
- [`BUILD.md`](BUILD.md)
- [`CONTRIBUTING.md`](CONTRIBUTING.md)

## Rust code

### Avoid `unsafe`

`unsafe` code should be only used when necessary, and should be carefully scrutinized during PR reviews.

### Avoid `unwrap`, `expect` etc.

The code should never panic or crash, which means that any instance of `unwrap` or `expect` is a potential time-bomb. Even if you structured your code to make them impossible, any reader will have to read the code very carefully to prove to themselves that an `unwrap` won't panic. Often you can instead rewrite your code so as to avoid it. The same goes for indexing into a slice (which will panic on out-of-bounds) - it is often preferable to use `.get()`.

For instance:

```rust
let first = if vec.is_empty() {
    return;
} else {
    vec[0]
    vec[0]
};
```

can be better written as:

```rust
let Some(first) = vec.get(0) else { return; };
```

### Error handling and logging

We use simple event logging system `env_logger` and `log` crate.

- An error should never happen in silence.
- Validate code invariants using `assert!` or `debug_assert!`.
- Validate user data and return errors using [`thiserror`](https://crates.io/crates/thiserror).
- Attach context to errors as they bubble up the stack using [`anyhow`](https://crates.io/crates/anyhow).
- Log errors using `log::error!`.
- If a problem is recoverable, use `log::warn!`.
- If an event is of interest to the user, log it using `log::info!`.
- The code should only panic if there is a bug in the code.
- Never ignore an error: either pass it on, or log it.
- Handle each error exactly once. If you log it, don't pass it on. If you pass it on, don't log it.

Strive to encode code invariants and contracts in the type system as much as possible. [Parse, don’t validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/).

Some contracts cannot be enforced using the type system. In those cases you should explicitly enforce them using `assert` (self-documenting code) and in documentation (if it is part of a public API).

### Log levels

The log is for several distinct users:

- The application user
- The application programmer
- The library user
- The library programmer

#### `ERROR`

This is for _unrecoverable_ problems. The application or library couldn't complete an operation.

Libraries should ideally not log `ERROR`, but instead return `Err` in a `Result`, but there are rare cases where returning a `Result` isn't possible (e.g. then doing an operation in a background task).

Application can "handle" `Err`ors by logging them as `ERROR` (perhaps in addition to showing a popup, if this is a GUI app).

#### `WARNING`

This is for _recoverable_ problems. The operation completed, but couldn't do exactly what it was instructed to do.

Sometimes an `Err` is handled by logging it as `WARNING` and then running some fallback code.

#### `INFO`

This is the default verbosity level. This should mostly be used _only by application code_ to write interesting and rare things to the application user. For instance, you may perhaps log that a file was saved to specific path, or where the default configuration was read from. These things lets application users understand what the application is doing, and debug their use of the application.

#### `DEBUG`

This is a level you opt-in to to debug either an application or a library. These are logged when high-level operations are performed (e.g. texture creation). If it is likely going to be logged each frame, move it to `TRACE` instead.

#### `TRACE`

This is the last-resort log level, and mostly for debugging libraries or the use of libraries. Here any and all spam goes, logging low-level operations.

The distinction between `DEBUG` and `TRACE` is the least clear. Here we use a rule of thumb: if it generates a lot of continuous logging (e.g. each frame), it should go to `TRACE`.

### Libraries

We use [`thiserror`](https://crates.io/crates/thiserror) for errors in our libraries, and [`anyhow`](https://crates.io/crates/anyhow) for type-erased errors in applications.

### Style

We follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html).

We use `rust fmt` with default settings.

We have blank lines before functions, types, `impl` blocks, and docstrings.

We format comments `// Like this`, and `//not like this`.

When importing a `trait` to use its trait methods, do this: `use Trait as _;`. That lets the reader know why you imported it, even though it seems unused.

When intentionally ignoring a `Result`, prefer `foo().ok();` over `let _ = foo();`. The former shows what is happening, and will fail to compile if `foo`:s return type ever changes.

### `TODO`:s

When you must remember to do something before merging a PR, write `TODO` or `FIXME` in any file. The PR will not be approved you either remove them or rewrite them as `TODO(yourname)`.

You can also use the `todo()!` macro during development, but again it won't pass CI until you rewrite it as `todo!("more details")`. Of course, we should try to avoid `todo!` macros in our code.

### Misc

Use debug-formatting (`{:?}`) when logging strings in logs and error messages. This will surround the string with quotes and escape newlines, tabs, etc. For instance: `log::warn!("Unknown key: {key:?}");`.

## Naming

When in doubt, be explicit. BAD: `id`. GOOD: `msg_id`.

Be terse when it doesn't hurt readability. BAD: `message_identifier`. GOOD: `msg_id`.

Avoid negations in names. A lot of people struggle with double negations, so things like `non_blocking = false` and `if !non_blocking { … }` can become a source of confusion and will slow down most readers. So prefer `connected` over `disconnected`, `initialized` over `uninitialized` etc.
