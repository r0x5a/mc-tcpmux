use tokio::net::TcpListener;
use tracing::{Level, event, span};

use crate::{
	config::Config,
	handler::{HandleResult, handle_packet},
};

mod config;
mod handler;
mod parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();

	let config = Config::load()?;
	let listener = TcpListener::bind((config.host.as_str(), config.port)).await?;

	loop {
		let (mut socket, addr) = listener.accept().await?;
		let config = config.clone();

		tokio::spawn(async move {
			let span = span!(Level::INFO, "connection", addr = %addr);
			let _enter = span.enter();

			let _dest = loop {
				match handle_packet(&mut socket, &config).await {
					Ok(HandleResult::Continue) => {}
					Ok(HandleResult::Close) => {
						event!(Level::INFO, "Connection closed");
						return;
					}
					Ok(HandleResult::Forward) => {
						event!(Level::INFO, "Forwarding packet (not implemented)");
						continue;
					}
					Err(e) => {
						event!(Level::ERROR, "Error handling packet: {e}");
						return;
					}
				}
			};
		});
	}
}
