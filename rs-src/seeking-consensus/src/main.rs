/* ANCHOR: all */

use serde::{Deserialize, Serialize};
use stateright::actor::{*, register::*};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::net::{SocketAddrV4, Ipv4Addr};

type RequestId = u64;

#[derive(Clone)]
struct ActorContext {
    peer_ids: BTreeSet<Id>,
}

// ANCHOR: actor-msg
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[derive(Deserialize, Serialize)]
enum InternalMsg {
    Replicate(RequestId, char),
    ReplicateOk(RequestId),
}
// ANCHOR_END: actor-msg

// ANCHOR: actor-state
#[derive(Clone, Debug, Hash, PartialEq)]
struct ActorState {
    value: char,
    delivered: BTreeSet<(Id, RequestId)>,
    in_flight_put: Option<PutState>,
}

#[derive(Clone, Debug, Hash, PartialEq)]
struct PutState {
    req_id: RequestId,
    src: Id,
    peer_acks: BTreeSet<Id>,
}
// ANCHOR_END: actor-state

impl Actor for ActorContext {
    type Msg = RegisterMsg<RequestId, char, InternalMsg>;
    type State = ActorState;

    fn on_start(&self, _id: Id, _o: &mut Out<Self>) -> Self::State {
        ActorState {
            value: '?',
            delivered: Default::default(),
            in_flight_put: None,
        }
    }

    // ANCHOR: actor
    fn on_msg(&self, _id: Id, state: &mut Cow<Self::State>,
              src: Id, msg: Self::Msg, o: &mut Out<Self>) {
        match msg {
            RegisterMsg::Put(req_id, value) if state.in_flight_put.is_none() => {
                if state.delivered.contains(&(src, req_id)) { return }

                let mut state = state.to_mut();
                state.value = value;
                state.delivered.insert((src, req_id));
                state.in_flight_put = Some(PutState {
                    req_id,
                    src,
                    peer_acks: Default::default(),
                });
                for &peer_id in &self.peer_ids {
                    o.send(peer_id,
                           RegisterMsg::Internal(
                               InternalMsg::Replicate(req_id, value)));
                }
                // Will not reply w/ `PutOk` until all replicas ack.
            }
            RegisterMsg::Get(req_id) => {
                o.send(src, RegisterMsg::GetOk(req_id, state.value));
            }
            RegisterMsg::Internal(InternalMsg::Replicate(req_id, value)) => {
                if state.delivered.contains(&(src, req_id)) { return }

                let mut state = state.to_mut();
                state.value = value;
                state.delivered.insert((src, req_id));
                o.send(src,
                       RegisterMsg::Internal(InternalMsg::ReplicateOk(req_id)));
            }
            RegisterMsg::Internal(InternalMsg::ReplicateOk(req_id)) => {
                if state.delivered.contains(&(src, req_id)) { return }

                let mut state = state.to_mut();
                if let Some(put) = &mut state.in_flight_put {
                    if req_id != put.req_id { return }

                    put.peer_acks.insert(src);
                    if put.peer_acks == self.peer_ids {
                        o.send(put.src, RegisterMsg::PutOk(req_id));
                        state.in_flight_put = None;
                    }
                }
            }
            _ => {}
        }
    }
    // ANCHOR_END: actor
}

#[cfg(test)]
mod test {
    use super::*;
    use stateright::*;
    use InternalMsg::{Replicate, ReplicateOk};
    use RegisterMsg::{Get, GetOk, Internal, Put, PutOk};
    use SystemAction::Deliver;

    #[test]
    fn appears_linearizable_in_limited_scenarios() {
        // Succeeds if there are 2 clients.
        let checker = RegisterTestSystem {
            servers: vec![
                ActorContext { peer_ids: vec![Id::from(1)].into_iter().collect() },
                ActorContext { peer_ids: vec![Id::from(0)].into_iter().collect() },
            ],
            client_count: 2,
            .. Default::default()
        }.into_model().checker().spawn_bfs().join();
        checker.assert_properties();
    }

    #[test]
    fn not_generally_linearizable() {
        // ANCHOR: test
        // Fails if there are 3 clients.
        let checker = RegisterTestSystem {
            servers: vec![
                ActorContext { peer_ids: vec![Id::from(1)].into_iter().collect() },
                ActorContext { peer_ids: vec![Id::from(0)].into_iter().collect() },
            ],
            client_count: 3,
            .. Default::default()
        }.into_model().checker()
        .spawn_dfs().join();     // TRY IT: Comment out this line, and uncomment
        //.serve("0:3000");      //         the next to load Stateright Explorer.
        //checker.assert_properties(); // TRY IT: Uncomment this line, and the test will fail.
        checker.assert_discovery("linearizable", vec![
            Deliver { src: Id::from(4), dst: Id::from(0), msg: Put(4, 'C') },
            Deliver { src: Id::from(0), dst: Id::from(1), msg: Internal(Replicate(4, 'C')) },
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Internal(ReplicateOk(4)) },
            Deliver { src: Id::from(3), dst: Id::from(1), msg: Put(3, 'B') },
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Internal(Replicate(3, 'B')) },
            Deliver { src: Id::from(0), dst: Id::from(1), msg: Internal(ReplicateOk(3)) },
            Deliver { src: Id::from(1), dst: Id::from(3), msg: PutOk(3) },
            Deliver { src: Id::from(2), dst: Id::from(0), msg: Put(2, 'A') },
            Deliver { src: Id::from(3), dst: Id::from(0), msg: Get(6) },
            Deliver { src: Id::from(0), dst: Id::from(3), msg: GetOk(6, 'A') },
            Deliver { src: Id::from(0), dst: Id::from(4), msg: PutOk(4) },
            Deliver { src: Id::from(4), dst: Id::from(1), msg: Get(8) },
            Deliver { src: Id::from(1), dst: Id::from(4), msg: GetOk(8, 'B') },
        ]);
        // ANCHOR_END: test
    }
}

// Running the program spawns actors on UDP ports 3000-3002. Messages are JSON-serialized.
fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let id0 = Id::from(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000));
    let id1 = Id::from(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3001));
    let id2 = Id::from(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3002));
    let handles = spawn(
        serde_json::to_vec,
        |bytes| serde_json::from_slice(bytes),
        vec![
            (id0, ActorContext { peer_ids: vec![id1, id2].into_iter().collect() } ),
            (id1, ActorContext { peer_ids: vec![id0, id2].into_iter().collect() } ),
            (id2, ActorContext { peer_ids: vec![id0, id1].into_iter().collect() } ),
        ]);
    for h in handles { let _ = h.join(); }
}

/* ANCHOR_END: all */
