extern crate mio;
#[macro_use]
extern crate log;

#[cfg(test)]
extern crate env_logger;

mod buffer;
mod looper;

/*
#[test]
fn it_works() {
use mio::*;
use mio::tcp::{TcpListener, TcpStream};

// Setup some tokens to allow us to identify which event is
// for which socket.
const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);

let addr = "127.0.0.1:13265".parse().unwrap();

// Setup the server socket
let server = TcpListener::bind(&addr).unwrap();

// Create an event loop
let mut event_loop = EventLoop::new().unwrap();

// Start listening for incoming connections
event_loop.register(&server, SERVER, EventSet::readable(),
                    PollOpt::edge()).unwrap();

// Setup the client socket
let sock = TcpStream::connect(&addr).unwrap();

// Register the socket
event_loop.register(&sock, CLIENT, EventSet::readable(),
                    PollOpt::edge()).unwrap();

// Define a handler to process the events
struct MyHandler(TcpListener);

impl Handler for MyHandler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<MyHandler>, token: Token, _: EventSet) {
        match token {
            SERVER => {
                let MyHandler(ref mut server) = *self;
                // Accept and drop the socket immediately, this will close
                // the socket and notify the client of the EOF.
                let conn = server.accept();
            }
            CLIENT => {
                // The server just shuts down the socket, let's just
                // shutdown the event loop
                event_loop.shutdown();
            }
            _ => panic!("unexpected token"),
        }
    }
}

struct NoHandler;
impl Handler for NoHandler {
    type Timeout = ();
    type Message = ();
    fn ready(&mut self, event_loop: &mut EventLoop<NoHandler>, token: Token, _: EventSet) {
    }
}

// Start handling events
event_loop.run(&mut MyHandler(server)).unwrap();
}
*/
