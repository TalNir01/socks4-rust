use std::io::{self};
use std::net::Ipv4Addr;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug)]
pub struct Socks4Request {
    pub version: u8,      // Always 0x04
    pub command: u8,      // 0x01 for CONNECT
    pub dst_port: u16,    // Destination port (big-endian)
    pub dst_ip: Ipv4Addr, // Destination IPv4 address
    pub user_id: String,  // Null-terminated USERID
}

#[derive(Debug)]
pub struct Socks4Response {
    // pub version: u8,      // Always 0x00 in response
    pub status: u8,       // Status (0x5A = success)
    pub dst_port: u16,    // Destination port (big-endian on the wire)
    pub dst_ip: Ipv4Addr, // Destination IP address
}

impl Socks4Response {
    /// Serialize the SOCKS4 response to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        // 8 bytes total:
        let mut buf = Vec::new(); // Vector to hold bytes
        buf.push(0x00); // Version (0x00 for response) - Constant
        buf.push(self.status);
        buf.extend(&self.dst_port.to_be_bytes()); // Big-endian port
        buf.extend(&self.dst_ip.octets()); // IPv4 address as bytes (Big-endian)
        buf // Return the serialized bytes
    }
}

impl Socks4Request {
    pub async fn from_stream<R: AsyncRead + Unpin>(stream: &mut R) -> io::Result<Socks4Request> {
        // Buffer for fixed header fields (version, command, port, ip)
        let mut header = [0u8; 8];
        stream.read_exact(&mut header).await?;

        let version = header[0]; // Should be 0x04 for SOCKS4
        if version != 0x04 {
            // Currently only SOCKS4 is supported
            // If the version is not 0x04, return an error
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid SOCKS4 version",
            ));
        }
        let command = header[1]; // Should be 0x01 for CONNECT
        if command != 0x01 {
            // If the command is not 0x01, return an error
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported SOCKS4 command",
            ));
        }

        let dst_port = u16::from_be_bytes([header[2], header[3]]); // Big-endian port
        let dst_ip = [header[4], header[5], header[6], header[7]]; // IPv4 address
        let dest_ip = Ipv4Addr::new(dst_ip[0], dst_ip[1], dst_ip[2], dst_ip[3]);

        // Read user_id until null byte
        let mut user_id_bytes = Vec::new();
        loop {
            // Read until null byte
            if user_id_bytes.len() >= 255 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "USERID too long",
                ));
            }
            let mut byte = [0u8; 1];
            stream.read_exact(&mut byte).await?; // Read one byte at a time

            if byte[0] == 0 {
                break;
            }
            user_id_bytes.push(byte[0]); // Append byte to user_id_bytes, if not null
        }

        let user_id = String::from_utf8(user_id_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Socks4Request {
            version: version,
            command: command,
            dst_port: dst_port,
            dst_ip: dest_ip,
            user_id: user_id,
        })
    }
}
