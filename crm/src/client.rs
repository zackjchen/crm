use crm::pb::crm::{user_service_client::UserServiceClient, GetUserRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(GetUserRequest { id: 101 });
    let response = client.get_user(request).await?;
    println!("Response:{:?}", response);
    Ok(())
}
