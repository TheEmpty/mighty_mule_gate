mod gate;
mod service_configuration;
mod server;

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use gate::Gate;

#[tokio::main]
async fn main() {
    let conf = service_configuration::load();
    unsafe {
        server::GATE = Some(Gate::new(conf.gate_configuration));
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], conf.server_port));
    let service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(server::handle))
    });
    let server = Server::bind(&addr).serve(service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
