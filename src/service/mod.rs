mod bufwrite;
mod buffer;
mod looper;
mod stream;
mod listen;
#[macro_use]
mod service;

#[cfg(test)]
mod test;

pub use self::service::ServiceRef;
pub use self::service::ServiceConfig;
pub use self::service::ServiceStreamer;
pub use self::service::ServiceHandler;
pub use self::bufwrite::BufWrite;
pub use mio::Token;

use self::looper::LoopHandler;
pub fn run_loop() {
    let mut handler = LoopHandler;
    handler.run();
}

