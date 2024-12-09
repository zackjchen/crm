use crm_metadata::{config::AppConfig, MetadataService};
use tracing::info;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _}; // Add this line to import the AppConfig type

#[tokio::main]
async fn main() {
    let layer = Layer::default().with_filter(tracing_subscriber::filter::LevelFilter::INFO); // Create a new Layer with the INFO level
    tracing_subscriber::registry().with(layer).init(); // Initialize the subscriber with the layer
    let config = AppConfig::load().unwrap(); // Load the configuration
    let addr = config.server.port; // Get the server address from the configuration
    let addr = format!("[::1]:{}", addr); // Format the address
    info!("Starting server on {}", addr); // Log the address
    let svc = MetadataService::new(config).into_server(); // Create a new MetadataServer
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
