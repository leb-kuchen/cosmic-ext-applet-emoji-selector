use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

use serde::{Deserialize, Serialize};
pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    show_tooltip: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_tooltip: false,
        }
    }
}
