pub mod abi;
pub mod config;
pub mod pb;

use config::AppConfig;
use pb::send::{notification_server::Notification, send_request::Msg, SendRequest, SendResponse};
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

#[derive(Debug, Clone)]
pub struct NotificationService {
    inner: Arc<NotificationServiceInner>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct NotificationServiceInner {
    config: AppConfig,
    sender: mpsc::Sender<Msg>,
}

type ServiceResult<T> = std::result::Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<SendResponse, Status>> + Send>>;

#[tonic::async_trait]
impl Notification for NotificationService {
    /// Server streaming response type for the Send method.
    type SendStream = ResponseStream;
    async fn send(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> ServiceResult<ResponseStream> {
        self.send(request.into_inner()).await
    }
}

pub async fn dummy_send() -> mpsc::Sender<Msg> {
    let (tx, mut rx) = mpsc::channel(1024);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            info!("Sent message: {:?}", msg);
            println!("Sent message: {:?}", msg);
            sleep(Duration::from_millis(300)).await;
        }
    });
    tx
}
