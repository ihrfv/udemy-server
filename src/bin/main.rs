use std::env;
use udemy_server::Server;
use udemy_server::WebsiteHandler;

#[tokio::main]
async fn main() {
    let defalut_public_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
    let public_path = env::var("PUBLIC_PATH").unwrap_or(defalut_public_path);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(());
    let server = Server::new("127.0.0.1:8080".to_string());

    let server_join_handler = tokio::spawn(async move {
        server
            .run(WebsiteHandler::new(public_path), shutdown_rx)
            .await;
    });

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for CTRL_C");
    println!("Signaling server shutdown due to CTRL_C keystroke");
    let _ = shutdown_tx.send(());

    server_join_handler
        .await
        .expect("Error while waiting for server to shutdown");
}
