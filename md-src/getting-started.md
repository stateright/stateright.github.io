# Chapter 1: Getting Started

> ## Caveats
>
> While Stateright is seeking early adopters, it is important to clarify that its actor runtime has
> not gone through a security review. If you want to use Stateright for production scenarios, then
> please [file a GitHub issue](https://github.com/stateright/stateright/issues/new) so that this can
> be prioritized before your product is live.
>
> Furthermore, the project is pre-1.0 and will be making breaking API changes leading up to a 1.0
> release.

Let's start with the simplest nontrivial distributed system: a single server
that can set or get a value. We'll see that even this minimal example is
susceptible to surprising behavior.

Initialize a new Rust project:

```sh
mkdir getting-started
cd getting-started
cargo init
```

Then add `serde` and `stateright` to the dependencies in `Cargo.toml`:

```toml
[dependencies]
env_logger = "0.7"
serde_json = "1.0"
stateright = "0.20"
```

Here is the complete implementation for `main.rs`, explained below:

```rust,ignore,noplayground
{{#include ../rs-src/getting-started/src/main.rs:all}}
```

## Explanation

The code starts by implementing a simple server actor that responds to `Put`
and `Get` messages based only on its own local state, providing a [shared
register](https://en.wikipedia.org/wiki/Shared_register) abstraction to its
clients. Responses are linked to requests via a request ID chosen by the
client.

```rust,ignore,noplayground
{{#include ../rs-src/getting-started/src/main.rs:actor}}
```

A test follows. It employs a technique called model checking that is similar
to [property based testing](https://github.com/BurntSushi/quickcheck), wherein
the programmer writes a predicate indicating what constitutes correct behavior;
for example, "*the function's output is a sorted permutation of its input*."
But whereas property based testing often enumerates the outputs of a
deterministic algorithm based on a random sampling of possible inputs, model
checking often systematically enumerates the reachable states of a
nondeterministic system with less focus on the variety of initial inputs; for
example "*if clients write `X` followed by `Y` to the the same database key
`K`, then a subsequent read may only return `Y` until the next write,
regardless of how the network reorders or redelivers messages*."

The test leverages
[`RegisterTestSystem`](https://docs.rs/stateright/0.18.0/stateright/actor/register/struct.RegisterTestSystem.html),
which specifies a system in which a specified number of clients concurrenly
write distinct values and independently read values without coordinating with
one another. Under the hood `RegisterTestSystem` also leverages Stateright's
built-in
[`LinearizabilityTester`](https://docs.rs/stateright/latest/stateright/semantics/struct.LinearizabilityTester.html)
to exercise the system. Notice how Stateright identifies a bug, which only
manifests when the network delivers messages.

```rust,ignore,noplayground
{{#include ../rs-src/getting-started/src/main.rs:test}}
```

The actor can also be run on a UDP socket, handling JSON messages:

```rust,ignore,noplayground
{{#include ../rs-src/getting-started/src/main.rs:main}}
```

## Running

Confirm the system behaves as expected by running the test, which should pass:

```sh
cargo test
```

Now run the actor over UDP using Stateright's actor runtime:

```sh
cargo run
```

If using a POSIX-oriented operating system, [netcat](https://en.wikipedia.org/wiki/Netcat) can be
used to interact with the actor. Actor responses are omitted from the listing below for clarity,
but you will see messages such as `{"PutOk":0}` printed to STDOUT. Numbers in the messages are
request IDs.

```sh
nc -u localhost 3000
{"Put":[0,"X"]}
{"Get":1}
{"Put":[2,"X"]}
{"Get":3}
```

## Summary

This chapter introduced one of the simplest possible actor systems and showed
how Stateright can find a subtle bug. The next chapter [Taming the
Network](./taming-the-network.md) will address that bug.
