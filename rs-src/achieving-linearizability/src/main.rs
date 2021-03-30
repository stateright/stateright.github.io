//! Provides a linearizable register "shared memory" abstraction that can serve requests as long as
//! a quorum of actors is available  (e.g. 3 of 5). This code is based on the algorithm described
//! in "[Sharing Memory Robustly in Message-Passing
//! Systems](https://doi.org/10.1145/200836.200869)" by Attiya, Bar-Noy, and Dolev. "ABD" in the
//! types refers to the author names.
//!
//! For a succinct overview of the algorithm, I recommend:
//! http://muratbuffalo.blogspot.com/2012/05/replicatedfault-tolerant-atomic-storage.html

/* ANCHOR: all */
use serde::{Deserialize, Serialize};
use stateright::actor::{*, register::*};
use stateright::util::{HashableHashMap, HashableHashSet};
use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;
use std::net::{SocketAddrV4, Ipv4Addr};

// ANCHOR: actor-msg
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
enum AbdMsg {
    Query(TestRequestId),
    AckQuery(TestRequestId, Seq, TestValue),
    Replicate(TestRequestId, Seq, TestValue),
    AckReplicate(TestRequestId),
}
// ANCHOR_END: actor-msg
use AbdMsg::*;

// ANCHOR: actor-state
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct AbdState {
    seq: Seq,
    val: TestValue,
    phase: Option<AbdPhase>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum AbdPhase {
    Phase1 {
        request_id: TestRequestId,
        requester_id: Id,
        write: Option<TestValue>, // `None` for read
        responses: HashableHashMap<Id, (Seq, TestValue)>,
    },
    Phase2 {
        request_id: TestRequestId,
        requester_id: Id,
        read: Option<TestValue>, // `None` for write
        acks: HashableHashSet<Id>,
    },
}

type Seq = (WriteCount, Id); // `Id` for uniqueness
type WriteCount = u64;
// ANCHOR_END: actor-state

// ANCHOR: actor
#[derive(Clone)]
struct AbdActor {
    peers: Vec<Id>,
}

impl Actor for AbdActor {
    type Msg = RegisterMsg<TestRequestId, TestValue, AbdMsg>;
    type State = AbdState;

    fn on_start(&self, _id: Id, _o: &mut Out<Self>) -> Self::State {
        AbdState {
            seq: (0, Id::from(0)),
            val: '?',
            phase: None,
        }
    }

    fn on_msg(&self, id: Id, state: &mut Cow<Self::State>,
              src: Id, msg: Self::Msg, o: &mut Out<Self>) {
        use RegisterMsg::*;
        match msg {
            Put(req_id, val) if state.phase.is_none() => {
                o.broadcast(&self.peers, &Internal(Query(req_id)));
                state.to_mut().phase = Some(AbdPhase::Phase1 {
                    request_id: req_id,
                    requester_id: src,
                    write: Some(val),
                    responses: {
                        let mut responses = HashableHashMap::default();
                        responses.insert(id, (state.seq, state.val.clone()));
                        responses
                    },
                });
            }
            Get(req_id) if state.phase.is_none() => {
                o.broadcast(&self.peers, &Internal(Query(req_id)));
                state.to_mut().phase = Some(AbdPhase::Phase1 {
                    request_id: req_id,
                    requester_id: src,
                    write: None,
                    responses: {
                        let mut responses = HashableHashMap::default();
                        responses.insert(id, (state.seq, state.val.clone()));
                        responses
                    },
                });
            }
            Internal(Query(req_id)) => {
                o.send(
                    src,
                    Internal(AckQuery(req_id, state.seq, state.val.clone())));
            }
            Internal(AckQuery(expected_req_id, seq, val))
                if matches!(state.phase,
                            Some(AbdPhase::Phase1 { request_id, .. })
                            if request_id == expected_req_id) =>
            {
                let mut state = state.to_mut();
                if let Some(AbdPhase::Phase1 {
                    request_id: req_id,
                    requester_id: requester,
                    write,
                    responses,
                    ..
                }) = &mut state.phase {
                    responses.insert(src, (seq, val));
                    if responses.len() == majority(self.peers.len() + 1) {
                        // Quorum reached. Move to phase 2.

                        // Determine sequencer and value.
                        let (_, (seq, val)) = responses.into_iter()
                            .max_by_key(|(_, (seq, _))| seq)
                            .unwrap();
                        let mut seq = *seq;
                        let mut read = None;
                        let val = if let Some(val) = std::mem::take(write) {
                            seq = (seq.0 + 1, id);
                            val
                        } else {
                            read = Some(val.clone());
                            val.clone()
                        };

                        o.broadcast(
                            &self.peers,
                            &Internal(Replicate(*req_id, seq, val.clone())));

                        state.seq = seq;
                        state.val = val;

                        let mut acks = HashableHashSet::default();
                        acks.insert(id);

                        state.phase = Some(AbdPhase::Phase2 {
                            request_id: *req_id,
                            requester_id: std::mem::take(requester),
                            read,
                            acks,
                        });
                    }
                }
            }
            Internal(Replicate(req_id, seq, val)) => {
                o.send(src, Internal(AckReplicate(req_id)));
                if seq > state.seq {
                    let mut state = state.to_mut();
                    state.seq = seq;
                    state.val = val;
                }
            }
            Internal(AckReplicate(expected_req_id))
                if matches!(state.phase,
                            Some(AbdPhase::Phase2 { request_id, .. })
                            if request_id == expected_req_id) =>
            {
                let mut state = state.to_mut();
                if let Some(AbdPhase::Phase2 {
                    request_id: req_id,
                    requester_id: requester,
                    read,
                    acks,
                    ..
                }) = &mut state.phase {
                    acks.insert(src);
                    if acks.len() == majority(self.peers.len() + 1) {
                        let msg = if let Some(val) = read {
                            GetOk(*req_id, std::mem::take(val))
                        } else {
                            PutOk(*req_id)
                        };
                        o.send(*requester, msg);
                        state.phase = None;
                    }
                }
            }
            _ => {}
        }
    }
}
// ANCHOR_END: actor

#[cfg(test)]
#[test]
fn is_linearizable() {
    use stateright::{Checker, Model};

    // ANCHOR: test
    let checker = RegisterCfg {
            servers: vec![
                AbdActor { peers: model_peers(0, 2) },
                AbdActor { peers: model_peers(1, 2) },
            ],
            client_count: 2,
        }
        .into_model()
        .duplicating_network(DuplicatingNetwork::No)
        .within_boundary(|_, state| {
            state.actor_states.iter().all(|s| {
                if let RegisterActorState::Server(s) = &**s {
                    s.seq.0 <= 3
                } else {
                    true
                }
            })
        })
        .checker().spawn_dfs().join();
    checker.assert_properties();
    // ANCHOR_END: test
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().default_filter_or("info"));
    let id0 = Id::from(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000));
    let id1 = Id::from(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3001));
    let id2 = Id::from(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3002));
    spawn(
        serde_json::to_vec,
        |bytes| serde_json::from_slice(bytes),
        vec![
            (id0, AbdActor { peers: vec![id1, id2] }),
            (id1, AbdActor { peers: vec![id0, id2] }),
            (id2, AbdActor { peers: vec![id0, id1] }),
        ]).unwrap();
}
/* ANCHOR_END: all */
