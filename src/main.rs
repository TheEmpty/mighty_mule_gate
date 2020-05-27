mod gate;
mod service_configuration;
mod server;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::thread;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use gate::Gate;
use log::{info, error, trace};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let conf = service_configuration::load();
    unsafe {
        server::GATE = Some(Gate::new(conf.gate_configuration));
        server::MAX_STATE_LOCK_TTL = Some(conf.max_state_lock_ttl);
    }

    thread::spawn(|| {
        loop {
            trace!("Calling gate sync from thread.");
            unsafe {
                server::GATE.as_mut().unwrap().sync();
            }
            thread::sleep(std::time::Duration::from_secs(3));
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], conf.server_port));
    let service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(server::handle))
    });

    info!("Accepting traffic on {}", conf.server_port);
    let server = Server::bind(&addr).serve(service);

    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}
