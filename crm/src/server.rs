use crm::{config::AppConfig, CrmService};
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::default().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load().unwrap();
    let addr = format!("[::1]:{}", config.server.port).parse().unwrap();
    info!("Starting server on {}", addr);
    let svc = CrmService::new(config).await?.into_server();
    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .unwrap();

    Ok(())
}
