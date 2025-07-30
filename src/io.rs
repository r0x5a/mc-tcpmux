use anyhow::{anyhow, ensure};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn read_varint<R: AsyncRead + Unpin>(reader: &mut R) -> anyhow::Result<i32> {
	let mut value = 0;
	let mut pos = 0;
	loop {
		let cur = reader.read_u8().await?;
		value |= ((cur & 0x7F) as i32) << pos;
		if (cur & 0x80) == 0 {
			return Ok(value);
		}
		pos += 7;
		ensure!(pos < 32, "VarInt is too big");
	}
}

pub async fn write_varint<W: AsyncWrite + Unpin>(writer: &mut W, value: i32) -> anyhow::Result<()> {
	let mut value = value as u32;
	while value >= 0x80 {
		writer.write_u8((value & 0x7F) as u8 | 0x80).await?;
		value >>= 7;
	}
	writer.write_u8(value as u8).await?;
	Ok(())
}

pub async fn read_string<R: AsyncRead + Unpin>(reader: &mut R) -> anyhow::Result<String> {
	let len = read_varint(reader).await?;
	let mut buf = vec![0; len as usize];
	reader.read_exact(&mut buf).await?;
	String::from_utf8(buf).map_err(|e| anyhow!("Invalid UTF-8 string: {e}"))
}

pub async fn read_packet<R: AsyncRead + Unpin>(socket: &mut R) -> anyhow::Result<Vec<u8>> {
	let len = read_varint(socket).await? as usize;
	let mut data = vec![0; len];
	socket.read_exact(&mut data).await?;
	Ok(data)
}
