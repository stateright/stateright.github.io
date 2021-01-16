/* ANCHOR: all */

use stateright::actor::{*, register::*};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::net::{SocketAddrV4, Ipv4Addr};

type RequestId = u64;

#[derive(Clone)]
struct ActorContext;

// ANCHOR: actor
#[derive(Clone, Debug, Hash, PartialEq)]
struct ActorState {
    value: char,
    delivered: BTreeSet<(Id, RequestId)>,
}

impl Actor for ActorContext {
    type Msg = RegisterMsg<RequestId, char, ()>;
    type State = ActorState;

    fn on_start(&self, _id: Id, _o: &mut Out<Self>) -> Self::State {
        ActorState {
            value: '?',
            delivered: Default::default(),
        }
    }

    fn on_msg(&self, _id: Id, state: &mut Cow<Self::State>,
              src: Id, msg: Self::Msg, o: &mut Out<Self>) {
        match msg {
            RegisterMsg::Put(req_id, value) => {
                if state.delivered.contains(&(src, req_id)) { return }

                let mut state = state.to_mut();
                state.value = value;
                state.delivered.insert((src, req_id));
                o.send(src, RegisterMsg::PutOk(req_id));
            }
            RegisterMsg::Get(req_id) => {
                o.send(src, RegisterMsg::GetOk(req_id, state.value));
            }
            _ => {}
        }
    }
}
// ANCHOR_END: actor

#[cfg(test)]
mod test {
    use super::*;
    use stateright::*;
    use RegisterMsg::{Get, GetOk, Put, PutOk};
    use SystemAction::Deliver;

    // ANCHOR: test
    #[test]
    fn satisfies_all_properties() {
        // Works with 1 client.
        let checker = RegisterTestSystem {
            servers: vec![ActorContext],
            client_count: 1,
            .. Default::default()
        }.into_model().checker().spawn_dfs().join();
        checker.assert_properties();

        // Or with multiple clients.
        let checker = RegisterTestSystem {
            servers: vec![ActorContext],
            client_count: 2, // TIP: test with `--release` mode for more clients
            .. Default::default()
        }.into_model().checker().spawn_dfs().join();
        checker.assert_properties();
    }

    #[test]
    fn not_linearizable_with_two_servers() {
        let checker = RegisterTestSystem {
            servers: vec![ActorContext, ActorContext], // two servers
            client_count: 1,
            .. Default::default()
        }.into_model().checker().spawn_dfs().join();
        //checker.assert_properties(); // TRY IT: Uncomment this line, and the test will fail.
        checker.assert_discovery("linearizable", vec![
            Deliver { src: Id::from(2), dst: Id::from(0), msg: Put(2, 'A') },
            Deliver { src: Id::from(0), dst: Id::from(2), msg: PutOk(2) },
            Deliver { src: Id::from(2), dst: Id::from(1), msg: Put(4, 'Z') },
            Deliver { src: Id::from(1), dst: Id::from(2), msg: PutOk(4) },
            Deliver { src: Id::from(2), dst: Id::from(0), msg: Get(6) },
            Deliver { src: Id::from(0), dst: Id::from(2), msg: GetOk(6, 'A') },
        ]);
    }
    // ANCHOR_END: test
}

// Running the program spawns a single actor on UDP port 3000. Messages are JSON-serialized.
fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let handles = spawn(
        serde_json::to_vec,
        |bytes| serde_json::from_slice(bytes),
        vec![
            (SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000), ActorContext)
        ]);
    for h in handles { let _ = h.join(); }
}

/* ANCHOR_END: all */
