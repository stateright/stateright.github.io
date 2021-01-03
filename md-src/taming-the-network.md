# Chapter 2: Taming the Network

In the last chapter we discovered a bug caused by the network's susceptibility
to message redelivery. We address that in this chapter.

Initialize a new Rust project:

```sh
mkdir taming-the-network
cd taming-the-network
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
{{#include ../rs-src/taming-the-network/src/main.rs:all}}
```

## Explanation

Addressing the linearizability issue is merely a matter of throwing away
message redeliveries. The simplest approach is to record every delivered request ID.

```rust,ignore,noplayground
{{#include ../rs-src/taming-the-network/src/main.rs:actor}}
```

With that small change, each server provides an independent linearizable
register. In the presence of messages concurrently in flight, the register
abstraction is still atomic and [linearizable](https://en.wikipedia.org/wiki/Linearizability) (for
example, reads cannot observe values overwritten before the read began, among other
characteristics).

The servers feature no replication, so a collection of servers does not provide
a unified service that emulates a linearizable register as the servers will not
generally agree upon the last value they received.

```rust,ignore,noplayground
{{#include ../rs-src/taming-the-network/src/main.rs:test}}
```

## Suggested Exercises

1. **Compaction**: Storing every request ID isn't viable for a long running
   process.  The simplest approach is to require that request IDs are
   "monotonic" -- which means they are increasing (and gaps are acceptable). In
   that case, the delivery handler throws away messages with a request ID
   smaller than the last handled message. See if you can amend the example
   accordingly.
2. **Optimized Compaction**: Reordered messages will be dropped because late
   delivered message will have a smaller request ID. Protocols need to account
   for the network dropping messages anyway, so generally speaking this
   tradeoff only impacts performance. Still, throughput can be improved by
   adding a "sliding window" buffer on the server side to minimize dropped
   messages. See if you can implement that.
3. **Lossless Link**: Another technique for minimizing message loss is to have
   the client also maintain a buffer of outgoing messages, and the client
   periodically resends messages that have not been acknowledged by the
   recipient within a particular timeout period. TCP for example does this for
   packets. See if you can implement this as well. If you need help, see
   [`ordered_reliable_link.rs`](https://github.com/stateright/stateright/blob/master/src/actor/ordered_reliable_link.rs) in the Stateright repository.

## Summary

This chapter showed how to fix the implementation from the previous chapter,
maintaining linearizability even if messages are redelivered. The next chapter
[Seeking Consensus](./seeking-consensus.md) will introduce the concept of
replication, which is used to provide (1) data recoverability in the event of a
server crash and/or (2) improved performance for high request rates by
distributing requests (such as reads) across a wider range of hosts.
