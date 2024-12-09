use crm::pb::{crm_client::CrmClient, WelcomeRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = CrmClient::connect("http://localhost:50000".to_string())
        .await
        .unwrap();
    let request = Request::new(WelcomeRequest {
        id: "1".to_string(),
        interval: 100,
        content_ids: vec![2, 3],
    });
    let res = client.clone().welcome(request).await?.into_inner();
    println!("{:?}", res);
    Ok(())
}
