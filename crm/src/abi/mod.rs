use chrono::{Duration, Utc};
use crm_metadata::pb::metadata::{Content, MaterializeRequest};
use crm_send::pb::send::{send_request::Msg, EmailMessage, SendRequest};
use prost_types::Timestamp;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Response, Status};
use user_stat::{
    pb::user_stats::{QueryRequest, QueryRequestBuilder},
    test_utils::new_timequery,
};

use crate::{
    CrmService, RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest,
    WelcomeResponse,
};

impl CrmService {
    // 整个逻辑是，
    // 1. 根据interval，请求user_stats服务，获取所有用户
    // 2. 根据content_ids请求metadata服务，获取用户metadata信息，比如like， dislike等
    // 3. 根据content_ids请求crm-send的notification服务，通知用户
    pub async fn welcome(
        &self,
        request: WelcomeRequest,
    ) -> Result<Response<WelcomeResponse>, Status> {
        let id = request.id.clone();
        let date = Utc::now() - Duration::days(request.interval as _);
        let after = Timestamp {
            seconds: date.timestamp(),
            nanos: date.timestamp_subsec_nanos() as i32,
        };

        let before = Timestamp {
            seconds: Utc::now().timestamp(),
            nanos: Utc::now().timestamp_subsec_nanos() as i32,
        };

        let user_stat_req = get_user_stats_req("created_at", before, after);
        let mut users = self
            .user_stats
            .clone()
            .query(user_stat_req)
            .await?
            .into_inner();

        let materialize_req = MaterializeRequest::new_with_ids(&request.content_ids);
        let contents: Vec<Content> = self
            .metadata
            .clone()
            .materialize(materialize_req)
            .await
            .unwrap()
            .into_inner()
            .filter_map(|v| v.ok())
            .collect()
            .await;

        let sender = self.config.server.sender.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        tokio::spawn(async move {
            while let Some(Ok(user)) = users.next().await {
                let contents = contents.clone();
                let sender = sender.clone();
                let tx = tx.clone();
                let msg = EmailMessage {
                    subject: format!("Welcome to our platform, {}", user.name),
                    from: sender,
                    to: vec![user.email],
                    body: format!("{:?}", contents),
                };
                let send_req = SendRequest {
                    message_id: id.clone(),
                    msg: Some(Msg::Email(msg)),
                };
                tx.send(send_req).await.unwrap();
            }
        });
        let send_reqs = ReceiverStream::from(rx);
        self.notification.clone().send(send_reqs).await?;

        let ret = WelcomeResponse { id: request.id };
        Ok(Response::new(ret))
    }

    pub fn remind(&self, _request: RemindRequest) -> Result<RemindResponse, Status> {
        todo!()
    }

    pub fn recall(&self, _request: RecallRequest) -> Result<RecallResponse, Status> {
        todo!()
    }
}

fn get_user_stats_req(name: &str, before: Timestamp, after: Timestamp) -> QueryRequest {
    let req = QueryRequestBuilder::default()
        .timestamp((name.to_string(), new_timequery(after, before)))
        .build()
        .unwrap();
    req
}
