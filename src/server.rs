use crate::http::{ParseError, Request, Response, StatusCode};
use async_trait::async_trait;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

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

    pub async fn run<HANDLER>(
        &self,
        handler: HANDLER,
        mut shutdown: tokio::sync::watch::Receiver<()>,
    ) where
        HANDLER: Handler + Send + Sync + 'static,
    {
        println!("Listening on {}", self.addr);
        let listener = TcpListener::bind(&self.addr).await.unwrap();
        let handler = Arc::new(handler);
        let cancelation_token = CancellationToken::new();
        let task_tracker = TaskTracker::new();
        loop {
            tokio::select! {
                res = listener.accept() => {
                    match res {
                        Ok((stream, _)) => {
                            let handler = Arc::clone(&handler);
                            let token = cancelation_token.clone();
                            task_tracker.spawn(async move {
                                // There is a need to use cancelation_token becase some connections
                                // can be kept alive with a keep_alive message and therefore the
                                // spawned task would never end
                                tokio::select! {
                                    _ = handler.handle_connection(stream) => {}
                                    _ = token.cancelled() => {
                                        // TODO: improve shutdown notification
                                    }
                                }
                            });
                        }
                        Err(error) => eprintln!("Failed to establish a connection: {}", error),
                    }
                }
                _ = shutdown.changed() => break,
            }
        }

        cancelation_token.cancel();
        println!("Waiting for every task to finish");
        task_tracker.close();
        task_tracker.wait().await;
        println!("Server stopped.");
    }
}
