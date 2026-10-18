//! Minimal loopback TCP exchange using DINAH's blocking byte-stream API.

use dinah::network::{Connection, ConnectionState, Endpoint};
use std::error::Error;
use std::net::{Shutdown, SocketAddr};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let endpoint = Endpoint::bind(SocketAddr::from(([127, 0, 0, 1], 0)))?;
    let address = endpoint.local_addr()?;

    let server = thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
        let (mut peer, peer_address) = endpoint.accept()?;
        let mut request = [0_u8; 4];
        read_exact_from_connection(&mut peer, &mut request)?;
        println!(
            "server_peer={peer_address} request={}",
            String::from_utf8_lossy(&request)
        );
        peer.write_all(b"pong")?;
        peer.shutdown(Shutdown::Write)?;
        Ok(())
    });

    let mut client = Connection::connect_timeout(address, Duration::from_secs(2))?;
    client.write_all(b"ping")?;
    let mut response = [0_u8; 4];
    read_exact_from_connection(&mut client, &mut response)?;
    println!("client_response={}", String::from_utf8_lossy(&response));
    let mut eof_probe = [0_u8; 1];
    let count = client.read(&mut eof_probe)?;
    println!("peer_eof={} state={:?}", count == 0, client.state());
    assert_eq!(count, 0);
    assert_eq!(client.state(), ConnectionState::ReadClosed);
    server.join().map_err(|_| "server thread panicked")??;
    Ok(())
}

fn read_exact_from_connection(
    connection: &mut Connection,
    buffer: &mut [u8],
) -> Result<(), Box<dyn Error>> {
    let mut offset = 0;
    while offset < buffer.len() {
        let count = connection.read(&mut buffer[offset..])?;
        if count == 0 {
            return Err("connection closed before the complete message arrived".into());
        }
        offset += count;
    }
    Ok(())
}
