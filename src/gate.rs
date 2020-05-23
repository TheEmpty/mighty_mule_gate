use std::{thread, time};
use std::str::FromStr;
use std::sync::{RwLock, RwLockWriteGuard};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum State {
    OPEN,
    MOVING,
    CLOSED
}

impl Default for State {
    fn default() -> Self { State::CLOSED }
}

impl FromStr for State {
    type Err = ();

    fn from_str(s: &str) -> Result<State, ()> {
        let mut copy = String::from(s);
        copy.make_ascii_uppercase();
        match copy.as_ref() {
            "OPEN" => Ok(State::OPEN),
            "MOVING" => Ok(State::MOVING),
            "CLOSED" => Ok(State::CLOSED),
            _ => Err(())
        }
    }
}

#[derive(Serialize, Deserialize)]
struct LockStateLock {
    expires: time::Duration
}

#[derive(Deserialize)]
pub struct GateConfiguration {
    pub time_to_move: time::Duration,
    pub time_held_open: time::Duration
}

#[derive(Serialize)]
pub struct Gate {
    #[serde(skip_serializing)]
    configuration: GateConfiguration,
    current_state: RwLock<State>,
    state_locks: Vec<LockStateLock>
}

fn move_state(mut lock: RwLockWriteGuard<State>, desired_state: State, time_to_move: &time::Duration) -> () {
    println!("Gate is moving!");
    // If desired_state = open, connect COM<->EXIT
    // If desired_state = closed, connect COM<->CYCLE
    *lock = State::MOVING;
    thread::sleep(*time_to_move);
    *lock = desired_state;
    println!("Gate is now {:?}", *lock);
}

impl Gate {
    pub fn new(config: GateConfiguration) -> Gate {
        return Gate {
            configuration: config,
            current_state: RwLock::new(State::CLOSED),
            state_locks: vec!()
        }
    }

    // Note: this is a long running function and should be ran in a thread.
    // Moving to using switches on the gate will fix this as we don't have
    // to use thread sleep to try and keep track of the gate's state.
    pub fn change_state(&mut self, desired_state: State) -> bool {
        self.clear_expired_locks();

        if self.state_locks.len() > 0 {
            // doesn't really matter if the state is desired or not.
            return false;
        }

        let state = self.current_state.write().unwrap();
        if *state == desired_state {
            // TODO: trigger exit if state is OPEN so it resets timer
            // ^ would also then need to expire the thread thing below if
            // that sticks around for awhile. But should just get some switches
            println!("Gate is already in desired state.");
            return true;
        } else {
            // giving up the lock here by passing ownership.
            move_state(state, desired_state, &self.configuration.time_to_move);
        }

        if *self.current_state.read().unwrap() == State::OPEN {
            thread::sleep(self.configuration.time_held_open);
            let state = self.current_state.write().unwrap();
            if *state == State::OPEN {
                println!("Gate auto closing");
                move_state(state, State::CLOSED, &self.configuration.time_to_move);
            }
        }

        return true;
    }

    pub fn clear_expired_locks(&mut self) -> () {
        self.state_locks.retain(|lock| {
            lock.expires > time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap()
        });
    }

    // return false if currently held in a different state.
    pub fn hold_state(&mut self, desired_state: State, ttl: time::Duration) -> bool {
        self.clear_expired_locks();
        if self.state_locks.len() > 0 && desired_state != *self.current_state.read().unwrap() {
            // being held in a different state
            return false;
        }

        let lock = LockStateLock {
            expires: ttl + time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap()
        };
        self.state_locks.push(lock);

        let state = self.current_state.write().unwrap();
        if *state != desired_state {
            move_state(state, desired_state, &self.configuration.time_to_move);
        }
        // then for OPEN, hold SAFETY<->COM
        // For holding the gate closed, hold OPEN EDGE<->COM

        return true;
    }
}
