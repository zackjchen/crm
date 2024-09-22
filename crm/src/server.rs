use crm::pb::crm::{
    user_service_server::UserServiceServer, CreateUserRequest, GetUserRequest, User,
};
// use prost::Message;
use crm::pb::crm::user_service_server::UserService;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
struct UserServer;

#[tonic::async_trait]
impl UserService for UserServer {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let inner = request.into_inner();
        println!("get user request:{:?}", inner);
        Ok(Response::new(User::default()))
    }
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let inner = request.into_inner();
        println!("create user request:{:?}", inner);
        Ok(Response::new(User::default()))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let user =User::new(1, "Alice", "zack.j.chen@hkjc.org.hk");
    // let encode = user.encode_to_vec();
    // println!("encode:{:?}", encode);
    let addr = "[::1]:50051".parse().unwrap();
    let user_server: UserServer = Default::default();
    println!("User Server running on {}", addr);
    tonic::transport::Server::builder()
        .add_service(UserServiceServer::new(user_server))
        .serve(addr)
        .await?;

    Ok(())
}
