use std::io::Cursor;

use tokio::{
	io::{AsyncRead, AsyncReadExt},
	net::TcpStream,
};
use tracing::{event, Level};

use crate::{config::Config, parser::{read_packet, read_string, read_varint}};

pub enum HandleResult {
	Continue,
	Close,
	Forward,
}

pub async fn handle_packet(socket: &mut TcpStream, config: &Config) -> anyhow::Result<HandleResult> {
	let buf = read_packet(socket).await?;
	let mut rdr = Cursor::new(&buf);

	let id = read_varint(&mut rdr).await?;
	match id {
		0x00 if buf.len() > 1 => {
			let packet = read_handshake(&mut rdr).await?;
			event!(Level::INFO, "Received handshake packet: {packet:?}");
		}
		0x00 => {
			event!(Level::INFO, "Received empty handshake packet");
		}
		_ => {
			event!(Level::INFO, "Unhandled packet ID: {id}");
		}
	}

	Ok(HandleResult::Continue)
}

#[derive(Debug)]
struct Handshake {
	protocol_version: i32,
	server_address: String,
	server_port: u16,
	intent: i32,
}

async fn read_handshake<R: AsyncRead + Unpin>(socket: &mut R) -> anyhow::Result<Handshake> {
	Ok(Handshake {
		protocol_version: read_varint(socket).await?,
		server_address: read_string(socket).await?,
		server_port: socket.read_u16().await?,
		intent: read_varint(socket).await?,
	})
}
