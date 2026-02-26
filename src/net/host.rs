use std::io;
use std::net::{Ipv4Addr, TcpListener, UdpSocket};

use super::transport::Connection;

fn default_gateway() -> Option<Ipv4Addr> {
    let content = std::fs::read_to_string("/proc/net/route").ok()?;
    for line in content.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 3 && fields[1] == "00000000" {
            let gw = u32::from_str_radix(fields[2], 16).ok()?;
            if gw != 0 {
                let b = gw.to_ne_bytes();
                return Some(Ipv4Addr::new(b[0], b[1], b[2], b[3]));
            }
        }
    }
    None
}

pub fn local_ip() -> Option<std::net::IpAddr> {
    if let Some(gw) = default_gateway() {
        let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
        if socket.connect((gw, 80)).is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return Some(addr.ip());
            }
        }
    }
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip())
}

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
