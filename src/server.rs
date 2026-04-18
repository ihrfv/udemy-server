use crate::http::{ParseError, Request, Response, StatusCode};
use async_trait::async_trait;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle_request(&self, request: &Request) -> Response;

    fn handle_bad_request(&self, error: &ParseError) -> Response {
        eprintln!("Failed to parse a request: {}", error);
        Response::new(StatusCode::BadRequest, None)
    }

    async fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer).await {
            Ok(0) => {
                // socket is closed
                return;
            }
            Ok(bytes_written) => {
                println!("Received a request: {}", String::from_utf8_lossy(&buffer));
                let response = match Request::try_from(&buffer[..bytes_written]) {
                    Ok(request) => self.handle_request(&request).await,
                    Err(error) => self.handle_bad_request(&error),
                };

                if let Err(error) = response.send(&mut stream).await {
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

    pub async fn run<HANDLER>(&self, handler: HANDLER)
    where
        HANDLER: Handler + Send + Sync + 'static,
    {
        println!("Listening on {}", self.addr);
        let listener = TcpListener::bind(&self.addr).await.unwrap();
        let handler = Arc::new(handler);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let handler = Arc::clone(&handler);
                    tokio::spawn(async move {
                        handler.handle_connection(stream).await;
                    });
                }
                Err(error) => eprintln!("Failed to establish a connection: {}", error),
            }
        }
    }
}
