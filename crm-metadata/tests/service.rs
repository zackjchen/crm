use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use crm_metadata::{
    config::AppConfig,
    pb::metadata::{metadata_client::MetadataClient, MaterializeRequest},
    MetadataService,
};
use tokio::time;
use tokio_stream::StreamExt;
use tracing::info;

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    let addr = start_server().await?;
    info!("Starting server at {}", addr);
    let requests = vec![MaterializeRequest { id: 1 }];
    let stream = tokio_stream::iter(requests);
    let mut client = MetadataClient::connect(format!("http://{}", addr)).await?;
    let res = client.materialize(stream).await?;
    let contents = res
        .into_inner()
        .collect::<Vec<_>>()
        .await
        .iter()
        .map(|r| r.clone().unwrap())
        .collect::<Vec<_>>();

    println!("{:?}", contents);
    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let config = AppConfig::load().unwrap();
    let addr = format!("[::1]:{}", config.server.port).parse().unwrap();
    let svc = MetadataService::new(config).into_server();
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .unwrap();
    });

    time::sleep(Duration::from_secs(1)).await;
    Ok(addr)
}
