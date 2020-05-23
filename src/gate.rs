use std::{thread, time};
use std::str::FromStr;
use std::future::Future;
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
    // TODO: will need a better way to know if the gate closes,
    // eg if something is in its way.
    // would be awesome to have a switch that is activated on open and one on closed.
    // this would require some physical changes.
    pub time_held_open: time::Duration
}

#[derive(Serialize)]
pub struct Gate {
    #[serde(skip_serializing)]
    pub configuration: GateConfiguration,
    // public until I can get default working
    // TODO: use default so this becomes a private variable
    pub current_state: State
}

// TODO: a hold_state function. Will likely use something like
// 'reference' counting so if multiple services demand the gate
// stays held open, we wait until all of them release their lock.
// We will probably also need to introduce an API to release all
// the locks or provide a reasonable TTL for the locks/hold_state.
impl Gate {
    // Note: this is a long running function and should be ran in a thread.
    pub fn change_state(&mut self, desired_state: State) -> () {
        if self.current_state == desired_state {
            return;
        } else {
            // TODO: Just use a write lock instead.
            while self.current_state == State::MOVING {
                thread::sleep(time::Duration::from_millis(500));
            }

            // Potentially moved
            if self.current_state == desired_state {
                return;
            }

            self.move_to(desired_state);
            if self.current_state == State::OPEN {
                thread::sleep(self.configuration.time_held_open);
                // TODO: take lock - also duplication
                if self.current_state == State::OPEN {
                    println!("Gate auto closing");
                    self.move_to(State::CLOSED);
                }
            }
        }
    }

    fn move_to(&mut self, desired_state: State) -> () {
        println!("Gate is moving!");
        self.current_state = State::MOVING;
        thread::sleep(self.configuration.time_to_move);
        self.current_state = desired_state;
        println!("Gate is now {:?}", self.current_state);
    }
}