use std::{thread, time};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gpio_cdev::{Chip, LineRequestFlags};
use log::{trace, debug, info, warn};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Serialize)]
pub struct LockStateLock {
    #[serde(skip_serializing)]
    id: String,
    expires: time::Duration
}

#[derive(Deserialize)]
pub struct GateConfiguration {
    pub pull_to_open: bool,
    pub gpio_motor: u32,
    pub gpio_cycle_relay: u32,
    pub gpio_exit_relay: u32,
    pub gpio_master_orange: u32
}

pub struct Gate {
    gpio_motor: gpio_cdev::LineHandle,
    gpio_master_orange: gpio_cdev::LineHandle,
    gpio_cycle_relay: gpio_cdev::LineHandle,
    gpio_exit_relay: gpio_cdev::LineHandle,
    pull_to_open: bool,
    locked_state: State,
    state_locks: Vec<LockStateLock>
}

fn cycle_relay(pin: &gpio_cdev::LineHandle) {
    pin.set_value(1);
    thread::sleep(time::Duration::from_secs(1));
    pin.set_value(0);
}

impl Gate {
    pub fn new(config: GateConfiguration) -> Gate {
        let mut chip = Chip::new("/dev/gpiochip0").unwrap();

        let motor_handle = chip.get_line(config.gpio_motor).unwrap().request(LineRequestFlags::INPUT, 0, "mighty_mule_gate").unwrap();
        let master_orange_handle = chip.get_line(config.gpio_master_orange).unwrap().request(LineRequestFlags::INPUT, 0, "mighty_mule_gate").unwrap();
        let exit_relay_handle = chip.get_line(config.gpio_exit_relay).unwrap().request(LineRequestFlags::OUTPUT, 0, "mighty_mule_gate").unwrap();
        let cycle_relay_handle = chip.get_line(config.gpio_cycle_relay).unwrap().request(LineRequestFlags::OUTPUT, 0, "mighty_mule_gate").unwrap();

        return Gate {
            pull_to_open: config.pull_to_open,
            gpio_motor: motor_handle,
            gpio_exit_relay: exit_relay_handle,
            gpio_cycle_relay: cycle_relay_handle,
            gpio_master_orange: master_orange_handle,
            locked_state: State::CLOSED,
            state_locks: vec!()
        }
    }

    pub fn get_state_locks(&self) -> Vec<LockStateLock> {
        return self.state_locks.clone();
    }

    pub fn get_state(&self) -> State {
        if self.gpio_motor.get_value().unwrap() == 1 {
            trace!("GPIO motor pin was high");
            return State::MOVING;
        } else if self.gpio_master_orange.get_value().unwrap() == 1 {
            trace!("GPIO master orange was high");
            if self.pull_to_open {
                return State::OPEN;
            } else {
                return State::CLOSED;
            }
        } else {
            trace!("Neither GPIO motor pin or master orange, was high.");
            if self.pull_to_open {
                return State::CLOSED;
            } else {
                return State::OPEN;
            }
        }
    }

    fn move_state(&mut self, desired_state: &State) -> bool {
        let state = self.get_state();
        if state == State::MOVING || *desired_state == State::MOVING {
            return false;
        }

        if state != *desired_state {
            info!("Gate moving to {:?} from {:?}.", desired_state, state);
            if *desired_state == State::OPEN {
                cycle_relay(&self.gpio_exit_relay);
            } else { // State::CLOSED
                cycle_relay(&self.gpio_cycle_relay);
            }
        }

        return true;
    }

    pub fn change_state(&mut self, desired_state: State) -> bool {
        self.sync();

        if self.state_locks.len() > 0 {
            debug!("Tried to move the gate while the state was locked");
            return false;
        }

        return self.move_state(&desired_state);
    }

    pub fn sync(&mut self) -> () {
        self.clear_expired_locks();
    }

    pub fn delete_lock(&mut self, id: &str) -> Result<(), ()> {
        if self.state_locks.len() == 0 {
            return Err(());
        }

        let mut index = 0;
        for lock in &self.state_locks {
            if lock.id == id {
                self.state_locks.remove(index);
                debug!("Removed lock {}", id);
                self.clear_lock_state_if_required();
                return Ok(());
            }
            index = index + 1;
        }

        info!("Did not find a lock with ID {}", id);
        return Err(());
    }

    fn clear_expired_locks(&mut self) -> () {
        if self.state_locks.len() == 0 {
            return;
        }

        let previous_lock_count = self.state_locks.len();
        self.state_locks.retain(|lock| {
            lock.expires > time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap()
        });
        let new_lock_count = self.state_locks.len();
        if previous_lock_count > new_lock_count {
            info!("Locks went from {} -> {}", previous_lock_count, new_lock_count);
        }

        self.clear_lock_state_if_required();
    }

    fn clear_lock_state_if_required(&mut self) -> () {
        if self.state_locks.len() == 0 {
            info!("All locks removed from gate. Was held in {:?}.", self.locked_state);
        }

        if self.locked_state == State::OPEN && self.state_locks.len() == 0 {
            debug!("Set GPIO exit relay to low");
            self.gpio_exit_relay.set_value(0);
        }
    }

    pub fn get_locked_state(&mut self) -> Option<State> {
        if self.state_locks.len() == 0 {
            return None;
        } else {
            return Some(self.locked_state);
        }
    }

    pub fn hold_state(&mut self, desired_state: State, ttl: time::Duration) -> Result<String, String> {
        self.sync();
        if self.state_locks.len() > 0 && desired_state != self.locked_state {
            warn!("Can not change locked state to {:?}. Already locked in {:?}.", desired_state, self.locked_state);
            return Err(format!("Being held in {:?} state. Can not change to holding {:?}.", self.locked_state, desired_state));
        }

        if desired_state == State::MOVING {
            return Err("Gate can not be locked to the moving state.".to_string());
        }

        let id = Uuid::new_v4().to_hyphenated();
        debug!("Issued a lock with ID {} for {:?}.", id.to_string(), &desired_state);
        let lock = LockStateLock {
            id: id.to_string(),
            expires: ttl + time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap()
        };
        self.state_locks.push(lock);

        if desired_state == State::OPEN {
            debug!("Set GPIO exit relay to high");
            self.gpio_exit_relay.set_value(1);
        } else if self.get_state() != desired_state { // State::CLOSED
            self.move_state(&desired_state);
        }

        self.locked_state = desired_state;

        return Ok(id.to_string());
    }
}
