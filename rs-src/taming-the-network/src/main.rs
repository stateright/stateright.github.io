/* ANCHOR: all */

use stateright::actor::{*, register::*};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::net::{SocketAddrV4, Ipv4Addr};

#[derive(Clone)]
struct ServerActor;

// ANCHOR: actor
type RequestId = u64;

#[derive(Clone, Debug, Hash, PartialEq)]
struct ActorState {
    value: char,
    delivered: BTreeSet<(Id, RequestId)>,
}

impl Actor for ServerActor {
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
    use stateright::{*, semantics::*, semantics::register::*};
    use ActorModelAction::Deliver;
    use RegisterMsg::{Get, GetOk, Put, PutOk};

    // ANCHOR: test
    #[test]
    fn satisfies_all_properties() {
        // Works with 1 client.
        base_model()
            .actor(RegisterActor::Server(ServerActor))
            .actor(RegisterActor::Client { put_count: 2, server_count: 1 })
            .checker().spawn_dfs().join()
            .assert_properties();

        // Or with multiple clients.
        // (TIP: test with `--release` mode for more clients)
        base_model()
            .actor(RegisterActor::Server(ServerActor))
            .actor(RegisterActor::Client { put_count: 1, server_count: 1 })
            .actor(RegisterActor::Client { put_count: 1, server_count: 1 })
            .checker().spawn_dfs().join()
            .assert_properties();
    }

    #[test]
    fn not_linearizable_with_two_servers() {
        let checker = base_model()
            .actor(RegisterActor::Server(ServerActor))
            .actor(RegisterActor::Server(ServerActor))
            .actor(RegisterActor::Client { put_count: 2, server_count: 2 })
            .checker().spawn_dfs().join();
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

    // ANCHOR: test-model-fn
    fn base_model()
        -> ActorModel<
            RegisterActor<ServerActor>,
            (),
            LinearizabilityTester<Id, Register<char>>>
    {
        ActorModel::new(
                (),
                LinearizabilityTester::new(Register('?'))
            )
            .property(Expectation::Always, "linearizable", |_, state| {
                state.history.serialized_history().is_some()
            })
            .property(Expectation::Sometimes, "get succeeds", |_, state| {
                state.network.iter_deliverable()
                    .any(|e| matches!(e.msg, RegisterMsg::GetOk(_, _)))
            })
            .property(Expectation::Sometimes, "put succeeds", |_, state| {
                state.network.iter_deliverable()
                    .any(|e| matches!(e.msg, RegisterMsg::PutOk(_)))
            })
            .record_msg_in(RegisterMsg::record_returns)
            .record_msg_out(RegisterMsg::record_invocations)
    }
    // ANCHOR_END: test-model-fn
}

// Running the program spawns a single actor on UDP port 3000. Messages are JSON-serialized.
fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    spawn(
        serde_json::to_vec,
        |bytes| serde_json::from_slice(bytes),
        vec![
            (SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000), ServerActor)
        ]).unwrap();
}

/* ANCHOR_END: all */
