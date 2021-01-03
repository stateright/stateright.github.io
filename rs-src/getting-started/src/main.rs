/* ANCHOR: all */

use stateright::actor::{*, register::*};
use std::net::{SocketAddrV4, Ipv4Addr};

// Here we have an actor that represents a "virtual register." As you would expect, `Put` messages
// indicate a value to store, while `Get` messages return the last stored value. Response messages
// are associated with request messages via an integer "request ID" that is expected to be unique.
// The Stateright library provides a `RegisterMsg` type for this common pattern.
// ANCHOR: actor

type RequestId = u64;

#[derive(Clone)]
struct ActorContext;

impl Actor for ActorContext {
    type Msg = RegisterMsg<RequestId, char, ()>;
    type State = char;

    fn on_start(&self, _id: Id, o: &mut Out<Self>) {
        o.set_state('?'); // default value for the register
    }

    fn on_msg(&self, _id: Id, state: &Self::State,
              src: Id, msg: Self::Msg, o: &mut Out<Self>) {
        match msg {
            RegisterMsg::Put(req_id, value) => {
                o.set_state(value);
                o.send(src, RegisterMsg::PutOk(req_id));
            }
            RegisterMsg::Get(req_id) => {
                o.send(src, RegisterMsg::GetOk(req_id, *state));
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

    // It turns out that even this minimal example of an actor system has a subtle bug, and this is
    // a category of bug present in many (if not *most*) distributed systems. The bug can manifest
    // even if there is a single server and a single client.
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
            // Here is a counterexample that Stateright will confirm. Each step indicates an
            // incoming event observed by an actor For brevity counterexamples show actor inputs
            // such as `Deliver` but not outputs such as `Send`.
            //
            // 1. The server receives a write from the client (which it acknowledges).
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Put(1, 'A') },
            // 2. The client receives the ack (and sends a second write with a new value).
            Deliver { src: Id::from(0), dst: Id::from(1), msg: PutOk(1) },
            // 3. The server receives the second write (which it acknowledges).
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Put(2, 'Z') },
            // 4. The network redelivers the first write, inadvertently overwriting the second.
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Put(1, 'A') },
            // 5. The client receives the ack for the second write (and sends a read request).
            Deliver { src: Id::from(0), dst: Id::from(1), msg: PutOk(2) },
            // 6. The server receives the read request (and replies with the current value).
            Deliver { src: Id::from(1), dst: Id::from(0), msg: Get(3) },
            // 7. The client receives a value that violates linearizability. QED.
            Deliver { src: Id::from(0), dst: Id::from(1), msg: GetOk(3, 'A') },
        ]);
    }
    // ANCHOR_END: test
}

// Running the program spawns a single actor on UDP port 3000. Messages are JSON-serialized.
// ANCHOR: main
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
// ANCHOR_END: main

/* ANCHOR_END: all */
