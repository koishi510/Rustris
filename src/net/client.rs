use std::io;
use std::net::TcpStream;

use super::transport::Connection;

pub fn connect(addr: &str) -> io::Result<Connection> {
    let stream = TcpStream::connect(addr)?;
    Connection::new(stream)
}
