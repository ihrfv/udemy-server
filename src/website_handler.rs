use super::http::{Method, Request, Response, StatusCode};
use super::server::Handler;
use async_trait::async_trait;
use tokio::time::{Duration, sleep};

pub struct WebsiteHandler {
    public_path: String,
}

impl WebsiteHandler {
    pub fn new(public_path: String) -> Self {
        WebsiteHandler { public_path }
    }

    fn read_file(&self, file_path: &str) -> Option<String> {
        let path = format!("{}/{}", self.public_path, file_path);
        match std::fs::canonicalize(path) {
            Ok(path) => {
                if path.starts_with(&self.public_path) {
                    std::fs::read_to_string(path).ok()
                } else {
                    println!("Directory Traversal Attack Attempted: {}", file_path);
                    None
                }
            }
            Err(_) => None,
        }
    }
}

#[async_trait]
impl Handler for WebsiteHandler {
    async fn handle_request(&self, request: &Request) -> Response {
        match request.method() {
            Method::GET => match request.path() {
                "/" => Response::new(StatusCode::Ok, self.read_file("index.html")),
                "/hello" => Response::new(StatusCode::Ok, self.read_file("hello.html")),
                "/sleep" => {
                    sleep(Duration::from_secs(5)).await;
                    Response::new(StatusCode::Ok, self.read_file("index.html"))
                }
                path => match self.read_file(path) {
                    Some(contents) => Response::new(StatusCode::Ok, Some(contents)),
                    None => Response::new(StatusCode::NotFound, None),
                },
            },
            _ => Response::new(StatusCode::NotFound, None),
        }
    }
}
