use std::path::Path;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub host: String,
	pub port: u16,
	pub motd: Option<Motd>,

	pub servers: Option<Vec<Server>>,
}

impl Config {
	pub fn load(config_path: &Path) -> anyhow::Result<Self> {
		let config = std::fs::read_to_string(config_path)
			.map_err(|e| anyhow!("Failed to read config file {}: {}", config_path.display(), e))?;
		let config: Config = toml::from_str(&config)
			.map_err(|e| anyhow!("Failed to parse config file {}: {}", config_path.display(), e))?;

		Ok(config)
	}

	pub fn find_server(&self, host: &str, port: u16) -> Option<&Server> {
		self.servers.as_ref().and_then(|l| l.iter().find(|s| s.src.matches(host, port)))
	}
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
	pub src: Target,
	pub dst: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
	pub host: Option<String>,
	pub port: Option<u16>,
}

impl Target {
	pub fn matches(&self, host: &str, port: u16) -> bool {
		self.host.as_ref().is_none_or(|p| p == host) && self.port.is_none_or(|p| p == port)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Motd {
	pub version: MotdVersion,
	pub description: Option<serde_json::Value>,
	pub favicon: Option<String>,
	pub players: Option<MotdPlayers>,

	#[serde(default = "default_ping")]
	pub ping: bool,
}

fn default_ping() -> bool {
	true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MotdVersion {
	pub name: Option<String>,
	pub protocol: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotdPlayers {
	pub max: Option<i32>,
	pub online: Option<i32>,
	pub sample: Option<Vec<MotdPlayer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotdPlayer {
	pub name: String,
	pub id: String,
}
