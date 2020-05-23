use std::{thread, time};
use std::str::FromStr;
use std::sync::{RwLock, RwLockWriteGuard};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
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

pub struct GateConfiguration {
    pub time_to_move: time::Duration,
    pub time_held_open: time::Duration
}

#[derive(Serialize)]
pub struct Gate {
    #[serde(skip_serializing)]
    configuration: GateConfiguration,
    current_state: RwLock<State>
}

fn move_state(mut lock: RwLockWriteGuard<State>, desired_state: State, time_to_move: &time::Duration) -> () {
    println!("Gate is moving!");
    *lock = State::MOVING;
    thread::sleep(*time_to_move);
    *lock = desired_state;
    println!("Gate is now {:?}", *lock);
}

impl Gate {
    pub fn new(config: GateConfiguration) -> Gate {
        return Gate {
            configuration: config,
            current_state: RwLock::new(State::CLOSED)
        }
    }

    // Note: this is a long running function and should be ran in a thread.
    pub fn change_state(&mut self, desired_state: State) -> () {
        let state = self.current_state.write().unwrap();
        if *state == desired_state {
            println!("Gate is already in desired state.");
            return;
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
    }
}
