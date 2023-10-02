use figment::providers::{Format, Serialized, Toml};
use figment::Figment;

use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {}
