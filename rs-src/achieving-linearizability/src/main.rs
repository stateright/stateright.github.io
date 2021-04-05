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
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::net::{SocketAddrV4, Ipv4Addr};

// ANCHOR: actor-msg
type RequestId = u64;
type Value = char;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
enum AbdMsg {
    Query(RequestId),
    AckQuery(RequestId, Seq, Value),
    Replicate(RequestId, Seq, Value),
    AckReplicate(RequestId),
}
// ANCHOR_END: actor-msg
use AbdMsg::*;

// ANCHOR: actor-state
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct AbdState {
    seq: Seq,
    val: Value,
    phase: Option<AbdPhase>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum AbdPhase {
    Phase1 {
        request_id: RequestId,
        requester_id: Id,
        write: Option<Value>, // `None` for read
        responses: BTreeMap<Id, (Seq, Value)>,
    },
    Phase2 {
        request_id: RequestId,
        requester_id: Id,
        read: Option<Value>, // `None` for write
        acks: BTreeSet<Id>,
    },
}

type Seq = (LogicalClock, Id); // `Id` for uniqueness
type LogicalClock = u64;
// ANCHOR_END: actor-state

// ANCHOR: actor
#[derive(Clone)]
struct AbdActor {
    peers: Vec<Id>,
}

impl Actor for AbdActor {
    type Msg = RegisterMsg<RequestId, Value, AbdMsg>;
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
                        let mut responses = BTreeMap::default();
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
                        let mut responses = BTreeMap::default();
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
                            .max_by_key(|(_, (seq, _))| *seq)
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

                        // Self-send `Replicate`.
                        if seq > state.seq {
                            state.seq = seq;
                            state.val = val;
                        }

                        // Self-send `AckReplicate`.
                        let mut acks = BTreeSet::default();
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
                            Some(AbdPhase::Phase2 { request_id, ref acks, .. })
                            if request_id == expected_req_id && !acks.contains(&src)) =>
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
mod test {
    use super::*;
    use stateright::{*, semantics::*, semantics::register::*};

    // ANCHOR: test
    #[test]
    fn is_linearizable_quick() {
        let checker = base_model()
            .actor(RegisterActor::Server(AbdActor {
                peers: Id::vec_from(vec![1]),
            }))
            .actor(RegisterActor::Server(AbdActor {
                peers: Id::vec_from(vec![0]),
            }))
            .actor(RegisterActor::Client { put_count: 1, server_count: 2 })
            .actor(RegisterActor::Client { put_count: 1, server_count: 2 })
            .checker().threads(num_cpus::get()).spawn_dfs().join();
        checker.assert_properties();
        assert_eq!(checker.generated_count(), 544);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "enabled for --release only")]
    fn is_linearizable() {
        let checker = base_model()
            .actor(RegisterActor::Server(AbdActor {
                peers: Id::vec_from(vec![1, 2]),
            }))
            .actor(RegisterActor::Server(AbdActor {
                peers: Id::vec_from(vec![0, 2]),
            }))
            .actor(RegisterActor::Server(AbdActor {
                peers: Id::vec_from(vec![0, 1]),
            }))
            .actor(RegisterActor::Client { put_count: 1, server_count: 2 })
            .actor(RegisterActor::Client { put_count: 1, server_count: 2 })
            .checker().threads(num_cpus::get()).spawn_dfs().join();
        checker.assert_properties();
        assert_eq!(checker.generated_count(), 37_168_889);
    }

    fn base_model()
        -> ActorModel<
            RegisterActor<AbdActor>,
            (),
            LinearizabilityTester<Id, Register<char>>>
    {
        ActorModel::new(
                (),
                LinearizabilityTester::new(Register('?'))
            )
            .duplicating_network(DuplicatingNetwork::No)
            .property(Expectation::Always, "linearizable", |_, state| {
                state.history.serialized_history().is_some()
            })
            .property(Expectation::Sometimes, "value chosen", |_, state| {
                state.network.iter().any(|e| {
                    if let RegisterMsg::GetOk(_, value) = e.msg {
                        return value != '?';
                    }
                    return false
                })
            })
            .record_msg_in(RegisterMsg::record_returns)
            .record_msg_out(RegisterMsg::record_invocations)
    }
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
