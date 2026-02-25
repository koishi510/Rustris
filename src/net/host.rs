use std::io;
use std::net::TcpListener;

use super::transport::Connection;

pub fn listen_nonblocking(port: u16) -> io::Result<TcpListener> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

pub fn try_accept(listener: &TcpListener) -> io::Result<Option<Connection>> {
    match listener.accept() {
        Ok((stream, _addr)) => {
            let conn = Connection::new(stream)?;
            Ok(Some(conn))
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e),
    }
}
