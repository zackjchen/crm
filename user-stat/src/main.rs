use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
use user_stat::{AppConfig, UserStatsService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let user =User::new(1, "Alice", "zack.j.chen@hkjc.org.hk");
    // let encode = user.encode_to_vec();
    // println!("encode:{:?}", encode);
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("[::1]:{}", config.server.port).parse().unwrap();
    info!("User Server running on {}", addr);
    let server = UserStatsService::new(config).await.into_server();
    tonic::transport::Server::builder()
        .add_service(server)
        .serve(addr)
        .await?;

    Ok(())
}
