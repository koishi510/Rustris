use std::io;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use super::transport::Connection;

pub fn connect(addr: &str) -> io::Result<Connection> {
    let sock_addr = addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "could not resolve address"))?;
    let stream = TcpStream::connect_timeout(&sock_addr, Duration::from_secs(5))?;
    Connection::new(stream)
}
