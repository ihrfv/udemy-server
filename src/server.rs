use crate::http::Request;
use std::convert::TryFrom;
use std::io::Read;
use std::net::TcpListener;

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Server { addr }
    }

    pub fn run(&self) {
        println!("Listening on {}", self.addr);
        let listener = TcpListener::bind(&self.addr).unwrap();

        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    // TODO: update buffer to allow requests of arbitrary (reasonable) size
                    let mut buffer = [0; 1024];
                    match stream.read(&mut buffer) {
                        Ok(bytes_written) => {
                            println!(
                                "Received a request: {}. Bytes written: {bytes_written}",
                                String::from_utf8_lossy(&buffer)
                            );
                            match Request::try_from(&buffer[..]) {
                                Ok(result) => {
                                    // dbg!(result);
                                }
                                Err(error) => {
                                    println!("[ERROR] Failed to parse a request: {}", error)
                                }
                            }
                        }
                        Err(error) => {
                            println!("[ERROR] Failed to read from the connection: {}", error)
                        }
                    }
                }
                Err(error) => println!("[ERROR] Failed to establish a connection: {}", error),
            }
        }
    }
}
