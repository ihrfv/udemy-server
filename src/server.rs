use crate::http::{ParseError, Request, Response, StatusCode};
use std::convert::TryFrom;
use std::io::Read;
use std::net::TcpListener;

pub trait Handler {
    fn handle_request(&mut self, request: &Request) -> Response;

    fn handle_bad_request(&mut self, error: &ParseError) -> Response {
        eprintln!("Failed to parse a request: {}", error);
        Response::new(StatusCode::BadRequest, None)
    }
}

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Server { addr }
    }

    pub fn run(&self, mut handler: impl Handler) {
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
                            let response = match Request::try_from(&buffer[..]) {
                                Ok(request) => handler.handle_request(&request),
                                Err(error) => handler.handle_bad_request(&error),
                            };
                            if let Err(error) = response.send(&mut stream) {
                                eprintln!("Failed to send a response: {error}");
                            }
                        }
                        Err(error) => {
                            eprintln!("Failed to read from the connection: {}", error)
                        }
                    }
                }
                Err(error) => eprintln!("Failed to establish a connection: {}", error),
            }
        }
    }
}
