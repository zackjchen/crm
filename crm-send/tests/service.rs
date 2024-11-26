use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use crm_send::{
    config::AppConfig,
    pb::send::{
        notification_client::NotificationClient, EmailMessage, InAppMessage, SendRequest,
        SmsMessage,
    },
    NotificationService,
};
use tokio::time;
use tokio_stream::StreamExt;
use tracing::info;

#[tokio::test]
async fn main() -> Result<()> {
    let addr = start_server().await?;
    info!("Starting server at {}", addr);

    let requests: Vec<SendRequest> = vec![
        EmailMessage::fake().into(),
        InAppMessage::fake().into(),
        SmsMessage::fake().into(),
    ];
    let stream = tokio_stream::iter(requests);

    let mut client = NotificationClient::connect(format!("http://{}", addr)).await?;
    let res = client
        .send(stream)
        .await?
        .into_inner()
        .collect::<Vec<_>>()
        .await;
    println!("{:?}", res);
    assert_eq!(res.len(), 3);
    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let config = AppConfig::load().unwrap();
    let addr = format!("[::1]:{}", config.server.port).parse().unwrap();
    let svc = NotificationService::new(config).await.into_server();
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
