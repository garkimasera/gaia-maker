use std::io::{BufRead, Write};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    Start {},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    Start {},
}

pub fn run_client(port: u16) {
    let stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", port))
        .expect("cannot open stream with launcher");

    std::thread::spawn(move || {
        if let Err(e) = client_task(stream) {
            eprintln!("launcher connection error: {:?}", e);
        }
    });
}

fn client_task(stream: std::net::TcpStream) -> Result<()> {
    let mut client = Client::new(stream);

    client.send(Request::Start {})?;
    let resp = client.recv()?;
    if !matches!(resp, Response::Start {}) {
        bail!("invalid response");
    }
    eprintln!("connected to game launcher");

    Ok(())
}

pub struct Client {
    stream: std::io::BufReader<std::net::TcpStream>,
    buf: Vec<u8>,
}

impl Client {
    fn new(stream: std::net::TcpStream) -> Self {
        Self {
            stream: std::io::BufReader::new(stream),
            buf: Vec::new(),
        }
    }

    fn recv(&mut self) -> Result<Response> {
        self.buf.clear();
        self.stream.read_until(b'\0', &mut self.buf)?;
        let resp = serde_json::from_slice(&self.buf[0..self.buf.len() - 1])?;
        Ok(resp)
    }

    fn send(&mut self, req: Request) -> Result<()> {
        self.buf.clear();
        serde_json::to_writer(&mut self.buf, &req)?;
        self.buf.push(b'\0');
        self.stream.get_mut().write_all(&self.buf)?;
        Ok(())
    }
}
