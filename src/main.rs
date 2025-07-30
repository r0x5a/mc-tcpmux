use tokio::{
	io::{copy_bidirectional, AsyncWriteExt},
	net::{TcpListener, TcpStream},
};
use tracing::{Instrument, error, info, info_span};

use crate::{
	config::Config,
	handler::{handle_packet, HandleResult}, io::write_varint,
};

mod config;
mod handler;
mod io;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();

	let config = Config::load()?;
	let listener = TcpListener::bind((config.host.as_str(), config.port)).await?;

	loop {
		let (mut socket, addr) = listener.accept().await?;
		let config = config.clone();

		tokio::spawn(
			async move {
				info!("Accepted connection.");

				loop {
					match handle_packet(&mut socket, &config).await {
						Ok(HandleResult::Continue) => {}
						Ok(HandleResult::Close) => break info!("Connection closed"),
						Ok(HandleResult::Forward((target, buf))) => {
							if let Err(e) = proxy(&mut socket, target, &buf).await {
								error!("Failed to proxy connection: {e}");
							}
							break;
						}
						Err(e) => break error!("Error handling packet: {e}"),
					}
				}

				if let Err(e) = socket.shutdown().await {
					error!("Failed to shutdown socket: {e}");
				} else {
					info!("Socket shutdown successfully");
				}
			}
			.instrument(info_span!("handle_conn", addr = %addr)),
		);
	}
}

async fn proxy(socket: &mut TcpStream, target: String, buf: &[u8]) -> anyhow::Result<()> {
	info!("Proxying connection to {}", target);

	let mut dst = TcpStream::connect(target).await?;

	// write handshake packet
	write_varint(&mut dst, buf.len() as i32).await?;
	dst.write_all(buf).await?;

	copy_bidirectional(socket, &mut dst).await?;
	Ok(())
}
