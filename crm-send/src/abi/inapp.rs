use fake::{faker::lorem::en::Sentence, Fake};
use tonic::Status;
use tracing::warn;
use uuid::Uuid;

use super::{to_timestamp, Sender};
use crate::{
    pb::send::{send_request::Msg, InAppMessage, SendRequest, SendResponse},
    NotificationService,
};

impl Sender for InAppMessage {
    async fn send(self, id: String, svc: NotificationService) -> Result<SendResponse, Status> {
        let response = SendResponse {
            message_id: id,
            timestamp: Some(to_timestamp()),
        };
        svc.sender.send(Msg::InApp(self)).await.map_err(|e| {
            warn!("failed to send in-app message: {}", e);
            Status::internal("failed to send in-app message")
        })?;
        Ok(response)
    }
}

#[cfg(feature = "test_utils")]
impl InAppMessage {
    pub fn fake() -> Self {
        Self {
            title: Sentence(10..20).fake(),
            body: Sentence(20..30).fake(),
            device_id: Sentence(10..11).fake(),
        }
    }
}
impl From<InAppMessage> for SendRequest {
    fn from(msg: InAppMessage) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            msg: Some(Msg::InApp(msg)),
        }
    }
}
