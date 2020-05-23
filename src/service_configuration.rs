use crate::gate::GateConfiguration;

use serde::Deserialize;
use std::io::BufReader;
use std::fs::File;

#[derive(Deserialize)]
pub struct ServiceConfiguration {
    pub server_port: u16,
    pub gate_configuration: GateConfiguration
}

pub fn load() -> ServiceConfiguration {
    let config_file = File::open("service_configuration.json").expect("Could not find service_configuration.json");
    let config_reader = BufReader::new(config_file);
    return serde_json::from_reader(config_reader).expect("Error reading configuration.");
}