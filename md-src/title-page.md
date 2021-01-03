> ![links](https://img.shields.io/badge/Library_Links:-gray)
[![chat](https://img.shields.io/discord/781357978652901386)](https://discord.gg/JbxGSVP4A6)
[![crates.io](https://img.shields.io/crates/d/stateright.svg)](https://crates.io/crates/stateright)
[![docs.rs](https://docs.rs/stateright/badge.svg)](https://docs.rs/stateright)
[![stars](https://img.shields.io/github/stars/stateright/stateright?style=social)](https://github.com/stateright/stateright/stargazers)
[![watchers](https://img.shields.io/github/watchers/stateright/stateright?style=social)](https://github.com/stateright/stateright/watchers)

# Building Distributed Systems With Stateright

*by Jonathan Nadal*

> Deep understanding of causality sometimes requires the understanding of very
> large patterns and their abstract relationships and interactions, not just the
> understanding of microscopic objects interacting in microscopic time intervals.
>
>  ― Douglas R. Hofstadter, *I Am a Strange Loop*

Stateright is a model checker for code written in the Rust programming language. The name refers to
its goal of verifying that a system's overall state is always within specified correctness
conditions.

This book explains how to use Stateright to verify the correctness of distributed system
implementations, wherein nondeterminism manifests from events such as process crashes, network
partitions, and clock drift. Model checking techniques have helped web service providers such as
[Amazon and Microsoft](https://lamport.azurewebsites.net/tla/industrial-use.html) verify that
distributed system designs are resilient to this nondeterminism, and Stateright goes further by
enabling runnable implementations to be verified.
