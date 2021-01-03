# Building Distributed Systems With Stateright

A book about the [Stateright](https://github.com/stateright/stateright) model checker.

You can read the book at
[stateright.rs](https://www.stateright.rs).

## Repository Layout

- [md-src/](md-src/) -- The main text of the book.
- [rs-src/](rs-src/) -- Rust code imported by the book.
- [docs/](docs/) -- Artifacts built by `mdbook`.

## Instructions

Test the Rust code with `cargo`:

```sh
cargo test
```

Build the book with [mdbook](https://rust-lang.github.io/mdBook/):

```sh
mdbook build
```

[Pull requests](https://github.com/stateright/stateright.github.io) are welcome. Currently the
repository includes the built artifacts, so those can be included in pull requests.
