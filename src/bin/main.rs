use std::env;
use udemy_server::Server;
use udemy_server::WebsiteHandler;

fn main() {
    let defalut_public_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
    let public_path = env::var("PUBLIC_PATH").unwrap_or(defalut_public_path);
    let mut handler = WebsiteHandler::new(public_path);

    let server = Server::new("127.0.0.1:8080".to_string());
    server.run(&mut handler);
}
