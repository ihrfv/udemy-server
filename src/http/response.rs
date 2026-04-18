use tokio::io::AsyncWriteExt;

use super::StatusCode;

pub struct Response {
    status_code: StatusCode,
    body: Option<String>,
}

impl Response {
    pub fn new(status_code: StatusCode, body: Option<String>) -> Self {
        Response { status_code, body }
    }

    pub async fn send<T>(&self, stream: &mut T) -> Result<(), std::io::Error>
    where
        T: AsyncWriteExt + Unpin,
    {
        let body = match &self.body {
            Some(b) => b,
            None => "",
        };
        let response_str = format!(
            "HTTP/1.1 {} {}\r\n\r\n{}",
            self.status_code,
            self.status_code.reason_phrase(),
            body
        );
        stream.write_all(response_str.as_bytes()).await
    }
}
