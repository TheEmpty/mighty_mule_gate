mod gate;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use std::collections::HashMap;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use http::header::HeaderValue;
use gate::Gate;
use gate::GateConfiguration;

static SERVER_PORT: u16 = 3005;

static mut GATE: Gate = Gate {
    configuration: GateConfiguration {
        time_to_move: std::time::Duration::from_secs(5),
        time_held_open: std::time::Duration::from_secs(15)
    },
    current_state: gate::State::CLOSED
};

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let params: HashMap<String, String> = req
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            unsafe {
                let gate_json = serde_json::to_string(&GATE).unwrap();
                let body = Body::from(gate_json);
                let mut response = Response::new(body);
                response.headers_mut().insert("Content-Type",  HeaderValue::from_str("application/json").unwrap());
                return Ok(response);
            }
        },

        (&Method::POST, "/state") => {
            unsafe {
                let state = gate::State::from_str(params.get("desired_state").unwrap()).unwrap();
                thread::spawn(|| {
                    GATE.change_state(state);
                });
                let body = Body::from("{}");
                let mut response = Response::new(body);
                response.headers_mut().insert("Content-Type",  HeaderValue::from_str("application/json").unwrap());
                return Ok(response);
            }
        }

        // 404 / catch-all
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            return Ok(not_found);
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
    let service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(router))
    });
    let server = Server::bind(&addr).serve(service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}