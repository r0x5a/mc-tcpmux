use anyhow::anyhow;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub host: String,
	pub port: u16,

	pub servers: Option<Vec<Server>>,
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
		let config: Config = toml::from_str(&config)
			.map_err(|e| anyhow!("Failed to parse config file {}: {}", config_path, e))?;

		Ok(config)
	}
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
	pub src: String,
	pub target: Target,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
	pub host: Option<String>,
	pub port: Option<u16>,
}
