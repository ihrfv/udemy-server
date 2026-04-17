use crate::ThreadPool;
use crate::http::{ParseError, Request, Response, StatusCode};
use std::convert::TryFrom;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub trait Handler {
    fn handle_request(&self, request: &Request) -> Response;

    fn handle_bad_request(&self, error: &ParseError) -> Response {
        eprintln!("Failed to parse a request: {}", error);
        Response::new(StatusCode::BadRequest, None)
    }

    fn handle_connection(&self, stream: &mut TcpStream) {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(bytes_written) => {
                println!("Received a request: {}", String::from_utf8_lossy(&buffer));
                let response = match Request::try_from(&buffer[..bytes_written]) {
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

    pub fn run<HANDLER>(&self, handler: HANDLER)
    where
        HANDLER: Handler + Send + Sync + 'static,
    {
        println!("Listening on {}", self.addr);
        let listener = TcpListener::bind(&self.addr).unwrap();
        let handler = Arc::new(handler);

        let threads_capacity = std::thread::available_parallelism().map_or(1, |x| x.get());
        println!("Threads to be in the pool {threads_capacity}");
        let thread_pool = ThreadPool::new(threads_capacity);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let handler = Arc::clone(&handler);
                    thread_pool.execute(move || handler.handle_connection(&mut stream));
                }
                Err(error) => eprintln!("Failed to establish a connection: {}", error),
            }
        }
    }
}
