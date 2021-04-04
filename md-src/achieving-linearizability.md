# Chapter 4: Achieving Linearizability

In the last chapter we implemented a faulty replication protocol that violated
linearizability because the emulated register was not atomic. This chapter
introduces a more sophisticated replication protocol that the author first
learned about when reading the blog post "[Replicated/Fault-tolerant atomic
storage](https://muratbuffalo.blogspot.com/2012/05/replicatedfault-tolerant-atomic-storage.html)"
by Murat Demirbas.

As usual, we start by initializing a new Rust project:

```sh
mkdir achieving-linearizability
cd achieving-linearizability
cargo init
```

Then we define dependencies.

```toml
{{#include ../rs-src/achieving-linearizability/Cargo.toml}}
```

## A Truly Atomic Register

One problem with the earlier protocol is that reads can observe different
values depending on which server accepts the read. We need to amend the
protocol to ensure that given the same aggregate system state, any two reads
are guaranteed to observe the same value. The simplest solution is to query
every server when servicing a read operation, but that introduces yet another
problem: availability. By forcing reads to query every server, if a single
server becomes unavailable, the entire system becomes unavailable. This problem
also exists for writes in that replication protocol.

The trick to solving this new problem is observing that we only need to ensure
that "read sets" overlap one another. If a majority of servers agree on a
value, then we know that any other majority of servers must either agree on the
same value or must have a different value that is not part of the majority.  We
can apply the same logic to the "write set" as well.  We call the set of
sufficient reads a "read quorum" and the set of sufficient writes a "write
quorum."

Leveraging read and write quorums solves the availability problem when a
minority of servers are unreachable, but we still have a second availability
problem: in many cases a read set will not agree on a value. This can happen if
the read is concurrent with a write or if two earlier writes were concurrent.
In those cases, the server accepting the read must either ignore the request or
return an error, upon which we can improve.

To solve this remaining problem, we simply need to force the system into
agreement, thereby ensuring any subsequent read either observes the forced
agreement or a subsequent write. This is the technique employed by Attiya,
Bar-Noy, and Dolev in their paper "[Sharing Memory Robustly in Message-Passing
Systems](https://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.96.5395)".

Both read and write operations are handled in two phases: (1) a **query** phase
followed by (2) a **replication** phase.
  
- **1. Query Phase**: In the query phase, a server finds out what replicated
  values a quorum of replicas has previously accepted. The replicas also
  return a [logical clock](https://en.wikipedia.org/wiki/Logical_clock) that
  serves to sequence earlier write operations, so the implementation will refer
  to this as a *sequencer*.
- **2a. Write Replication Phase**: If servicing a write, then upon receiving
  responses from a quorum, the server replicates the chosen value along with a
  slightly larger sequencer than the one observed. Once a majority of the
  replicas ack this second phase, the server can indicate to the client that
  the write is complete.
- **2b. Read Replication Phase**: If servicing a read, then upon receiving
  responses from a quorum, the server replicates the value with the largest
  observed sequencer. Once a majority of the replicas ack this second phase,
  the server can return the value that it replicated.

## Implementation Walkthrough

We first define our message type. `Abd...` in the type refers to the names of
the algorithm's original author's -- Attiya, Bar-Noy, and Dolev -- or "ABD."

```rust,ignore,noplayground
{{#include ../rs-src/achieving-linearizability/src/main.rs:actor-msg}}
```

The current implementation combines the roles of "replica" and "coordinator
that accepts a request and facilitates replication." `seq` and `val` are for
the first role, while `phase` tracks information needed by the second role.

Of particular note is that for phase 1, writes need to remember what value to
later replicate, which is stored in `write`. Conversely for phase 2, reads need
to remember what value to later return to the client, which is stored in
`read`.

```rust,ignore,noplayground
{{#include ../rs-src/achieving-linearizability/src/main.rs:actor-state}}
```
We are now ready to implement the protocol. Note that the implementation
intentionally avoids decomposing the message handlers into different function
calls because each call needs to manage shared actor state. Spreading state
mutation across multiple locations in the source code arguably makes the
implementation harder to follow (much like mutable global variables), and
functions are only used where they can be reasoned about locally.

An alternative composition strategy that does often work well for actor systems
involves distinguishing roles so that each has less state (having different
`AbdServer` and `AbdReplica` actors for example), although we will not do that
for this short example. The next chapter on the other hand will demonstrate how
to compose a system of different actor types.

```rust,ignore,noplayground
{{#include ../rs-src/achieving-linearizability/src/main.rs:actor}}
```

The test case confirms that this implementation is linearizable. It also
introduces two new aspects:

1. The model contains a `within_boundary` predicate, which is used to reduce
   the number of visited states while still retaining systematic testing.
2. The model is parameterized by a configuration type `AbdModelCfg`.

Remember to run the tests with the `--release` flag if you want to check with a
larger max clock or number of clients/servers as the state space
grows rapidly.

```rust,ignore,noplayground
{{#include ../rs-src/achieving-linearizability/src/main.rs:test}}
```

## Complete Implementation

Here is the complete implementation for `main.rs`:

```rust,ignore,noplayground
{{#include ../rs-src/achieving-linearizability/src/main.rs:all}}
```

## Suggested Exercises

1. This algorithm can be optimized by observing that the replication phases
   need not replicate values if the quorum already agrees on a value. See if
   you can implement this optimization.
2. More generally, replication messages only need to be sent to replicas that
   disagree. See if you can implement this optimization as well.
   
   > TIP: This is a slightly more complex optimization because we need to treat
   "all agree" versus "not all agree" slightly differently to avoid dropping
   requests in some cases. Can you see why?

## Summary

This chapter provided the first taste of a real distributed algorithm. We were
able to incrementally infer a solution, but it was nontrivial. If there
were bugs in the code, they could be relatively difficult to identify without a
model checker.

In the next chapter, we will introduce the notion of "consensus" and implement
it via the Multi-Paxos algorithm. That chapter is not yet available, so in the
meantime you can learn more about Stateright by browsing additional [Stateright
examples](https://github.com/stateright/stateright/tree/master/examples) and
reviewing the [Stateright API docs](https://docs.rs/stateright). If you are
familiar with TLA+, then the subsequent chapter [Comparison with
TLA+](./comparison-with-tlaplus.md) may also be interesting to you.

If you have any questions, comments, or ideas, please share them on
[Stateright's Discord server](https://discord.com/channels/781357978652901386).
At this time Stateright is a small personal project, and the main author is
eager to hear community feedback.
