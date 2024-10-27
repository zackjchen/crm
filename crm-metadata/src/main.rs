use crm_metadata::{config::AppConfig, MetadataService};
use tracing::info; // Add this line to import the AppConfig type

#[tokio::main]
async fn main() {
    let config = AppConfig::load().unwrap(); // Load the configuration
    let addr = config.server.port; // Get the server address from the configuration
    let addr = format!("0.0.0.0:{}", addr); // Format the address
    info!("Starting server on {}", addr); // Log the address
    let svc = MetadataService::new(config).into_server(); // Create a new MetadataServer
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
