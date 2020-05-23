mod gate;
mod service_configuration;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use std::collections::HashMap;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use http::header::HeaderValue;
use gate::Gate;

static mut GATE: Option<Gate> = None;

// TODO: break this up
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
            let body = Body::from("Might Mule Gate API");
            let mut response = Response::new(body);
            response.headers_mut().insert("Content-Type", HeaderValue::from_str("text/plain").unwrap());
            return Ok(response);
        }

        (&Method::GET, "/gate") => {
            unsafe {
                let gate = GATE.as_mut().unwrap();
                gate.clear_expired_locks();
                let gate_json = serde_json::to_string(&GATE).unwrap();
                let body = Body::from(gate_json);
                let mut response = Response::new(body);
                response.headers_mut().insert("Content-Type",  HeaderValue::from_str("application/json").unwrap());
                return Ok(response);
            }
        },

        (&Method::POST, "/gate") => {
            unsafe {
                let mut operation = false;

                let desired_state_param = params.get("desired_state");
                if desired_state_param.is_some() {
                    operation = true;
                    let desired_state = gate::State::from_str(desired_state_param.unwrap());
                    if desired_state.is_ok() {
                        thread::spawn(move || {
                            GATE.as_mut().unwrap().change_state(desired_state.unwrap());
                        });
                    } else {
                        let body = Body::from(format!("{{\"error\": \"Invalid state, {}\"}}", desired_state_param.unwrap()));
                        let mut response = Response::new(body);
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                }

                let lock_state_param = params.get("lock_state");
                if lock_state_param.is_some() {
                    operation = true;
                    let lock_state = gate::State::from_str(lock_state_param.unwrap());
                    if lock_state.is_ok() {
                        // TODO: TTL is passed in and server_config has a max TTL setting
                        // Note: a real TTL would be like 15-60 minutes
                        GATE.as_mut().unwrap().hold_state(lock_state.unwrap(), std::time::Duration::from_secs(180));
                    } else {
                        let body = Body::from(format!("{{\"error\": \"Invalid state, {}\"}}", lock_state_param.unwrap()));
                        let mut response = Response::new(body);
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                }

                if operation == false {
                    let body = Body::from("{\"error\": \"no operation requested\"}");
                    let mut response = Response::new(body);
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }

                // TODO: don't do this async once we get rid of the Thread::Sleeping
                let body = Body::from(format!("{{\"success\": {}}}", "maybe"));
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
    let conf = service_configuration::load();
    unsafe {
        GATE = Some(Gate::new(conf.gate_configuration));
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], conf.server_port));
    let service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(router))
    });
    let server = Server::bind(&addr).serve(service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
