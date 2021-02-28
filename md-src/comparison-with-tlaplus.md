# Comparison with TLA+

This chapter compares two abstract models of the [two-phase commit
protocol](https://en.wikipedia.org/wiki/Two-phase_commit_protocol): one written
in a language called [TLA+](https://en.wikipedia.org/wiki/TLA%2B) and the other
written in Rust. The chapter exists to assist those who are already familiar
with TLA+, so feel free to skip if you are not familiar with TLA+.

## Attribution

The TLA+ model comes from the paper "[Consensus on transaction
commit](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/tr-2003-96.pdf)"
by Jim Gray and Leslie Lamport and is used in accordance with the ACM's
[Software Copyright
Notice](https://www.acm.org/publications/policies/software-copyright-notice).
It has been adapted slightly for this book. Here are the copyright details from
the paper:

> Copyright 2005 by the Association for Computing Machinery, Inc. Permission
to make digital or hard copies of part or all of this work for personal or
classroom use is granted without fee provided that copies are not made or
distributed for profit or  commercial  advantage and  that  copies  bear this
notice  and  the  full  citation on the first page.  Copyrights for components
of this work owned by others than ACM must be honored.  Abstracting with credit
is permitted.  To copy otherwise, to republish, to post on servers, or to
redistribute to lists, requires prior specific permission and/or a fee.
Request permissions from Publications Dept, ACM Inc., fax +1 (212) 869-0481, or
[permissions@acm.org](mailto:permissions@acm.org).

Citation:

> Jim Gray and Leslie Lamport. 2006. Consensus on transaction commit. ACM
Trans. Database Syst. 31, 1 (March 2006), 133–160.
DOI:[https://doi.org/10.1145/1132863.1132867](https://doi.org/10.1145/1132863.1132867)

## Unique Benefits of Each

Before getting to the code, it is valuable to highlight that while the
functionality of TLC (the model checker for TLA+) and Stateright overlap to
some degree, each has unique benefits. Enumerating some of these can assist
with deciding when to use each solution.

Unique benefits of TLC/TLA+:

- **Brevity**: TLA+ is more concise than Rust.
- **State Reduction**: TLC supports symmetry reduction.
- **Features**: TLC supports arbitrarily complex temporal properties including
  fairness.
- **Features**: TLC supports refinement mapping between models.

Unique benefits of Stateright/Rust:

- **Additional Verification**: With Stateright your model and final
  implementation are encouraged to share code. This eliminates a possible
  failure mode whereby a model and it's resulting production system
  implementation deviate.
- **Reuse and Extensibility**: Rust has a far larger library ecosystem and is
  arguably more amenable to reuse than TLA+. For example, Stateright's code for
  defining a system's [operational
  semantics](https://docs.rs/stateright/latest/stateright/semantics/index.html)
  are not built into the model checker and could be provided as an external
  library. The same can be said about the included [register
  semantics](https://docs.rs/stateright/latest/stateright/semantics/register/index.html),
  [linearizability
  tester](https://docs.rs/stateright/latest/stateright/semantics/struct.LinearizabilityTester.html),
  and [actor
  model](https://docs.rs/stateright/latest/stateright/actor/index.html) to name
  a few other examples. More generally the entire [Rust crate
  registry](https://crates.io/) (with tens of thousands of libraries) is at
  your disposal. In contrast, the pattern for reuse in TLA+ is
  [copy-paste-modify](https://groups.google.com/g/tlaplus/c/BHBNTkJ2QFE/m/meTQs4pHBwAJ),
  and the number of reusable modules is [relatively
  small](https://github.com/tlaplus/CommunityModules).
- **Checker Performance**: Stateright tends to be faster and offers additional
  optimization possibilities such as replacing a set with a bit vector for
  example. While TLC allows you to [override modules with Java
  implementations](https://stackoverflow.com/questions/53908653/use-module-overloading-to-implement-a-hash-function-in-tla),
  doing so is relatively cumbersome and rarely used.
- **Final Implementation Performance**: As a more auxiliary benefit, Stateright
  can serve as a stress test for your final implementation, identifying
  regressions and also facilitating performance investigation.
- **Features**: Stateright offers
  "[sometimes](https://docs.rs/stateright/latest/stateright/struct.Property.html#method.sometimes)"
  properties that serve to sanity check that expected outcomes are possible.
  These are less powerful than temporal properties but serve a slightly
  different purpose because examples of these properties being met are included
  in the checker discoveries.
- **Features**: [Stateright
  Explorer](https://docs.rs/stateright/latest/stateright/struct.CheckerBuilder.html#method.serve)
  allows you to interactively browse a model's state space and also lets you
  jump between discoveries (whether they are property violations or instances
  of "sometimes" properties).

With those out of the way, let's move on to the code comparison.

## Code Comparison

Both models are parameterized by a collection of "resource managers." The TLA+
spec does not specify a type, but a set is expected. The Stateright spec maps
each resource manager to an integer in the range `0..N` (`0` to `N-1` inclusive).

<table><tr><td>

```ignore,noplayground
{{#include ../other-src/comparison-with-tlaplus/TwoPhase.tla:constants}}
```

</td><td>

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/src/lib.rs:constants}}
```

</td></tr></table>

Next we define variables. These are global in the TLA+ spec, and their type
constraints are indicated later in the spec via an invariant. In Stateright
these have a statically defined type.

<table><tr><td>

```ignore,noplayground
{{#include ../other-src/comparison-with-tlaplus/TwoPhase.tla:variables}}
```

</td><td>

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/src/lib.rs:variables}}
```

</td></tr></table>

Types in the TLA+ spec are conveyed via an invariant that is passed to its
model checker, TLC.

<table><tr><td>

```ignore,noplayground
{{#include ../other-src/comparison-with-tlaplus/TwoPhase.tla:types}}
```

</td><td>

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/src/lib.rs:types}}
```

</td></tr></table>

TLA+ leverages temporal logic to convey the specification, while Stateright
requires that a trait is implemented. One other distinct aspect is that
Stateright actions are also types.

<table><tr><td>

```ignore,noplayground
{{#include ../other-src/comparison-with-tlaplus/TwoPhase.tla:spec}}
```

</td><td>

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/src/lib.rs:spec}}
    ...
}
```

</td></tr></table>

The initial aggregate state is one where

- the resource mangers are ready to start a transaction;
- the sole transaction manager is ready to start a transaction;
- the transaction manager's view of each resource manager indicates that none
  have prepared for a transaction yet;
- and the network is an empty set.

Note that set semantics provide an ideal model for a network in this case
because they capture the fact that networks rarely make guarantees about
message order.  Those guarantees must be imposed by additional protocols. And
keep in mind that often those protocols only provide guarantees under limited
scopes (e.g. TCP only orders messages within the lifetime of that connection,
so a transient network partition can still cause message redelivery with TCP).

<table><tr><td>

```ignore,noplayground
{{#include ../other-src/comparison-with-tlaplus/TwoPhase.tla:init}}
```

</td><td>

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/src/lib.rs:init}}
```

</td></tr></table>

Now we get to the most interesting part of the model, the state transitions.
TLA+ requires each action definition to precede the aggregate next state
relation (`TPNext` in this case). Each action serves two roles: (1) it defines
its own preconditions and (2) it definies the subsequent state change (along
with the unchanged states). In Stateright it is more idiomatic (and performant)
to distinguish between the preconditions (in `fn actions...`) and state change
(in `fn next_state...`).

<table><tr><td>

```ignore,noplayground
{{#include ../other-src/comparison-with-tlaplus/TwoPhase.tla:next}}
```

</td><td>

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/src/lib.rs:next}}
```

</td></tr></table>

Then we get to the sole property: if a resource manager reaches a final
commit/abort state, then no other resource manager can disagree with that
decision.

<table><tr><td>

```ignore,noplayground
{{#include ../other-src/comparison-with-tlaplus/TwoPhase.tla:properties}}
```

</td><td>

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/src/lib.rs:properties}}
```

</td></tr></table>

# Performance Comparison

Now we need to configure the model. For TLC, this is done via a special "CFG"
file, while for Stateright you simply introduce a Rust test.

<table><tr><td>

```ignore,noplayground
{{#include ../other-src/comparison-with-tlaplus/TwoPhase.cfg}}
```

</td><td>

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/src/lib.rs:configuration}}
```

</td></tr></table>

TLA+ can be run from the command line using a tool such as
[tla-bin](https://github.com/pmer/tla-bin) or
[tlacli](https://github.com/hwayne/tlacli), while the Stateright test is run
using `cargo test --release`. The example below first calls `build` to avoid
timing compilation.

<table><tr><td>

```ignore,noplayground
$ tlc -workers auto TwoPhase.tla
...
18507778 states generated,
1745408 distinct states found,
0 states left on queue.
...
Finished in 12s at (2021-02-28 15:28:44)
```

</td><td>

```ignore,noplayground
$ cargo build --release
$ time cargo test --release
...
real    0m1.275s
...
```

</td></tr></table>

Here is a table comparing model checking times on the author's laptop:

| #RM | TLC   | Stateright | Speedup |
|-----|-------|------------|---------|
| 7   |   3 s |    0.372 s |     ~8X |
| 8   |  12 s |    1.275 s |     ~9X |
| 9   |  90 s |    7.786 s |    ~11X |
