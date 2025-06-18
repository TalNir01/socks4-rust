//! A simple SOCKS4 server implementation in Rust
#![deny(warnings, unsafe_code)]
use std::env;
use std::net::Ipv4Addr;
use std::process;
use std::str::FromStr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};
use tracing_subscriber;

pub mod socks_constants;
pub mod socks_specification;
use crate::socks_constants::socks4_status;
use crate::socks_specification::{Socks4Request, Socks4Response};

/// A simple SOCKS4 server that listens for incoming connections and forwards them to a specified destination.
/// The server accepts connections, reads SOCKS4 requests, connects to the specified destination,
/// and forwards data between the client and the destination server.
/// The destination IP and port are provided as command line arguments.
/// Usage: cargo run -- <listening_ip> <destination_port>
/// Example: cargo run -- 0.0.0.0 1080
/// This server is a basic implementation and does not handle all edge cases or security concerns.
#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let cmd_args: Vec<String> = env::args().collect();
    if cmd_args.len() != 3 {
        error!("Usage: {} <listening_ip> <destination_port>", cmd_args[0]);
        process::exit(1); // Exit with an error code
    }

    let dst_ip = Ipv4Addr::from_str(&cmd_args[1]).unwrap_or_else(|_| {
        error!("Invalid destination IP address: {}", cmd_args[1]);
        process::exit(1);
    });
    let dst_port: u16 = cmd_args[2].parse().unwrap_or_else(|_| {
        error!("Invalid destination port: {}", cmd_args[2]);
        process::exit(1);
    });
    // TODO: Replace this with command line argument parsing
    let bind_address = format!("{}:{}", dst_ip, dst_port);
    let listener = TcpListener::bind(bind_address.clone()).await?;
    info!("Socks4 Server Initialized On {bind_address}");

    loop {
        let (socket, addr) = listener.accept().await?;
        debug!("Accepted connection from {}", addr);
        // Spawn a new task to handle the connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                error!("Error handling {}: {:?}", addr, e);
            }
        });
    }
}

/// Initiate TcpStream to the destination server.
/// # Arguments
/// * `dst_ip` - The destination IP address to connect to.
/// * `dst_port` - The destination port to connect to.  
/// # Returns
/// A `TcpStream` connected to the destination server, or an error if the connection fails. (async value)
async fn connect_to_destination(dst_ip: Ipv4Addr, dst_port: u16) -> tokio::io::Result<TcpStream> {
    let addr = format!("{}:{}", dst_ip, dst_port); // Format the destination address
    TcpStream::connect(addr).await
}

/// Handle an incoming connection from a client.
/// This function reads the SOCKS4 request from the client, connects to the specified destination,
/// and forwards data between the client and the destination server.    
/// # Arguments
/// * `client` - The `TcpStream` representing the client connection.    
/// # Returns
/// A `Result` indicating success or failure. If successful, the connection is handled and data is forwarded.
async fn handle_connection(mut client: TcpStream) -> tokio::io::Result<()> {
    // Connect to the destination server (hardcoded for now)
    let client_request: Socks4Request =
        Socks4Request::from_stream(&mut client).await.map_err(|e| {
            error!("Failed to read SOCKS4 request: {}", e);
            e
        })?;
    debug!("Received client socks4 request: {:?}", client_request);

    match connect_to_destination(client_request.dst_ip, client_request.dst_port).await {
        Ok(mut target_connection) => {
            info!(
                "Successfully connected to destination: {}:{}",
                client_request.dst_ip, client_request.dst_port
            );
            let success_response = Socks4Response {
                status: socks4_status::REQUEST_GRANTED,
                dst_port: client_request.dst_port,
                dst_ip: client_request.dst_ip,
            };
            // Send success response to the client
            client.write(success_response.to_bytes().as_slice()).await?;
            debug!("Returned success response to client: {success_response:?}");
            tokio::io::copy_bidirectional(&mut client, &mut target_connection).await?; // Propagate errors
            Ok(())
        }
        Err(e) => {
            error!("Failed to connect to destination: {}", e);
            let failure_response = Socks4Response {
                status: socks4_status::REQUEST_REJECTED,
                dst_port: client_request.dst_port,
                dst_ip: client_request.dst_ip,
            };
            client.write(failure_response.to_bytes().as_slice()).await?;
            warn!("Returned failure response to client: {failure_response:?}");
            return Err(e);
        }
    }
}
