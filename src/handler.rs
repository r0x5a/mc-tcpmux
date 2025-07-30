use std::io::Cursor;

use tokio::{
	io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
};
use tracing::info;

use crate::{
	config::{Config, ErrorConfig, Motd},
	io::{calc_varint_size, read_packet, read_string, read_varint, write_varint},
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
	if socket.peek(&mut [0; 1]).await? == 0 {
		info!("Socket closed by client");
		return Ok(HandleResult::Close);
	}
	let buf = read_packet(socket).await?;
	let mut rdr = Cursor::new(&buf);

	let id = read_varint(&mut rdr).await?;
	match id {
		0x00 if buf.len() > 1 => {
			let packet = read_handshake(&mut rdr).await?;
			info!(
				"Received handshake packet. Version: {}, Host: {}, Port: {}, Intent: {}",
				packet.version, packet.host, packet.port, packet.intent
			);

			let server = config.find_server(&packet.host, packet.port);
			if let Some(server) = server {
				info!("Found server. Destination: {}", server.dst);
				return Ok(HandleResult::Forward((server.dst.clone(), buf)));
			}

			info!("No matching server found for {}:{}", packet.host, packet.port);
			info!("Handling error using \"{}\"", config.error);
			match config.error {
				ErrorConfig::Close => return Ok(HandleResult::Close),
				ErrorConfig::Motd { .. } => return Ok(HandleResult::Continue),
			}
		}
		0x00 => {
			info!("Received status request packet.");

			if let ErrorConfig::Motd(Motd { json, .. }) = &config.error {
				info!("Sending MOTD response");

				let json_len = json.len() as i32;
				let size = calc_varint_size(0x00) + calc_varint_size(json_len) + json.len();

				write_varint(socket, size as i32).await?;
				write_varint(socket, 0x00).await?;
				write_varint(socket, json_len).await?;
				socket.write_all(json.as_bytes()).await?;
				socket.flush().await?;
			}
		}
		0x01 => {
			let ping_id = read_ping(&mut rdr).await?;
			info!("Received ping packet. ID: {ping_id}");

			if let ErrorConfig::Motd(Motd { ping, .. }) = &config.error {
				if *ping {
					write_varint(socket, buf.len() as i32).await?;
					socket.write_all(&buf).await?; // echo the packet
					socket.flush().await?;
				} else {
					info!("Ping handling is disabled in the configuration");
				}
			}
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

async fn read_ping<R: AsyncRead + Unpin>(socket: &mut R) -> anyhow::Result<i64> {
	Ok(socket.read_i64().await?)
}
