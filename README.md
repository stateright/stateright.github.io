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

Build the book with [Graphviz](https://graphviz.org/),
[Gnuplot](http://www.gnuplot.info/), and
[mdBook](https://rust-lang.github.io/mdBook/) using the included script:

```sh
./build.sh
```

You can preview your updates locally with `mdbook serve`, but remember to
rebuild before preparing your commit:

```sh
mdbook serve # Iterate with "serve"
^C           # ... then ctrl-C to quit
./build.sh   # ... and rebuild.
git commit
```

[Pull requests](https://github.com/stateright/stateright.github.io) are welcome. Currently the
repository includes the built artifacts, so those can be included in pull requests.
