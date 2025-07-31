use std::path::PathBuf;

use clap::{arg, command};
use tokio::{net::TcpListener, sync::mpsc};
use tracing::{Instrument, error, info, info_span};

use crate::{config::Config, handler::handle_connection, watcher::watch_file};

mod config;
mod handler;
mod io;
mod watcher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();

	let matches = command!()
		.arg(arg!([config] "Path to the config file").default_value("config.toml"))
		.arg(arg!(-r --reload "Reload the config file when it changes. Note this only applies to new established connections."))
		.get_matches();

	let config_path = PathBuf::from(matches.get_one::<String>("config").unwrap());
	let reload = *matches.get_one::<bool>("reload").unwrap();

	let mut config = Config::load(&config_path)?;
	let listener = TcpListener::bind((config.host.as_str(), config.port)).await?;

	let (tx, mut rx) = mpsc::channel(16);
	if reload {
		info!("Watching for changes in {}", config_path.display());
		let config_path0 = config_path.clone();
		tokio::spawn(async move {
			watch_file(&config_path0, 500, tx).await.unwrap();
		});
	}

	loop {
		let recv_fut = rx.recv();
		tokio::pin!(recv_fut);

		tokio::select! {
			res = listener.accept() => {
				let (socket, addr) = res?;
				info!("Accepted connection from {}", addr);

				let config = config.clone();
				tokio::spawn(async move {
					handle_connection(socket, &config).await;
				}.instrument(info_span!("handle_conn", addr = %addr)));
			}
			_ = recv_fut => {
				match Config::load(&config_path) {
					Ok(new_config) => {
						config = new_config;
						info!("Config reloaded successfully");
					}
					Err(e) => {
						error!("Failed to reload config: {}", e);
					}
				}
			}
		}
	}
}
