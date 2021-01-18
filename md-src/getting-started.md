# Chapter 1: Getting Started

> **IMPORTANT**: Stateright is a relatively new framework and will be making breaking API
changes leading up to a 1.0 release, at which point the API will be considered
stable. If you plan to use Stateright for production scenarios, then please
[file a GitHub issue](https://github.com/stateright/stateright/issues/new) so
that the author can coordinate with you to minimize any disruption.

Let's start with the simplest nontrivial distributed system: a single client
that can interact with a single server by reading or writing a value. We'll see
that even this minimal example is susceptible to surprising behavior.

[Install the Rust programming
language](https://www.rust-lang.org/learn/get-started) if it is not already
installed, then initialize a new project using the `cargo` utility included
with Rust. If you are new to Rust, then you may want to review some of the
language's [learning resources](https://www.rust-lang.org/learn).

```sh
mkdir getting-started
cd getting-started
cargo init
```

Define dependencies in `Cargo.toml`.

```toml
{{#include ../rs-src/getting-started/Cargo.toml}}
```

Here is the complete implementation for `main.rs`. Copy-paste it into your own
file. The next section explains the most important aspects.

```rust,ignore,noplayground
{{#include ../rs-src/getting-started/src/main.rs:all}}
```

## Implementation Walkthrough

The code starts by implementing a simple server using the "[actor
model](https://en.wikipedia.org/wiki/Actor_model)." An actor is an event
oriented process that is able to communicate with other actors over a network
by sending messages.

The server responds to `Put` and `Get` messages based only on its own local
state, providing its clients with a simple form of distributed storage,
sometimes known as a [shared
register](https://en.wikipedia.org/wiki/Shared_register).  Responses are linked
to requests via a request ID chosen by the client.

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

The test checks the system for a property called
[linearizability](https://en.wikipedia.org/wiki/Linearizability), which loosely
speaking means that the visible behavior of the register abstraction provided
by the actors is identical to that of a register within a single-threaded
system.

> **Terminology**: The term "linearizable" derives from the insight that the
operations executed by a system form a directed acyclic graph where edges
indicate "happens before." For example, if Computer1 sends messages to invoke
operations on Computer2 and Computer3, then the message sends happens before
Computer2 or Computer3 handle them, but the handling of the messages by
Computer2 and Computer 3 lack a defined order because the messages
[race](https://en.wikipedia.org/wiki/Race_condition). A "linearization" defines
a viable linear order such as "Computer 1 sends the messages, Computer 3
handles one, and then Computer2 handles the other." A system is not
linearizable when its behavior cannot be mapped to a linearization. For
example, if the operation invoked on Computer2 was `Append "Hello"` and
Computer3 was `Append "World"`, but the final value interlaced the inputs to
form `"WolloHerld"`, then the system would not be linearizable. Either
`"HelloWorld"` or `"WorldHello"` on the other hand would be valid
linearizations since the messages race.

The test leverages
[`RegisterTestSystem`](https://docs.rs/stateright/0.18.0/stateright/actor/register/struct.RegisterTestSystem.html),
which is built into Stateright and defines a system whereby a specified number
of clients (only 1 in this case) write distinct values and independently read
values without coordinating with one another. Under the hood
`RegisterTestSystem` also leverages Stateright's built-in
[`LinearizabilityTester`](https://docs.rs/stateright/latest/stateright/semantics/struct.LinearizabilityTester.html).

Stateright is able to find a bug that arises even if there is only a single
client. The bug manifests whenever the network redeliveries messages, something
that can and does happen in practice.

```rust,ignore,noplayground
{{#include ../rs-src/getting-started/src/main.rs:test}}
```

The last bit of code defines the `main` method, which runs the actor on a UDP
socket, encoding messages with the JSON format.

```rust,ignore,noplayground
{{#include ../rs-src/getting-started/src/main.rs:main}}
```

## Running

Confirm the system behaves as expected by running the test, which should pass
because the test asserts that the bug exists. It's a good idea to get into the
habit of passing the `--release` flag when testing more complex systems so that
Rust fully optimizes the code, as testing can be computationally intensive and
time consuming.

```sh
cargo test --release
```

Now run the actor on a UDP socket.

```sh
cargo run
```

If using a POSIX-oriented operating system,
[netcat](https://en.wikipedia.org/wiki/Netcat) can be used to interact with the
actor from a different terminal window. Actor responses are omitted from the
listing below for clarity, but you will see messages such as `{"PutOk":0}`
printed to STDOUT. Numbers in the messages are request IDs, the importance of
which will be more evident in the next chapter.

```sh
nc -u localhost 3000
{"Put":[0,"X"]}
{"Get":1}
{"Put":[2,"X"]}
{"Get":3}
```

## Exercise

Uncomment the `// TRY IT` line, then run the test again. It should fail
indicating a sequence of steps that would cause the linearizability expectation
to be violated. This exercise demonstrates how Stateright can detect flaws that
would likely go undetected when simply reviewing the actor implementation.

## Summary

This chapter introduced one of the simplest possible distributed systems and
showed how Stateright can find a subtle bug. The next chapter [Taming the
Network](./taming-the-network.md) will address that bug.
