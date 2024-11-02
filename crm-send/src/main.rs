use crm_send::{config::AppConfig, NotificationService};
use tracing::info;

#[tokio::main]
async fn main() {
    let config = AppConfig::load().unwrap();
    let addr = format!("[::1]:{}", config.server.port).parse().unwrap();
    let svc = NotificationService::new(config).await.into_server();
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .unwrap();
    info!("Starting server at {}", addr);
}
