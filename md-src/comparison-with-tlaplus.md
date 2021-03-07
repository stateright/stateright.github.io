# Comparison with TLA+

The previous part of this book focused on model checking runnable actor
systems.  Stateright is also able to model check higher level designs via
"abstract models," much like a traditional model checker. This chapter compares
two abstract models of the [two-phase commit
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

[Lecture 6](https://lamport.azurewebsites.net/video/video6.html) of Leslie
Lamport's [TLA+ Video
Course](https://lamport.azurewebsites.net/video/videos.html) is recommended for
an additional overview of the specification.

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
  in the checker discoveries. You can simulate these in TLC by introducing
  false "invariants," but they need to be commented out and periodically run,
  which is more cumbersome.
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
  have indicated that they are preparing a transaction;
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

Now we get to the most interesting part of the model, the state transitions
(which Stateright calls
[actions](https://docs.rs/stateright/latest/stateright/trait.Model.html#tymethod.actions)).
TLA+ requires each transition relation to precede the aggregate next state
relation (`Next` in this case). Each action serves two roles: (1) it defines
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
file, while for Stateright you simply introduce a Rust test. Rust also requires
a `Cargo.toml` file.

<table><tr><td>

```ignore,noplayground
{{#include ../other-src/comparison-with-tlaplus/TwoPhase.cfg}}
```

</td><td>

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/src/lib.rs:configuration}}
```

```rust,ignore,noplayground
{{#include ../rs-src/comparison-with-tlaplus/Cargo.toml}}
```

</td></tr></table>

TLA+ can be run from the command line using a tool such as
[tla-bin](https://github.com/pmer/tla-bin) or
[tlacli](https://github.com/hwayne/tlacli), while the Stateright test is run
using `cargo test --release`. The example below first calls `build --tests` to
avoid timing dependency compilation but then revises the file timestamp to
include compilation time relevant to the development iteration cycle.
enables native compilation since the resulting binary does not need to be
distributed.

<table><tr><td>

```ignore,noplayground
$ tlc -workers auto TwoPhase.tla  
```

</td><td>

```ignore,noplayground
$ export RUSTFLAGS='-C target-cpu=native'  
$ cargo build --tests --release
$ touch src/lib.rs
$ time cargo test --release
```

</td></tr></table>

Here is a table comparing model checking times on the author's laptop:

| #RM | TLC   | Stateright | Speedup |
|-----|-------|------------|---------|
| 7   |   3 s |    1.697 s |    1.7X |
| 8   |  12 s |    2.566 s |    4.6X |
| 9   |  90 s |    8.902 s |   10.1X |
