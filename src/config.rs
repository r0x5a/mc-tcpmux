use anyhow::{anyhow, ensure};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub host: String,
	pub port: u16,

	pub servers: Option<Vec<Server>>,

	#[serde(skip)]
	pub default_server: Option<Server>,
}

impl Config {
	pub fn load() -> anyhow::Result<Self> {
		let args: Vec<_> = std::env::args().collect();
		if args.len() > 2 {
			eprintln!("Usage: {} [config file]", args[0]);
			std::process::exit(1);
		}
		let config_path = args.get(1).map_or("config.toml", |s| s.as_str());
		let config = std::fs::read_to_string(config_path)
			.map_err(|e| anyhow!("Failed to read config file {}: {}", config_path, e))?;
		let mut config: Config = toml::from_str(&config)
			.map_err(|e| anyhow!("Failed to parse config file {}: {}", config_path, e))?;

		if let Some(servers) = &config.servers {
			for server in servers {
				if let Some(true) = server.default {
					ensure!(config.default_server.is_none(), "Multiple default servers defined");
					config.default_server = Some(server.clone());
				}
			}
		}
		Ok(config)
	}

	pub fn find_server(&self, host: &str, port: u16) -> Option<&Server> {
		let server = self.servers.as_ref().and_then(|l| l.iter().find(|s| s.src.matches(host, port)));
		server.or(self.default_server.as_ref())
	}
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
	pub default: Option<bool>,
	pub src: Target,
	pub dst: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
	pub host: String,
	pub port: Option<u16>,
}

impl Target {
	pub fn matches(&self, host: &str, port: u16) -> bool {
		self.host == host && self.port.is_none_or(|p| p == port)
	}
}
