/* ANCHOR: all */
use stateright::actor::{*, register::*};
use std::borrow::Cow; // COW == clone-on-write
use std::net::{SocketAddrV4, Ipv4Addr};

// ANCHOR: actor
type RequestId = u64;

#[derive(Clone)]
struct ActorContext;

impl Actor for ActorContext {
    type Msg = RegisterMsg<RequestId, char, ()>;
    type State = char;

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
    use stateright::*;
    use RegisterMsg::{Get, GetOk, Put, PutOk};
    use SystemAction::Deliver;

    // ANCHOR: test
    #[test]
    fn is_unfortunately_not_linearizable() {
        let checker = RegisterTestSystem {
            servers: vec![ActorContext],
            client_count: 1,
            .. Default::default()
        }.into_model().checker().spawn_dfs().join();
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
            (SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000), ActorContext)
        ]).unwrap();
}
// ANCHOR_END: main
/* ANCHOR_END: all */
