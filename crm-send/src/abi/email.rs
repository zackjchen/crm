use super::{to_timestamp, Sender};
use crate::{
    pb::send::{send_request::Msg, EmailMessage, SendRequest, SendResponse},
    NotificationService,
};
use fake::{faker::internet::zh_cn::SafeEmail, Fake};
use tonic::Status;
use tracing::warn;
use uuid::Uuid;

impl Sender for EmailMessage {
    async fn send(self, id: String, svc: NotificationService) -> Result<SendResponse, Status> {
        let response = SendResponse {
            message_id: id,
            timestamp: Some(to_timestamp()),
        };
        svc.sender.send(Msg::Email(self)).await.map_err(|e| {
            warn!("failed to send email: {}", e);
            Status::internal("failed to send email")
        })?;
        Ok(response)
    }
}
#[cfg(feature = "test_utils")]
impl EmailMessage {
    pub fn fake() -> Self {
        Self {
            subject: "Hello".to_string(),
            from: SafeEmail().fake(),
            to: vec![SafeEmail().fake()],
            body: "hello world".to_string(),
        }
    }
}

impl From<EmailMessage> for SendRequest {
    fn from(msg: EmailMessage) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            msg: Some(Msg::Email(msg)),
        }
    }
}
