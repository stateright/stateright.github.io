//! This module implements a subset of the two phase commit specification presented in the paper
//! ["Consensus on Transaction Commit"](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/tr-2003-96.pdf)
//! by Jim Gray and Leslie Lamport.

// ANCHOR: dependencies
use stateright::{Model, Property};
use std::collections::BTreeSet;
use std::hash::Hash;
use std::ops::Range;
// ANCHOR_END: dependencies

// ANCHOR: constants
type R = usize; // RM in 0..N

#[derive(Clone)]
struct TwoPhaseSys { pub rms: Range<R> }
// ANCHOR_END: constants

// ANCHOR: variables
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct TwoPhaseState {
    rm_state: Vec<RmState>,
    tm_state: TmState,
    tm_prepared: Vec<bool>,
    msgs: BTreeSet<Message>,
}
// ANCHOR_END: variables

// ANCHOR: types
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Message { Prepared { rm: R }, Commit, Abort }

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum RmState { Working, Prepared, Committed, Aborted }

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum TmState { Init, Committed, Aborted }
// ANCHOR_END: types

// ANCHOR: spec
#[derive(Clone, Debug)]
enum Action {
    TmRcvPrepared(R),
    TmCommit,
    TmAbort,
    RmPrepare(R),
    RmChooseToAbort(R),
    RmRcvCommitMsg(R),
    RmRcvAbortMsg(R),
}

impl Model for TwoPhaseSys {
    type State = TwoPhaseState;
    type Action = Action;
// ANCHOR_END: spec

// ANCHOR: init
fn init_states(&self) -> Vec<Self::State> {
    vec![TwoPhaseState {
        rm_state: self.rms.clone().map(|_| RmState::Working).collect(),
        tm_state: TmState::Init,
        tm_prepared: self.rms.clone().map(|_| false).collect(),
        msgs: Default::default(),
    }]
}
// ANCHOR_END: init

// ANCHOR: next
fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
    if state.tm_state == TmState::Init
            && state.tm_prepared.iter().all(|p| *p) {
        actions.push(Action::TmCommit);
    }
    if state.tm_state == TmState::Init {
        actions.push(Action::TmAbort);
    }
    for rm in self.rms.clone() {
        if state.tm_state == TmState::Init
                && state.msgs.contains(&Message::Prepared { rm }) {
            actions.push(Action::TmRcvPrepared(rm));
        }
        if state.rm_state.get(rm) == Some(&RmState::Working) {
            actions.push(Action::RmPrepare(rm));
        }
        if state.rm_state.get(rm) == Some(&RmState::Working) {
            actions.push(Action::RmChooseToAbort(rm));
        }
        if state.msgs.contains(&Message::Commit) {
            actions.push(Action::RmRcvCommitMsg(rm));
        }
        if state.msgs.contains(&Message::Abort) {
            actions.push(Action::RmRcvAbortMsg(rm));
        }
    }
}

fn next_state(&self, last_state: &Self::State, action: Self::Action)
        -> Option<Self::State> {
    let mut state = last_state.clone();
    match action {
        Action::TmRcvPrepared(rm) => {
            state.tm_prepared[rm] = true;
        }
        Action::TmCommit => {
            state.tm_state = TmState::Committed;
            state.msgs.insert(Message::Commit);
        }
        Action::TmAbort => {
            state.tm_state = TmState::Aborted;
            state.msgs.insert(Message::Abort);
        },
        Action::RmPrepare(rm) => {
            state.rm_state[rm] = RmState::Prepared;
            state.msgs.insert(Message::Prepared { rm });
        },
        Action::RmChooseToAbort(rm) => {
            state.rm_state[rm] = RmState::Aborted;
        }
        Action::RmRcvCommitMsg(rm) => {
            state.rm_state[rm] = RmState::Committed;
        }
        Action::RmRcvAbortMsg(rm) => {
            state.rm_state[rm] = RmState::Aborted;
        }
    }
    Some(state)
}
// ANCHOR_END: next

// ANCHOR: properties
fn properties(&self) -> Vec<Property<Self>> {
    vec![
        Property::<Self>::always("consistent", |_, state| {
           !state.rm_state.iter().any(|s1|
                state.rm_state.iter().any(|s2|
                    s1 == &RmState::Aborted && s2 == &RmState::Committed))
        }),
    ]
}
// ANCHOR_END: properties
}

// ANCHOR: configuration
#[cfg(test)]
#[test]
fn can_model_2pc() {
    use stateright::Checker;
    TwoPhaseSys { rms: 0..9 }.checker()
        .threads(num_cpus::get()).spawn_dfs().join()
    	.assert_properties();
}
// ANCHOR_END: configuration
