use crate::http::{ParseError, Request, Response, StatusCode};
use std::convert::TryFrom;
use std::io::Read;
use std::net::{TcpListener, TcpStream};

pub trait Handler {
    fn handle_request(&self, request: &Request) -> Response;

    fn handle_bad_request(&self, error: &ParseError) -> Response {
        eprintln!("Failed to parse a request: {}", error);
        Response::new(StatusCode::BadRequest, None)
    }

    fn handle_connection(&self, stream: &mut TcpStream) {
        // TODO: update buffer to allow requests of arbitrary (reasonable) size
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(_bytes_written) => {
                println!("Received a request: {}", String::from_utf8_lossy(&buffer));
                let response = match Request::try_from(&buffer[..]) {
                    Ok(request) => self.handle_request(&request),
                    Err(error) => self.handle_bad_request(&error),
                };
                if let Err(error) = response.send(stream) {
                    eprintln!("Failed to send a response: {error}");
                }
            }
            Err(error) => {
                eprintln!("Failed to read from the connection: {}", error)
            }
        }
    }
}

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Server { addr }
    }

    pub fn run(&self, handler: impl Handler) {
        println!("Listening on {}", self.addr);
        let listener = TcpListener::bind(&self.addr).unwrap();
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => handler.handle_connection(&mut stream),
                Err(error) => eprintln!("Failed to establish a connection: {}", error),
            }
        }
    }
}
