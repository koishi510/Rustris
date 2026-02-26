use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::protocol::NetMessage;

const MAX_MESSAGE_LEN: usize = 64 * 1024;

pub struct Connection {
    stream: TcpStream,
    read_buf: Vec<u8>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            read_buf: Vec::new(),
        })
    }

    pub fn send(&mut self, msg: &NetMessage) -> io::Result<()> {
        let json = serde_json::to_string(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let payload = json.as_bytes();
        let len = payload.len() as u32;
        self.stream.write_all(&len.to_be_bytes())?;
        self.stream.write_all(payload)?;
        self.stream.flush()?;
        Ok(())
    }

    fn check_length(len: usize) -> io::Result<()> {
        if len > MAX_MESSAGE_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("message too large: {} bytes", len),
            ));
        }
        Ok(())
    }

    pub fn try_recv(&mut self) -> io::Result<Option<NetMessage>> {
        let mut tmp = [0u8; 4096];
        match self.stream.read(&mut tmp) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "peer disconnected",
                ));
            }
            Ok(n) => {
                self.read_buf.extend_from_slice(&tmp[..n]);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }

        if self.read_buf.len() < 4 {
            return Ok(None);
        }

        let len = u32::from_be_bytes([
            self.read_buf[0],
            self.read_buf[1],
            self.read_buf[2],
            self.read_buf[3],
        ]) as usize;

        Self::check_length(len)?;

        if self.read_buf.len() < 4 + len {
            return Ok(None);
        }

        let json_bytes: Vec<u8> = self.read_buf.drain(..4 + len).skip(4).collect();
        let msg: NetMessage = serde_json::from_slice(&json_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Some(msg))
    }

    pub fn recv_blocking(&mut self) -> io::Result<NetMessage> {
        self.stream.set_nonblocking(false)?;
        self.stream
            .set_read_timeout(Some(Duration::from_secs(10)))?;
        let result = loop {
            match self.try_recv_blocking_inner() {
                Ok(Some(msg)) => break Ok(msg),
                Ok(None) => continue,
                Err(e) => break Err(e),
            }
        };
        let _ = self.stream.set_read_timeout(None);
        let _ = self.stream.set_nonblocking(true);
        result
    }

    fn try_recv_blocking_inner(&mut self) -> io::Result<Option<NetMessage>> {
        let mut tmp = [0u8; 4096];
        match self.stream.read(&mut tmp) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "peer disconnected",
                ));
            }
            Ok(n) => {
                self.read_buf.extend_from_slice(&tmp[..n]);
            }
            Err(e) => return Err(e),
        }

        if self.read_buf.len() < 4 {
            return Ok(None);
        }

        let len = u32::from_be_bytes([
            self.read_buf[0],
            self.read_buf[1],
            self.read_buf[2],
            self.read_buf[3],
        ]) as usize;

        Self::check_length(len)?;

        if self.read_buf.len() < 4 + len {
            return Ok(None);
        }

        let json_bytes: Vec<u8> = self.read_buf.drain(..4 + len).skip(4).collect();
        let msg: NetMessage = serde_json::from_slice(&json_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Some(msg))
    }
}
