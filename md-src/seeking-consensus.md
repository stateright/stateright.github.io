# Chapter 3: Seeking Consensus

In the last chapter we fixed a bug caused by the network's susceptibility to
message redelivery, but our solution could only run on a single server.
Introducing a second server would break linearizability as the system failed to
replicate information between servers.

In this chapter we introduce a simple replication protocol in an attempt to
address that shortcoming.

Initialize a new Rust project:

```sh
mkdir seeking-consensus
cd seeking-consensus
cargo init
```

Add dependencies to `Cargo.toml`. Note that we now need to include `serde`.

```toml
[dependencies]
env_logger = "0.7"
serde = "1.0"
serde_json = "1.0"
stateright = "0.20"
```

By now you have the hang of implementing basic actor systems in Stateright, so
we'll defer the full `main.rs` source code listing until later in the chapter.

## A Replication Protocol

First we get to decide on a replication protocol. Give this some thought, and
see if you have ideas.

> **Exercise**: Yes, really. Take some time to think about how you might add
replication.

Done? Great!

For this example, we'll proceed with a protocol that simply involves forwarding
the value to every peer server before replying to the client, thereby ensuring
the servers agree on the value. This might be the protocol that you envisioned
as well.

## Implementation Walkthrough

The first notable difference is the need to introduce a message type for
replication.

```rust,ignore,noplayground
{{#include ../rs-src/seeking-consensus/src/main.rs:actor-msg}}
```

The server defers sending a `PutOk` message until replicas reply, but
Stateright actors are nonblocking, so they must manage some additional state:

- the ID of the request, against which replica replies are matched (to guard
  against late responses),
- the ID of the client that made the request (to facilitate replying later),
- and the set of servers that have acknowledged the replicated value (to
  facilitate waiting until all have replied).

```rust,ignore,noplayground
{{#include ../rs-src/seeking-consensus/src/main.rs:actor-state}}
```
We are now ready to implement the protocol.

```rust,ignore,noplayground
{{#include ../rs-src/seeking-consensus/src/main.rs:actor}}
```

Now the big question: does this protocol solve the problem we ran
into last chapter?

Unfortunately achieving linearizability involves a bit more sophistication, and
Stateright identifies a sequence of steps that are not linearizable. The
sequence is nontrivial and demonstrates why a model checker is so useful for
implementing distributed systems.

```rust,ignore,noplayground
{{#include ../rs-src/seeking-consensus/src/main.rs:test}}
```

## Complete Implementation

Here is the complete implementation for `main.rs`:

```rust,ignore,noplayground
{{#include ../rs-src/seeking-consensus/src/main.rs:all}}
```

## Suggested Exercise

Uncomment the commented lines in the tests to cause them to fail, and see if
you can amend the actor implementation to make the tests pass. Do not get too
frustrated if you are unable to do so, as the next chapter will provide a
solution, and you may be surprised by its complexity if you are new to
implementing distributed system protocols.

Tips:

- Use `cargo test --release` when running the tests for dramatically better
  model checking performance. Running tests without that flag may result in
  significant delays.
- Read the [API docs for Stateright
  Explorer](https://docs.rs/stateright/latest/stateright/struct.CheckerBuilder.html#method.serve)
  to get an early preview of a powerful tool that will be fully introduced in a
  later chapter. See if you can use this to help debug your revised
  implementation. *Hint*: you can start Stateright Explorer by temporarily replacing
  `spawn_dfs().join()` with `serve("localhost:8000")` for example.

## Summary

This chapter introduced replication, and Stateright was able to find a bug in
our replication protocol. The next chapter will introduce a more sophisticated
protocol that makes the replicated register linearizable.

That next chapter is not yet available, so in the meantime you can learn more
about Stateright by browsing additional [Stateright
examples](https://github.com/stateright/stateright/tree/master/examples) and
reviewing the [Stateright API docs](https://docs.rs/stateright).

If you have any questions, comments, or ideas, please share them on
[Stateright's Discord server](https://discord.com/channels/781357978652901386).
At this time Stateright is a small personal project, and the main author is
eager to hear community feedback.
