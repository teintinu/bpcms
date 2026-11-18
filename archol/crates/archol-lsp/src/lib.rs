//! Archol LSP server over stdio (custom JSON-RPC, no async runtime).

mod app;
mod features;
mod resolve;
mod rpc;
mod server;
mod util;

pub fn run_stdio() -> i32 {
    server::Server::new().run()
}
