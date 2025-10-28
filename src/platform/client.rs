use std::io::{BufRead, Write};
use std::sync::{OnceLock, mpsc};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

static TX_REQUEST: OnceLock<mpsc::Sender<Request>> = OnceLock::new();

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    Start {},
    UnlockAchivement { name: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    Start {},
    UnlockAchivement {},
}

pub fn send_request(req: Request) {
    if let Some(tx) = TX_REQUEST.get()
        && let Err(e) = tx.send(req)
    {
        log::warn!("cannot send to client stream task: {e}");
    }
}

pub fn run_client(port: u16) {
    let (tx, rx) = mpsc::channel();
    TX_REQUEST.set(tx).unwrap();
    let stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}"))
        .expect("cannot open stream with launcher");

    std::thread::spawn(move || {
        if let Err(e) = client_task(stream, rx) {
            eprintln!("launcher connection error: {e:?}");
        }
    });
}

fn client_task(stream: std::net::TcpStream, rx: mpsc::Receiver<Request>) -> Result<()> {
    let mut client = Client::new(stream);

    client.send(Request::Start {})?;
    let resp = client.recv()?;
    if !matches!(resp, Response::Start {}) {
        bail!("invalid response");
    }
    eprintln!("connected to game launcher");

    while let Ok(req) = rx.recv() {
        client.send(req)?;
        let resp = client.recv()?;
        log::info!("response {resp:?}");
    }

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
