use crm_send::{config::AppConfig, NotificationService};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() {
    let layer = Layer::default().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = AppConfig::load().unwrap();
    let addr = format!("[::1]:{}", config.server.port).parse().unwrap();
    let svc = NotificationService::new(config).await.into_server();
    info!("Starting server at {}", addr);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .unwrap();
}
