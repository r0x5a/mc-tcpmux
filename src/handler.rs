use std::io::Cursor;

use tokio::{
	io::{AsyncRead, AsyncReadExt},
	net::TcpStream,
};
use tracing::info;

use crate::{
	config::Config,
	io::{read_packet, read_string, read_varint},
};

pub enum HandleResult {
	Continue,
	Close,
	Forward((String, Vec<u8>)),
}

#[tracing::instrument(skip(socket, config))]
pub async fn handle_packet(
	socket: &mut TcpStream,
	config: &Config,
) -> anyhow::Result<HandleResult> {
	let buf = read_packet(socket).await?;
	let mut rdr = Cursor::new(&buf);

	let id = read_varint(&mut rdr).await?;
	match id {
		0x00 if buf.len() > 1 => {
			let packet = read_handshake(&mut rdr).await?;
			info!("Received handshake packet. Version: {}, Host: {}, Port: {}, Intent: {}", packet.version, packet.host, packet.port, packet.intent);

			let server = config.find_server(&packet.host, packet.port);
			if let Some(server) = server {
				info!("Found server. Destination: {}", server.dst);
				return Ok(HandleResult::Forward((server.dst.clone(), buf)));
			} else {
				info!("No matching server found for {}:{}", packet.host, packet.port);
				// TODO: Return a proper error response (either an error MOTD or just disconnect)
			}
		}
		0x00 => {
			info!("Received empty handshake packet");
		}
		_ => {
			info!("Unhandled packet ID: {id}");
			return Ok(HandleResult::Close);
		}
	}

	Ok(HandleResult::Continue)
}

#[derive(Debug)]
struct Handshake {
	version: i32,
	host: String,
	port: u16,
	intent: i32,
}

async fn read_handshake<R: AsyncRead + Unpin>(socket: &mut R) -> anyhow::Result<Handshake> {
	let version = read_varint(socket).await?;
	let host = read_string(socket).await?;
	let port = socket.read_u16().await?;
	let intent = read_varint(socket).await?;

	Ok(Handshake { version, host, port, intent })
}
