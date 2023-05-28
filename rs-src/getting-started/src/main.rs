/* ANCHOR: all */
use stateright::actor::{*, register::*};
use std::borrow::Cow; // COW == clone-on-write
use std::net::{SocketAddrV4, Ipv4Addr};

// ANCHOR: actor
type RequestId = u64;

#[derive(Clone)]
struct ServerActor;

impl Actor for ServerActor {
    type Msg = RegisterMsg<RequestId, char, ()>;
    type State = char;
    type Timer = ();

    fn on_start(&self, _id: Id, _o: &mut Out<Self>) -> Self::State {
        '?' // default value for the register
    }

    fn on_msg(&self, _id: Id, state: &mut Cow<Self::State>,
              src: Id, msg: Self::Msg, o: &mut Out<Self>) {
        match msg {
            RegisterMsg::Put(req_id, value) => {
                *state.to_mut() = value;
                o.send(src, RegisterMsg::PutOk(req_id));
            }
            RegisterMsg::Get(req_id) => {
                o.send(src, RegisterMsg::GetOk(req_id, **state));
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
    fn is_unfortunately_not_linearizable() {
        let checker = ActorModel::new(
                (),
                LinearizabilityTester::new(Register('?'))
            )
            .actor(RegisterActor::Server(ServerActor))
            .actor(RegisterActor::Client { put_count: 2, server_count: 1 })
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
            .checker().spawn_dfs().join();
        //checker.assert_properties(); // TRY IT: Uncomment this line, and the test will fail.
        checker.assert_discovery("linearizable", vec![
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Put(1, 'A') },
            Deliver { src: Id::from(0), dst: Id::from(1), msg: PutOk(1) },
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Put(2, 'Z') },
            Deliver { src: Id::from(0), dst: Id::from(1), msg: PutOk(2) },
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Put(1, 'A') },
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Get(3) },
            Deliver { src: Id::from(0), dst: Id::from(1), msg: GetOk(3, 'A') },
        ]);
    }
    // ANCHOR_END: test
}

// ANCHOR: main
fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    spawn(
        serde_json::to_vec,
        |bytes| serde_json::from_slice(bytes),
        vec![
            (SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000), ServerActor)
        ]).unwrap();
}
// ANCHOR_END: main
/* ANCHOR_END: all */
