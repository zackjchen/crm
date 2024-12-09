use anyhow::Result;
use sqlx_db_tester::TestPg;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::info;
use user_stat::{
    pb::user_stats::{user_stats_client::UserStatsClient, QueryRequestBuilder},
    test_utils::{new_timequery, to_timestamp},
    UserStatsService,
};
const PORT_BASE: u32 = 60000;

#[tokio::test]
async fn raw_query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server(PORT_BASE).await?;
    let mut client = UserStatsClient::connect(format!("http://{}", addr)).await?;
    let request = tonic::Request::new(user_stat::pb::user_stats::RawQueryRequest {
        query: "SELECT * from user_stats limit 2".to_string(),
    });
    let res = client.raw_query(request).await?;
    let users = res
        .into_inner()
        .collect::<Vec<_>>()
        .await
        .iter()
        .map(|r| r.clone().unwrap())
        .collect::<Vec<_>>();
    println!("{:?}", users);
    assert_eq!(users.len(), 2);
    Ok(())
}

#[tokio::test]
async fn query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server(PORT_BASE + 1).await?;
    let mut client = UserStatsClient::connect(format!("http://{}", addr)).await?;

    let after = to_timestamp(300);
    let before = to_timestamp(100);
    let query = QueryRequestBuilder::default()
        .timestamp(("created_at".to_string(), new_timequery(after, before)))
        .build()
        .unwrap();
    let request = tonic::Request::new(query);
    let res = client.query(request).await?;
    let users = res
        .into_inner()
        .collect::<Vec<_>>()
        .await
        .iter()
        .map(|r| r.clone().unwrap())
        .collect::<Vec<_>>();
    println!("{:?}", users);
    assert_eq!(users.len(), 112);
    Ok(())
}

async fn start_server(port: u32) -> Result<(TestPg, SocketAddr)> {
    let addr = format!("[::1]:{}", port).parse()?;

    let (tdb, svc) = UserStatsService::new_for_test().await;
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(svc.into_server())
            .serve(addr)
            .await
            .unwrap();
    });
    info!("Starting server at {}", addr);
    // 这里等一微秒，等这个服务起来，不然会报错
    sleep(Duration::from_micros(1)).await;
    Ok((tdb, addr))
}

#[test]
fn test_1() {
    let env1 = env!("CARGO_MANIFEST_DIR");
    println!("env:{:?}", env1);
}
