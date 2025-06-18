use std::net::Ipv4Addr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

pub mod socks_constants;
pub mod socks_specification;
use crate::socks_constants::socks4_status;
use crate::socks_specification::{Socks4Request, Socks4Response};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // TODO: Replace this with command line argument parsing
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Echo server listening on 0.0.0.0:8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling {}: {:?}", addr, e);
            }
        });
    }
}

async fn connect_to_destination(dst_ip: Ipv4Addr, dst_port: u16) -> tokio::io::Result<TcpStream> {
    let addr = format!("{}:{}", dst_ip, dst_port); // Format the destination address
    TcpStream::connect(addr).await
}

async fn handle_connection(mut client: TcpStream) -> tokio::io::Result<()> {
    // Connect to the destination server (hardcoded for now)
    let client_request: Socks4Request =
        Socks4Request::from_stream(&mut client).await.map_err(|e| {
            eprintln!("Failed to read SOCKS4 request: {}", e);
            e
        })?;
    println!("Received SOCKS4 request: {:?}", client_request);
    println!("{} To {}", client_request.dst_ip, client_request.dst_port);

    match connect_to_destination(client_request.dst_ip, client_request.dst_port).await {
        Ok(mut target_connection) => {
            println!(
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
            println!("Sent success response to client");
            tokio::io::copy_bidirectional(&mut client, &mut target_connection).await?; // Propagate errors
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to connect to destination: {}", e);
            let failure_response = Socks4Response {
                status: socks4_status::REQUEST_REJECTED,
                dst_port: client_request.dst_port,
                dst_ip: client_request.dst_ip,
            };
            client.write(failure_response.to_bytes().as_slice()).await?;
            println!("Sent failure response to client: {failure_response:?}");
            return Err(e);
        }
    }

    // let mut server = TcpStream::connect("127.0.0.1:8088").await?;
    // println!("Connected to destination server at 127.0.0.1:8080");

    // // Forward: client <=> server (two-way)
    // tokio::io::copy_bidirectional(&mut client, &mut server).await?; // Propagate errors

    // Ok(())
}
