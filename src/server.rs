use std::convert::Infallible;
use std::str::FromStr;
use std::collections::HashMap;
use hyper::{Body, Method, Request, Response, StatusCode};
use http::header::HeaderValue;
use crate::gate::Gate;
use crate::gate;

pub static mut GATE: Option<Gate> = None;
pub static mut MAX_STATE_LOCK_TTL: Option<std::time::Duration> = None;

fn get_params(req: &Request<Body>) -> HashMap<String, String> {
    return req
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);
}

fn easy_json_response(json: String) -> Response<Body> {
    let body = Body::from(json);
    let mut response = Response::new(body);
    set_json_response_type(&mut response);
    return response;
}

fn set_json_response_type(response: &mut Response<Body>) -> () {
    response.headers_mut().insert("Content-Type",  HeaderValue::from_str("application/json").unwrap());
}

// GET /
fn get_root() -> Response<Body> {
    let body = Body::from("Might Mule Gate API");
    let mut response = Response::new(body);
    response.headers_mut().insert("Content-Type", HeaderValue::from_str("text/plain").unwrap());
    return response;
}

// GET /gate
fn get_gate() -> Response<Body> {
    let gate_json: String;
    unsafe {
        let gate = GATE.as_mut().unwrap();
        gate.clear_expired_locks();
        gate_json = serde_json::to_string(&GATE).unwrap();
    };

    let body = Body::from(gate_json);
    let mut response = Response::new(body);
    set_json_response_type(&mut response);
    return response;
}

fn invalid_state_response(desired_state: &String) -> Response<Body> {
    let json = format!("{{\"error\": \"Invalid state, {}\"}}", desired_state).to_string();
    let mut response = easy_json_response(json);
    *response.status_mut() = StatusCode::BAD_REQUEST;
    return response;
}

fn set_desired_state(req: &Request<Body>) -> Option<Response<Body>> {
    let params = get_params(req);
    let desired_state_param = params.get("state").unwrap();
    let desired_state = gate::State::from_str(desired_state_param);

    if desired_state.is_ok() {
        let state_changed: bool;
        unsafe {
            state_changed = GATE.as_mut().unwrap().change_state(desired_state.unwrap());
        }
        if state_changed == false {
            let json_response = "{\"error\": \"Could not move to desired_state. Most likely due to a lock.\"}".to_string();
            return Some(easy_json_response(json_response));
        }
        return None;
    } else {
        return Some(invalid_state_response(desired_state_param));
    }
}

fn lock_state(req: &Request<Body>) -> Option<Response<Body>> {
    let params = get_params(req);
    let lock_state_param = params.get("lock_state").unwrap();
    let desired_state = gate::State::from_str(lock_state_param);

    if desired_state.is_ok() {
        // TODO: Safety, also prob default TTL in config?
        let ttl_param = params.get("lock_state_ttl_seconds").unwrap();
        let ttl = std::time::Duration::from_secs(ttl_param.parse().unwrap());
        let lock_added: bool;
        unsafe {
            if ttl > MAX_STATE_LOCK_TTL.unwrap() {
                let json_response = format!("{{\"error\": \"Requested TTL is greater than {:?}, the server limit.\"}}", MAX_STATE_LOCK_TTL.unwrap()).to_string();
                return Some(easy_json_response(json_response));
            }
            lock_added = GATE.as_mut().unwrap().hold_state(desired_state.unwrap(), ttl);
        }
        if lock_added == false {
            let json_response = "{\"error\": \"Could not move to desired_state. Most likely due to a lock to a different state.\"}".to_string();
            return Some(easy_json_response(json_response));
        }
        return None;
    } else {
        return Some(invalid_state_response(lock_state_param));
    }            
}

// POST /gate
fn post_gate(req: Request<Body>) -> Response<Body> {
    let mut operation_taken = false;
    let params = get_params(&req);

    if params.contains_key("state") {
        operation_taken = true;
        let result = set_desired_state(&req);
        if result.is_some() {
            return result.unwrap();
        }
    }

    if params.contains_key("lock_state") {
        operation_taken = true;
        let result = lock_state(&req);
        if result.is_some() {
            return result.unwrap();
        }
    }

    if operation_taken == false {
        let body = Body::from("{\"error\": \"no operation requested\"}");
        let mut response = Response::new(body);
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return response;
    } else {
        return get_gate();
    }
}

// 404 handler
pub fn page_not_found() -> Response<Body> {
    let mut response = Response::default();
    *response.status_mut() = StatusCode::NOT_FOUND;
    return response;
}

pub async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            return Ok(get_root());
        }
        (&Method::GET, "/gate") => {
           return Ok(get_gate());
        },
        (&Method::POST, "/gate") => {
            return Ok(post_gate(req));
        }
        _ => {
            return Ok(page_not_found());
        }
    }
}
