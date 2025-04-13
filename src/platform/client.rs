use anyhow::Result;

pub fn run_client(port: u16) {
    let stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", port))
        .expect("cannot open stream with launcher");

    std::thread::spawn(move || {
        if let Err(e) = client_task(stream) {
            log::error!("launcher connection error: {:?}", e);
        }
    });
}

fn client_task(_stream: std::net::TcpStream) -> Result<()> {
    Ok(())
}
