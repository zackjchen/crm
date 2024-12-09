use super::{to_timestamp, Sender};
use crate::{
    pb::send::{send_request::Msg, SendRequest, SendResponse, SmsMessage},
    NotificationService,
};
use fake::{
    faker::{internet::zh_cn::SafeEmail, name::en::Name},
    Fake,
};
use tonic::Status;
use tracing::warn;
use uuid::Uuid;

impl Sender for SmsMessage {
    async fn send(self, id: String, svc: NotificationService) -> Result<SendResponse, Status> {
        let response = SendResponse {
            message_id: id,
            timestamp: Some(to_timestamp()),
        };
        svc.sender.send(Msg::Sms(self)).await.map_err(|e| {
            warn!("failed to send sms: {}", e);
            Status::internal("failed to send sms")
        })?;
        Ok(response)
    }
}

#[cfg(feature = "test_utils")]
impl SmsMessage {
    pub fn fake() -> Self {
        Self {
            sender: SafeEmail().fake(),
            recipients: vec![SafeEmail().fake()],
            body: format!("Sms Message: hello {}", Name().fake::<String>()),
        }
    }
}

impl From<SmsMessage> for SendRequest {
    fn from(msg: SmsMessage) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            msg: Some(Msg::Sms(msg)),
        }
    }
}
