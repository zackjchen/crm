mod email;
mod inapp;
mod sms;

use crate::pb::send::{send_request::Msg, SendResponse};
use crate::{
    config::AppConfig,
    dummy_send,
    pb::send::{notification_server::NotificationServer, SendRequest},
    NotificationService, NotificationServiceInner, ResponseStream, ServiceResult,
};
use chrono::Utc;
use prost_types::Timestamp;
use std::{ops::Deref, sync::Arc};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Response, Status};

impl NotificationService {
    pub async fn new(config: AppConfig) -> Self {
        let sender = dummy_send().await;
        let inner = NotificationServiceInner { config, sender };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> NotificationServer<Self> {
        NotificationServer::new(self)
    }

    pub async fn send(
        &self,
        mut stream: impl Stream<Item = Result<SendRequest, Status>> + Send + 'static + Unpin,
    ) -> ServiceResult<ResponseStream> {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        let notif_clone = self.clone();
        tokio::spawn(async move {
            while let Some(Ok(req)) = stream.next().await {
                // Clone the sender to be moved into the async block
                // 不能直接clone，不然self会被move进来然后报错
                let notif_clone = notif_clone.clone();
                let response = match req.msg {
                    Some(Msg::Email(e)) => e.send(req.message_id, notif_clone).await,
                    Some(Msg::Sms(e)) => e.send(req.message_id, notif_clone).await,
                    Some(Msg::InApp(e)) => e.send(req.message_id, notif_clone).await,
                    None => Err(Status::invalid_argument("missing message")),
                };
                tx.send(response).await.unwrap();
            }
        });
        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}

fn to_timestamp() -> Timestamp {
    let now = Utc::now();
    Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    }
}

#[allow(async_fn_in_trait)]
pub trait Sender {
    async fn send(self, id: String, svc: NotificationService) -> Result<SendResponse, Status>;
}

impl Deref for NotificationService {
    type Target = NotificationServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::AppConfig,
        pb::send::{EmailMessage, InAppMessage, SmsMessage},
    };

    #[tokio::test]
    async fn test_send_should_work() {
        let config = AppConfig::load().unwrap();
        let svc = NotificationService::new(config).await;
        let stream = tokio_stream::iter(vec![
            Ok(EmailMessage::fake().into()),
            Ok(SmsMessage::fake().into()),
            Ok(InAppMessage::fake().into()),
        ]);
        let response = svc.send(stream).await.unwrap();
        let mut stream = response.into_inner();
        let email_response = stream.next().await.unwrap().unwrap();
        println!("Email response: {:?}", email_response);
    }
}
