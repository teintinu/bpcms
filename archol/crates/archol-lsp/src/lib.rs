//! Servidor LSP do Archol sobre stdio (JSON-RPC próprio, sem runtime async).

mod features;
mod resolve;
mod rpc;
mod server;
mod util;

pub fn run_stdio() -> i32 {
    server::Server::new().run()
}
