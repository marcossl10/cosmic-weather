// SPDX-License-Identifier: MIT

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 2]
pub struct Config {
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub city: Option<String>,
    pub units: String, // 'metric', 'imperial', 'kelvin'
    pub auto_update: bool,
    pub update_interval: u64, // in minutes
}

impl Config {
    pub fn new(lat: Option<String>, lon: Option<String>, city: Option<String>) -> Self {
        Self {
            latitude: lat,
            longitude: lon,
            city,
            units: "metric".to_string(),
            auto_update: true,
            update_interval: 15, // 15 minutes by default
        }
    }
}
