> ![links](https://img.shields.io/badge/Library_Links:-gray)
[![chat](https://img.shields.io/discord/781357978652901386)](https://discord.gg/JbxGSVP4A6)
[![crates.io](https://img.shields.io/crates/d/stateright.svg)](https://crates.io/crates/stateright)
[![docs.rs](https://docs.rs/stateright/badge.svg)](https://docs.rs/stateright)
[![stars](https://img.shields.io/github/stars/stateright/stateright?style=social)](https://github.com/stateright/stateright/stargazers)
[![youtube](https://img.shields.io/youtube/views/Oh0Dmz6asbs?style=social)](https://youtube.com/playlist?list=PLUhyBsVvEJjaF1VpNhLRfIA4E7CFPirmz)

# Building Distributed Systems With Stateright

*by Jonathan Nadal*

> Deep understanding of causality sometimes requires the understanding of very
> large patterns and their abstract relationships and interactions, not just the
> understanding of microscopic objects interacting in microscopic time intervals.
>
>  ― Douglas R. Hofstadter, *I Am a Strange Loop*

Distributed computing is a term that refers to multiple computers working
together *as a system* to solve a problem, typically because that problem would
not be solvable on a single computer. For example, we all want to know that our
important files will be accessible even when computer hardware inevitably
fails. As a second example, a researcher at a pharmaceutical company may have a
complex problem that would take decades for a single computer to solve, but
which a collection of computers working together could solve in days.

Unique algorithms must be employed to coordinate workloads across these
geographically distributed systems of computers because they are susceptible to
categories of nondeterminism that do not arise when a problem is solved with a
single computer. For example, the networks that link these computers will drop,
reorder, and even redeliver messages. Algorithms that fail to account for this
behavior may run correctly for extended periods but will eventually fail at
unpredicatable times in unpredictable ways, such as causing data corruption.

Stateright is a software framework for analyzing and systematically verifying
distributed systems. Its name refers to the goal of verifying that a system's
collective state always satisfies a correctness specification, such as "any
data written to the system should be accessible as long as at least one data
center is reachable."

Cloud service providers like AWS and Azure leverage verification software such
as [the TLA+ model
checker](https://lamport.azurewebsites.net/tla/industrial-use.html) to achieve
the same goal, but whereas those solutions typically verify a high level system
design, Stateright is able to verify the underlying system *implementation* in
addition to the design.

We'll jump right in with a motivating example in the first chapter, [Getting
Started](./getting-started.md). Please see the [Stateright YouTube
channel](https://www.youtube.com/playlist?list=PLUhyBsVvEJjaF1VpNhLRfIA4E7CFPirmz)
if you prefer starting with a video introduction.
