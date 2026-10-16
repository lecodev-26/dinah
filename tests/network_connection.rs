//! Public contract tests for blocking TCP connections.
use dinah::network::{Connection, ConnectionError};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

#[test]
fn connection_exchanges_bytes_and_reports_eof() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
    let address = listener.local_addr().expect("listener address");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept client");
        let mut received = [0_u8; 4];
        stream.read_exact(&mut received).expect("read request");
        assert_eq!(&received, b"ping");
        stream.write_all(b"pong").expect("write response");
        stream
            .shutdown(Shutdown::Write)
            .expect("close server write side");
    });

    let mut connection = Connection::connect_timeout(address, Duration::from_secs(2))
        .expect("connect to local listener");
    assert_eq!(connection.peer_addr().expect("peer address"), address);
    connection.write_all(b"ping").expect("write request");
    let mut response = [0_u8; 4];
    let mut read = 0;
    while read < response.len() {
        let count = connection
            .read(&mut response[read..])
            .expect("read response");
        assert_ne!(count, 0, "peer closed before complete response");
        read += count;
    }
    assert_eq!(&response, b"pong");
    assert_eq!(connection.read(&mut [0_u8; 1]).expect("observe EOF"), 0);
    server.join().expect("server thread completes");
}

#[test]
fn zero_connect_timeout_is_rejected() {
    let address = "127.0.0.1:1".parse().expect("valid socket address");
    let result = Connection::connect_timeout(address, Duration::ZERO);
    assert!(
        matches!(result, Err(ConnectionError::Io(ref error)) if error.kind() == std::io::ErrorKind::InvalidInput)
    );
}

#[test]
fn existing_tcp_stream_can_be_wrapped_and_closed_explicitly() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
    let address = listener.local_addr().expect("listener address");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept client");
        let mut byte = [0_u8; 1];
        assert_eq!(stream.read(&mut byte).expect("read shutdown EOF"), 0);
    });
    let connection =
        Connection::from_tcp_stream(TcpStream::connect(address).expect("connect client"));
    assert_eq!(
        connection.local_addr().expect("local address").ip(),
        address.ip()
    );
    connection
        .shutdown(Shutdown::Write)
        .expect("explicit write shutdown");
    server.join().expect("server thread completes");
}
