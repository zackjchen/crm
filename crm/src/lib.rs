pub mod abi;
pub mod config;
pub mod pb;

use config::AppConfig;
use crm_server::CrmServer;
use pb::{crm_server::Crm, *};

use crm_metadata::pb::metadata::metadata_client::MetadataClient;
use crm_send::pb::send::notification_client::NotificationClient;
use tonic::{transport::Channel, Request, Response, Status};
use user_stat::pb::user_stats::user_stats_client::UserStatsClient;

#[allow(unused)]
pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    notification: NotificationClient<Channel>,
    metadata: MetadataClient<Channel>,
}

#[tonic::async_trait]
impl Crm for CrmService {
    /// user has registered in x days, and given welcome message
    async fn welcome(
        &self,
        request: Request<WelcomeRequest>,
    ) -> Result<Response<WelcomeResponse>, Status> {
        self.welcome(request.into_inner()).await
    }
    /// last visited in x days, and given them something to watch
    async fn recall(
        &self,
        _request: tonic::Request<RecallRequest>,
    ) -> Result<tonic::Response<RecallResponse>, tonic::Status> {
        todo!()
    }
    /// last watched in x days, and user still have unfinished contents
    async fn remind(
        &self,
        _request: tonic::Request<RemindRequest>,
    ) -> Result<tonic::Response<RemindResponse>, Status> {
        todo!()
    }
}

impl CrmService {
    pub async fn new(config: AppConfig) -> Result<Self, tonic::transport::Error> {
        let user_stats = UserStatsClient::connect(config.server.user_stats.clone())
            .await
            .unwrap();
        let notification = NotificationClient::connect(config.server.notification.clone())
            .await
            .unwrap();
        let metadata = MetadataClient::connect(config.server.metadata.clone())
            .await
            .unwrap();
        Ok(Self {
            config,
            user_stats,
            notification,
            metadata,
        })
    }

    pub fn into_server(self) -> CrmServer<Self> {
        CrmServer::new(self)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::net::SocketAddr;

//     use super::*;
//     use crm_client::CrmClient;
//     #[tokio::test]
//     async fn test_crm_should_work() -> anyhow::Result<()> {
//         let addr = start_server().await;
//         println!("Server started at {}", addr);
//         let client = CrmClient::connect("http://localhost:50000".to_string())
//             .await
//             .unwrap();
//         let request = Request::new(WelcomeRequest {
//             id: "1".to_string(),
//             interval: 100,
//             content_ids: vec![2, 3],
//         });
//         let res = client.clone().welcome(request).await?.into_inner();
//         println!("{:?}", res);
//         Ok(())
//     }

//     async fn start_server() -> SocketAddr {
//         let config = AppConfig::load().unwrap();
//         let addr = format!("[::1]:{}", config.server.port).parse().unwrap();
//         let service = CrmService::new(config).await.unwrap();
//         let svc = service.into_server();
//         tokio::spawn(async move {
//             tonic::transport::Server::builder()
//                 .add_service(svc)
//                 .serve(addr)
//                 .await
//                 .unwrap();
//         });
//         addr
//     }
// }
